// Copyright 2025 Google
// SPDX-License-Identifier: MIT

use crate::common::utils::to_pascal_case;
use crate::common::*;
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;

const NUM_BYTES_IN_U64: usize = 8;
const NUM_BYTES_IN_U32: usize = 4;

#[derive(Debug, Default, Serialize, Clone)]
pub struct Api {
    name: String,
    copyright: Copyright,
    version: u32,
    definitions: HashMap<String, Definition>,
    definition_items: HashMap<String, DefinitionItem>,
    type_sizes: HashMap<String, usize>,
    rust_to_c_typemap: HashMap<String, String>,
    generated_files: Vec<GeneratedFile>,
}

// Free functions that were causing borrow checker issues as methods.
fn calculate_member_size(
    members: &[Member],
    type_sizes: &HashMap<String, usize>,
) -> Result<usize, ApiGenError> {
    let mut size = 0;
    for member in members {
        if let Some(s) = type_sizes.get(&member.type_name) {
            size += s;
        } else if member.type_name.starts_with('[') {
            let re = Regex::new(r"\[([^;]+);\s*([^\]]+)\]").unwrap();
            let caps = re
                .captures(&member.type_name)
                .ok_or_else(|| ApiGenError::InvalidArrayTypeFormat(member.type_name.clone()))?;
            let base_type = caps.get(1).unwrap().as_str();
            let count_name = caps.get(2).unwrap().as_str();

            let base_type_size = type_sizes
                .get(base_type)
                .ok_or_else(|| ApiGenError::TypeNotFound(base_type.to_string()))?;

            let count = type_sizes
                .get(count_name)
                .ok_or_else(|| ApiGenError::ConstantNotFound(count_name.to_string()))?;
            size += base_type_size * count;
        } else {
            return Err(ApiGenError::TypeNotFound(member.type_name.clone()));
        }
    }
    Ok(size)
}

fn calculate_padding(size: usize) -> Option<Member> {
    let padding = (NUM_BYTES_IN_U64 - (size % NUM_BYTES_IN_U64)) % NUM_BYTES_IN_U64;
    if padding == NUM_BYTES_IN_U32 {
        Some(Member {
            type_name: format!("u32"),
            qualifier: String::new(),
            name: "padding".to_string(),
        })
    } else if padding > 0 {
        Some(Member {
            type_name: format!("[u8; {}]", padding),
            qualifier: String::new(),
            name: "padding".to_string(),
        })
    } else {
        None
    }
}

impl Api {
    pub fn new() -> Self {
        let type_sizes: HashMap<String, usize> = HashMap::from([
            ("u8".to_string(), 1),
            ("i8".to_string(), 1),
            ("u16".to_string(), 2),
            ("i16".to_string(), 2),
            ("i32".to_string(), 4),
            ("u32".to_string(), 4),
            ("u64".to_string(), 8),
            ("i64".to_string(), 8),
            ("f64".to_string(), 8),
            ("usize".to_string(), 8), // Assuming 64-bit target
            ("*mut std::ffi::c_void".to_string(), 8),
        ]);
        let rust_to_c_typemap: HashMap<String, String> = HashMap::from([
            ("u8".to_string(), "uint8_t".to_string()),
            ("i8".to_string(), "int8_t".to_string()),
            ("u16".to_string(), "uint16_t".to_string()),
            ("i16".to_string(), "int16_t".to_string()),
            ("i32".to_string(), "int32_t".to_string()),
            ("u32".to_string(), "uint32_t".to_string()),
            ("u64".to_string(), "uint64_t".to_string()),
            ("i64".to_string(), "int64_t".to_string()),
            ("f64".to_string(), "double".to_string()),
            ("usize".to_string(), "size_t".to_string()),
            ("*mut std::ffi::c_void".to_string(), "void*".to_string()),
        ]);
        Api {
            type_sizes,
            rust_to_c_typemap,
            ..Default::default()
        }
    }

    // Getters
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn copyright(&self) -> &Copyright {
        &self.copyright
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn definitions(&self) -> &HashMap<String, Definition> {
        &self.definitions
    }

    pub fn definition_items(&self) -> &HashMap<String, DefinitionItem> {
        &self.definition_items
    }

    pub fn generated_files(&self) -> &[GeneratedFile] {
        &self.generated_files
    }

    // Setters/mutators for parser
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_copyright(&mut self, copyright: Copyright) {
        self.copyright = copyright;
    }

    pub fn set_version(&mut self, version: u32) {
        self.version = version;
    }

    pub fn add_generated_file(&mut self, file: GeneratedFile) {
        self.generated_files.push(file);
    }

    pub fn add_definition(&mut self, definition: Definition) {
        self.definitions.insert(definition.name.clone(), definition);
    }

    // Methods with logic moved from parser
    pub fn add_constant(&mut self, constant: Constant) -> Result<(), ApiGenError> {
        let item_name = constant.name.clone();
        let value =
            constant
                .value
                .parse::<usize>()
                .map_err(|_| ApiGenError::InvalidConstantValue {
                    name: constant.name.clone(),
                    value: constant.value.clone(),
                })?;
        self.type_sizes.insert(item_name.clone(), value);
        self.definition_items
            .insert(item_name.clone(), DefinitionItem::Constant(constant));
        Ok(())
    }

    pub fn add_struct(&mut self, mut struct_def: StructDef) -> Result<(), ApiGenError> {
        // Post-process to find array and count members.
        let re = Regex::new(r"\[([^;]+);\s*([^\]]+)\]").unwrap();
        for member in &struct_def.common.members {
            if let Some(caps) = re.captures(&member.type_name) {
                let base_type = caps.get(1).unwrap().as_str().to_string();
                let array_member_name = member.name.clone();

                // Find the corresponding count member. Convention is singular name + "Count".
                let singular_name = array_member_name
                    .strip_suffix('s')
                    .unwrap_or(&array_member_name);
                let count_member_name = format!("{}_count", singular_name);

                if struct_def
                    .common
                    .members
                    .iter()
                    .any(|m| m.name == count_member_name)
                {
                    struct_def.common.array_info.push(ArrayInfo {
                        array_member_name,
                        array_base_type: base_type,
                        count_member_name,
                    });
                }
            }
        }
        let item_name = struct_def.common.name.clone();
        let size = calculate_member_size(&struct_def.common.members, &self.type_sizes)?;
        self.type_sizes.insert(item_name.clone(), size);
        self.definition_items
            .insert(item_name.clone(), DefinitionItem::Struct(struct_def));
        Ok(())
    }

    pub fn add_enum(&mut self, new_enum: Enum) -> Result<(), ApiGenError> {
        let item_name = new_enum.name.clone();
        let size = self
            .type_sizes
            .get(&new_enum.type_name)
            .ok_or_else(|| ApiGenError::TypeNotFound(new_enum.type_name.clone()))?;
        self.type_sizes.insert(item_name.clone(), *size);
        self.definition_items
            .insert(item_name.clone(), DefinitionItem::Enum(new_enum));
        Ok(())
    }

    pub fn add_flag(&mut self, new_flag: Flag) -> Result<(), ApiGenError> {
        let item_name = new_flag.name.clone();
        let size = self
            .type_sizes
            .get(&new_flag.type_name)
            .ok_or_else(|| ApiGenError::TypeNotFound(new_flag.type_name.clone()))?;
        self.type_sizes.insert(item_name.clone(), *size);
        self.definition_items
            .insert(item_name.clone(), DefinitionItem::Flag(new_flag));
        Ok(())
    }

    pub fn add_object(&mut self, mut object: Object) {
        // Hack, object doesn't have a name.
        object.name = object.ffi.clone();
        let item_name = object.name.clone();
        self.definition_items
            .insert(item_name.clone(), DefinitionItem::Object(object));
    }

    pub fn add_function(&mut self, function: Function) {
        let item_name = function.name.clone();
        self.definition_items
            .insert(item_name.clone(), DefinitionItem::Function(function));
    }

    pub fn add_protocol(&mut self, mut protocol: Protocol) -> Result<(), ApiGenError> {
        // Create and add the protocol header struct.
        let protocol_struct_name = format!("{}CommandHdr", to_pascal_case(&protocol.name));
        protocol.protocol_struct_name = protocol_struct_name.clone();

        let protocol_struct = StructDef {
            common: StructCommon {
                name: protocol_struct_name.clone(),
                members: vec![
                    Member {
                        type_name: "u32".to_string(),
                        qualifier: String::new(),
                        name: "proto".to_string(),
                    },
                    Member {
                        type_name: "u32".to_string(),
                        qualifier: String::new(),
                        name: "size".to_string(),
                    },
                ],
                ..Default::default()
            },
        };
        let size = calculate_member_size(&protocol_struct.common.members, &self.type_sizes)?;
        self.type_sizes.insert(protocol_struct_name.clone(), size);
        self.definition_items.insert(
            protocol_struct_name.clone(),
            DefinitionItem::Struct(protocol_struct),
        );

        // Prepend the header member to all requests and responses.
        let header_member = Member {
            type_name: protocol_struct_name,
            qualifier: String::new(),
            name: "hdr".to_string(),
        };

        for req in &mut protocol.requests {
            req.members.insert(0, header_member.clone());
            let size = calculate_member_size(&req.members, &self.type_sizes)?;
            if let Some(padding) = calculate_padding(size) {
                req.members.push(padding);
            }
        }
        for res in &mut protocol.responses {
            res.members.insert(0, header_member.clone());
            let size = calculate_member_size(&res.members, &self.type_sizes)?;
            if let Some(padding) = calculate_padding(size) {
                res.members.push(padding);
            }
        }

        let item_name = protocol.name.clone();
        self.definition_items
            .insert(item_name.clone(), DefinitionItem::Protocol(protocol));
        Ok(())
    }

    pub fn add_extensible_structs(
        &mut self,
        stypes_name: String,
        mut parsed_structs: Vec<ExtensibleStruct>,
    ) -> Result<(), ApiGenError> {
        let mut stype_enum = Enum {
            name: stypes_name.clone(),
            type_name: "u32".to_string(),
            ..Default::default()
        };

        // Add the individual extensible structs as struct definitions and collect stypes.
        for s in &mut parsed_structs {
            stype_enum.entries.push(s.stype.clone().into());
        }

        self.add_enum(stype_enum)?;

        // Create and add the protocol struct for the container.
        let protocol_struct_name = format!("{}Hdr", to_pascal_case(&stypes_name));
        let protocol_struct = StructCommon {
            name: protocol_struct_name.clone(),
            members: vec![
                Member {
                    type_name: stypes_name.clone(),
                    qualifier: String::new(),
                    name: "stype".to_string(),
                },
                Member {
                    type_name: "u32".to_string(),
                    qualifier: String::new(),
                    name: "size".to_string(),
                },
            ],
            ..Default::default()
        };

        let protocol_struct_size =
            calculate_member_size(&protocol_struct.members, &self.type_sizes)?;

        // Create and add the FFI struct for the container.
        let ffi_struct_name = format!("{}FFI", to_pascal_case(&stypes_name));
        let ffi_struct = StructCommon {
            name: ffi_struct_name.clone(),
            members: vec![
                Member {
                    type_name: stypes_name.clone(),
                    qualifier: String::new(),
                    name: "stype".to_string(),
                },
                Member {
                    type_name: "*mut std::ffi::c_void".to_string(),
                    qualifier: String::new(),
                    name: "pNext".to_string(),
                },
            ],
            ..Default::default()
        };

        // Add the individual extensible structs as struct definitions and collect stypes.
        for s in &mut parsed_structs {
            let item_name = s.common.name.clone();
            let size = calculate_member_size(&s.common.members, &self.type_sizes)?;
            let total_size = size + protocol_struct_size;
            self.type_sizes.insert(item_name.clone(), total_size);
            s.padding = calculate_padding(total_size);
            self.definition_items.insert(
                item_name.clone(),
                DefinitionItem::ExtensibleStruct(s.clone()),
            );
        }

        let structs = ExtensibleStructs {
            stypes_name: stypes_name.clone(),
            structs: parsed_structs,
            protocol_struct,
            ffi_struct,
        };

        let item_name = structs.stypes_name.clone();
        self.definition_items.insert(
            item_name.clone(),
            DefinitionItem::ExtensibleStructs(structs),
        );
        Ok(())
    }
}
