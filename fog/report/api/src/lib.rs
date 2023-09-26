// Copyright (c) 2018-2022 The MobileCoin Foundation

mod autogenerated_code {
    // Expose proto data types from included third-party/external proto files.
    pub use mc_api::{blockchain, external};
    pub use protobuf::well_known_types::Empty;

    // Needed due to how to the auto-generated code references the Empty message.
    pub mod empty {
        pub use protobuf::well_known_types::Empty;
    }

    // Include the auto-generated code.
    include!(concat!(env!("OUT_DIR"), "/protos-auto-gen/mod.rs"));
}

pub use autogenerated_code::*;

// These are needed for tests
impl Eq for report::Report {}
impl Eq for report::ReportResponse {}

impl From<report::Report> for mc_fog_report_types::Report {
    fn from(mut src: report::Report) -> mc_fog_report_types::Report {
        mc_fog_report_types::Report {
            fog_report_id: src.take_fog_report_id(),
            attestation_evidence: (&src.take_attestation_evidence())
                .try_into()
                .unwrap_or_default(),
            pubkey_expiry: src.pubkey_expiry,
        }
    }
}

impl From<mc_fog_report_types::Report> for report::Report {
    fn from(src: mc_fog_report_types::Report) -> report::Report {
        let mut result = report::Report::new();
        result.set_fog_report_id(src.fog_report_id);
        result.set_attestation_evidence((&src.attestation_evidence).into());
        result.set_pubkey_expiry(src.pubkey_expiry);
        result
    }
}

impl From<report::ReportResponse> for mc_fog_report_types::ReportResponse {
    fn from(src: report::ReportResponse) -> Self {
        Self {
            // Note: this is out of order because get_chain is a borrow, but the
            //       iter below is a partial move.
            chain: src.get_chain().into(),
            reports: src.reports.into_iter().map(|x| x.into()).collect(),
            signature: src.signature,
        }
    }
}

impl From<mc_fog_report_types::ReportResponse> for report::ReportResponse {
    fn from(src: mc_fog_report_types::ReportResponse) -> Self {
        let mut result = report::ReportResponse::new();
        result.set_signature(src.signature);
        result.set_chain(src.chain.into());
        result.set_reports(src.reports.into_iter().map(|x| x.into()).collect());
        result
    }
}
