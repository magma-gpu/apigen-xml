// Copyright 2025 Google
// SPDX-License-Identifier: MIT

mod ffi_writer;
mod header_writer;
mod protocol_writer;
mod rust_writer;
mod types;

mod writer;

pub use writer::generate_api;
