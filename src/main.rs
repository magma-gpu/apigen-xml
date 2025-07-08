// Copyright 2025 Google
// SPDX-License-Identifier: MIT

use std::fs;
use std::path::PathBuf;

use clap::Parser;

mod common;
mod generator;
mod parser;

#[allow(dead_code)]
mod generated_protocols;

use common::ApiGenError;

#[derive(Parser, Debug)]
#[command(version, about = None, long_about = None)]
struct Args {
    /// The filename used to generate Rust protocol
    #[arg(long)]
    filename: PathBuf,

    /// The output directory for the generated Rust files
    #[arg(long)]
    out_dir: PathBuf,
}

fn main() -> Result<(), ApiGenError> {
    let args = Args::parse();
    fs::create_dir_all(&args.out_dir)?;
    let api_data = parser::parse_api(&args.filename)?;
    generator::generate_api(&api_data, &args.out_dir)?;
    Ok(())
}
