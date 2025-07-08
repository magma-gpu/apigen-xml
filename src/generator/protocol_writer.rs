// Copyright 2025 Google
// SPDX-License-Identifier: MIT

use std::fs::File;
use std::io::Write;

use minijinja::{context, Environment};


use crate::common::*;
use crate::generator::types::{FileType, Writer};
use crate::generator::utils::{calculate_type_sizes, to_pascal_case};



pub struct ProtocolWriter;

impl Writer for ProtocolWriter {
    fn write(&self, api: &Api, gen_file: &GeneratedFile, output: &mut File) -> Result<(), ApiGenError> {
        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader("src/generator/templates"));
        env.add_filter("pascal_case", to_pascal_case);

        let type_sizes = calculate_type_sizes(api, &FileType::Protocol);
        let tmpl = env.get_template("protocol.jinja")?;

        let defs: Vec<&Definition> = gen_file
            .instantiations
            .iter()
            .filter_map(|def_name| api.definitions.iter().find(|d| d.name == *def_name))
            .collect();

        write!(
            output,
            "{}",
            tmpl.render(context! {
                year => api.copyright.year,
                holder => api.copyright.holder,
                spdx => api.copyright.spdx,
                defs => defs,
                type_sizes => type_sizes,
            })?
        )?;
        Ok(())
    }
}
