// Copyright (c) 2018-2022 The MobileCoin Foundation

//! Helpers for server pipelines.

use futures::{Stream, StreamExt};
use grpcio::Environment;
use mc_common::logger::Logger;
use mc_ledger_db::LedgerDB;
use mc_ledger_streaming_api::{
    BlockData, BlockIndex, Error, Result, Streamer, DEFAULT_MERGED_BLOCKS_BUCKET_SIZES,
};
use mc_ledger_streaming_client::{DbStream, GrpcBlockSource};
#[cfg(feature = "publisher_local")]
use mc_ledger_streaming_publisher::LocalFileProtoWriter;
use mc_ledger_streaming_publisher::{ArchiveBlockSink, GrpcServerSink, ProtoWriter};
#[cfg(feature = "publisher_s3")]
use mc_ledger_streaming_publisher::{S3ClientProtoWriter, S3Region};
use mc_util_uri::ConnectionUri;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub struct LedgerToArchiveBlocksAndGrpc<US, W>
where
    US: Streamer<Result<BlockData>, BlockIndex> + Sync + 'static,
    W: ProtoWriter + 'static,
{
    pub source: DbStream<US, LedgerDB>,
    pub archive_block_sink: ArchiveBlockSink<W, LedgerDB>,
    pub archive_block_writer: W,
    pub grpc_sink: GrpcServerSink,
}

impl<US, W> LedgerToArchiveBlocksAndGrpc<US, W>
where
    US: Streamer<Result<BlockData>, BlockIndex> + Sync + 'static,
    W: ProtoWriter + 'static,
{
    pub fn new(
        upstream: US,
        ledger_path: impl AsRef<Path>,
        archive_block_writer: W,
        logger: Logger,
    ) -> Result<Self> {
        let _ = LedgerDB::create(ledger_path.as_ref());
        let ledger = LedgerDB::open(ledger_path.as_ref())?;
        let source = DbStream::new(upstream, ledger.clone(), true, logger.clone());
        let archive_block_sink = ArchiveBlockSink::new(
            archive_block_writer.clone(),
            ledger,
            DEFAULT_MERGED_BLOCKS_BUCKET_SIZES.to_vec(),
            logger.clone(),
        );
        let grpc_sink = GrpcServerSink::new(logger);
        Ok(Self {
            source,
            archive_block_sink,
            archive_block_writer,
            grpc_sink,
        })
    }

    pub fn run(&self, starting_height: u64) -> Result<impl Stream<Item = Result<()>> + Send + '_> {
        let stream = self.source.get_stream(starting_height)?;
        // Write ArchiveBlock(s), and if that succeeds, publish them to gRPC.
        Ok(stream.then(move |result| async move {
            let block_data = result?;
            let archive_block = self.archive_block_sink.write(&block_data).await?;
            self.grpc_sink.publish(archive_block).await;
            Ok(())
        }))
    }
}

#[cfg(feature = "publisher_local")]
impl<US> LedgerToArchiveBlocksAndGrpc<US, LocalFileProtoWriter>
where
    US: Streamer<Result<BlockData>, BlockIndex> + Sync + 'static,
{
    pub fn new_local(
        upstream: US,
        ledger_path: impl AsRef<Path>,
        block_path_base: impl Into<PathBuf>,
        logger: Logger,
    ) -> Result<Self> {
        Self::new(
            upstream,
            ledger_path,
            LocalFileProtoWriter::new(block_path_base.into()),
            logger,
        )
    }
}

#[cfg(feature = "publisher_s3")]
impl<US> LedgerToArchiveBlocksAndGrpc<US, S3ClientProtoWriter>
where
    US: Streamer<Result<BlockData>, BlockIndex> + Sync + 'static,
{
    pub fn new_s3(
        upstream: US,
        ledger_path: impl AsRef<Path>,
        region: S3Region,
        s3_path: impl Into<PathBuf>,
        logger: Logger,
    ) -> Result<Self> {
        Self::new(
            upstream,
            ledger_path,
            S3ClientProtoWriter::new(region, s3_path.into()),
            logger,
        )
    }
}

/// A pipeline that subscribes to the given URI, and repeats any
/// [ArchiveBlock]s it receives from that URI over its own gRPC server.
#[derive(Debug)]
pub struct GrpcRepeater {
    pub source: GrpcBlockSource,
    pub sink: GrpcServerSink,
}

impl GrpcRepeater {
    pub fn new(grpc_uri: &impl ConnectionUri, env: Arc<Environment>, logger: Logger) -> Self {
        let source = GrpcBlockSource::new(grpc_uri, env, logger.clone());
        let sink = GrpcServerSink::new(logger);
        Self { source, sink }
    }

    /// The returned value is a `Stream` where the `Item` type is
    /// `Result<()>`; it is executed entirely for its side effects, while
    /// propagating errors back to the caller.
    pub fn subscribe_and_repeat(
        &self,
        starting_height: u64,
    ) -> Result<impl Stream<Item = Result<()>>> {
        let stream = self
            .source
            .subscribe(starting_height)?
            .map(|result| result.map_err(Error::from));
        Ok(self.sink.consume_protos(stream))
    }

    /// Create a [grpcio::Server] with a [LedgerUpdates] service backed by
    /// this pipeline.
    pub fn create_server(
        &self,
        uri: &impl ConnectionUri,
        env: Arc<grpcio::Environment>,
    ) -> grpcio::Result<grpcio::Server> {
        self.sink.create_server(uri, env)
    }

    /// Helper to create a local server.
    #[cfg(any(test, feature = "test_utils"))]
    pub fn create_local_server(
        &self,
        env: Arc<grpcio::Environment>,
    ) -> (grpcio::Server, mc_util_uri::ConsensusPeerUri) {
        self.sink.create_local_server(env)
    }
}
