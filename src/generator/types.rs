// Copyright 2025 Google
// SPDX-License-Identifier: MIT

use std::fs::File;

use crate::common::*;

pub enum FileType {
    Protocol,
    Header,
    Ffi,
    Rust,
    Encoder,
    Decoder,
}

impl FileType {
    pub fn from_str(s: &str) -> Option<FileType> {
        match s {
            "protocol" => Some(FileType::Protocol),
            "header" => Some(FileType::Header),
            "ffi" => Some(FileType::Ffi),
            "Rust" => Some(FileType::Rust),
            "encoder" => Some(FileType::Encoder),
            "decoder" => Some(FileType::Decoder),
            _ => None,
        }
    }
}

pub trait Writer {
    fn write(&self, api: &Api, gen_file: &GeneratedFile, out: &mut File)
        -> Result<(), ApiGenError>;
}
