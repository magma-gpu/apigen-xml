// Copyright 2025 Google
// SPDX-License-Identifier: MIT

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiGenError {
    #[error("IoError")]
    Io(std::io::Error),
    #[error("Xml parsing error")]
    Xml(xml::reader::Error),
    #[error("Integer parsing error")]
    ParseInt(std::num::ParseIntError),
    #[error("Missing attribute: {0}")]
    MissingAttribute(String),
    #[error("Formatting error")]
    Fmt(std::fmt::Error),
    #[error("Template error")]
    Template(minijinja::Error),
}

impl From<minijinja::Error> for ApiGenError {
    fn from(err: minijinja::Error) -> Self {
        ApiGenError::Template(err)
    }
}

impl From<std::io::Error> for ApiGenError {
    fn from(err: std::io::Error) -> Self {
        ApiGenError::Io(err)
    }
}

impl From<xml::reader::Error> for ApiGenError {
    fn from(err: xml::reader::Error) -> Self {
        ApiGenError::Xml(err)
    }
}

impl From<std::num::ParseIntError> for ApiGenError {
    fn from(err: std::num::ParseIntError) -> Self {
        ApiGenError::ParseInt(err)
    }
}

impl From<std::fmt::Error> for ApiGenError {
    fn from(err: std::fmt::Error) -> Self {
        ApiGenError::Fmt(err)
    }
}
