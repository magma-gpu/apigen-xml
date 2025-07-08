// Copyright 2025 Google
// SPDX-License-Identifier: MIT

use std::fs::File;
use std::io::Write;

use minijinja::{context, Environment};

use crate::common::*;
use crate::common::utils::to_pascal_case;
use crate::generator::types::Writer;

pub struct DecoderWriter;

impl Writer for DecoderWriter {
    fn write(
        &self,
        api: &Api,
        gen_file: &GeneratedFile,
        output: &mut File,
    ) -> Result<(), ApiGenError> {
        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader("src/generator/templates"));
        env.add_filter("pascal_case", to_pascal_case);

        let tmpl = env.get_template("decoder/file.jinja")?;
        write!(
            output,
            "{}",
            tmpl.render(context! {
                year => api.copyright().year,
                holder => api.copyright().holder,
                spdx => api.copyright().spdx,
                generated_file => gen_file,
                api => api,
            })?
        )?;

        Ok(())
    }
}
