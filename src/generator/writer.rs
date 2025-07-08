// Copyright 2025 Google
// SPDX-License-Identifier: MIT

use std::fs::{create_dir_all, File};
use std::path::Path;

use crate::common::*;
use crate::generator::decoder_writer::DecoderWriter;
use crate::generator::encoder_writer::EncoderWriter;
use crate::generator::ffi_writer::FfiWriter;
use crate::generator::header_writer::HeaderWriter;
use crate::generator::protocol_writer::ProtocolWriter;
use crate::generator::rust_writer::RustWriter;
use crate::generator::types::{FileType, Writer};

pub fn generate_api(api: &Api, out_dir: &Path) -> Result<(), ApiGenError> {
    for gen_file in api.generated_files() {
        let full_out_path = out_dir.join(&gen_file.out_path);
        create_dir_all(&full_out_path)?;
        let output_path = full_out_path.join(&gen_file.file_name);
        let mut file = File::create(output_path)?;

        let writer: Box<dyn Writer> = match FileType::from_str(&gen_file.file_type) {
            Some(FileType::Protocol) => Box::new(ProtocolWriter),
            Some(FileType::Header) => Box::new(HeaderWriter),
            Some(FileType::Ffi) => Box::new(FfiWriter),
            Some(FileType::Rust) => Box::new(RustWriter),
            Some(FileType::Encoder) => Box::new(EncoderWriter),
            Some(FileType::Decoder) => Box::new(DecoderWriter),
            None => {
                // Handle unknown file type
                continue;
            }
        };

        writer.write(api, gen_file, &mut file)?;
    }

    Ok(())
}
