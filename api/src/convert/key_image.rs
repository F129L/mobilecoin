//! Convert to/from external::KeyImage.

use crate::{external, ConversionError};
use mc_transaction_core::ring_signature::KeyImage;

/// Convert KeyImage -->  external::KeyImage.
impl From<&KeyImage> for external::KeyImage {
    fn from(other: &KeyImage) -> Self {
        Self {
            data: other.as_bytes().to_vec(),
        }
    }
}

/// Convert external::KeyImage --> KeyImage.
impl TryFrom<&external::KeyImage> for KeyImage {
    type Error = ConversionError;

    fn try_from(source: &external::KeyImage) -> Result<Self, Self::Error> {
        Ok(KeyImage::try_from(&source.data[..])?)
    }
}

/* TODO: Remove this?
impl From<Vec<u8>> for external::KeyImage {
    fn from(data: Vec<u8>) -> Self {
        Self { data }
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // KeyImage --> external::KeyImage
    fn test_key_image_from() {
        let source: KeyImage = KeyImage::from(7);
        let converted = external::KeyImage::from(&source);
        assert_eq!(converted.data, source.as_bytes().to_vec());
    }

    #[test]
    // external::keyImage --> KeyImage
    fn test_key_image_try_from() {
        let source = external::KeyImage {
            data: KeyImage::from(11).as_bytes().to_vec(),
        };

        // try_from should succeed.
        let key_image = KeyImage::try_from(&source).unwrap();

        // key_image should have the correct value.
        assert_eq!(key_image, KeyImage::from(11));
    }

    #[test]
    // `KeyImage::try_from` should return ConversionError if the source contains the
    // wrong number of bytes.
    fn test_key_image_try_from_conversion_errors() {
        // Helper function asserts that a ConversionError::ArrayCastError is produced.
        fn expects_array_cast_error(bytes: &[u8]) {
            let source = external::KeyImage {
                data: bytes.to_vec(),
            };

            match KeyImage::try_from(&source).unwrap_err() {
                ConversionError::ArrayCastError => {} // Expected outcome.
                _ => panic!(),
            }
        }

        // Too many bytes should produce an ArrayCastError.
        expects_array_cast_error(&[11u8; 119]);

        // Too few bytes should produce an ArrayCastError.
        expects_array_cast_error(&[11u8; 3]);
    }
}
