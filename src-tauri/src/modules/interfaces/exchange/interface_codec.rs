use crate::modules::interfaces::document::DocumentFormat;
use crate::modules::interfaces::document::InterfaceDocument;
use crate::modules::interfaces::exchange::ImportPreview;
use base64::Engine;
use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use serde_json::Value;
use std::io::{Read, Write};

pub(crate) struct InterfaceCodec;

impl InterfaceCodec {
    pub(crate) fn decode(text: &str) -> Result<ImportPreview, String> {
        if text.len() > 1_000_000 {
            return Err("Import string is too large".into());
        }

        let text = text.trim();

        let value: Value = if let Some(encoded) = text.strip_prefix(DocumentFormat::PREFIX) {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .map_err(|_| "Invalid import string")?;

            let decoder = DeflateDecoder::new(bytes.as_slice());

            let mut output = Vec::new();

            decoder
                .take(4_000_001)
                .read_to_end(&mut output)
                .map_err(|_| "Could not decompress string")?;

            if output.len() > 4_000_000 {
                return Err("Decompressed interface is too large".into());
            }

            serde_json::from_slice(&output).map_err(|_| "Invalid interface JSON")?
        } else {
            serde_json::from_str(text)
                .map_err(|_| "An !AURION string or interface JSON is required")?
        };

        let document: InterfaceDocument = serde_json::from_value(value).map_err(|error| {
            if DocumentFormat::unsupported(&error) {
                format!("Profile uses unsupported fields or values; a newer Aurion may be required: {error}")
            } else {
                format!("Unsupported structure: {error}")
            }
        })?;

        Ok(ImportPreview { document })
    }

    pub(crate) fn encode(document: &InterfaceDocument) -> Result<String, String> {
        let bytes = serde_json::to_vec(document).map_err(|error| error.to_string())?;

        if bytes.len() > 4_000_000 {
            return Err("Interface is too large to export".into());
        }

        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());

        encoder
            .write_all(&bytes)
            .map_err(|error| error.to_string())?;
        let encoded = base64::engine::general_purpose::STANDARD
            .encode(encoder.finish().map_err(|error| error.to_string())?);

        let text = format!("{}{encoded}", DocumentFormat::PREFIX);

        if text.len() > 1_000_000 {
            return Err("Export string exceeds 1 MB".into());
        }

        Ok(text)
    }
}
