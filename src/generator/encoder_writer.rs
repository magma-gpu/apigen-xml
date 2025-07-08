// Copyright 2025 Google
// SPDX-License-Identifier: MIT

use std::fs::File;
use std::io::Write;

use minijinja::{context, Environment};

use crate::common::*;
use crate::generator::types::Writer;

pub struct EncoderWriter;

impl Writer for EncoderWriter {
    fn write(
        &self,
        api: &Api,
        gen_file: &GeneratedFile,
        output: &mut File,
    ) -> Result<(), ApiGenError> {
        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader("src/generator/templates"));

        let tmpl = env.get_template("encoder/file.jinja")?;
        let defs: Vec<&DefinitionItem> = gen_file
            .instantiations
            .iter()
            .filter_map(|def_name| {
                api.definitions().get(def_name).map(|def| {
                    def.items
                        .iter()
                        .filter_map(|item_name| api.definition_items().get(item_name))
                })
            })
            .flatten()
            .collect();
        write!(
            output,
            "{}",
            tmpl.render(context! {
                year => api.copyright().year,
                holder => api.copyright().holder,
                spdx => api.copyright().spdx,
                defs => defs,
                gen_file => gen_file,
            })?
        )?;

        Ok(())
    }
}
