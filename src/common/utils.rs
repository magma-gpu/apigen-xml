// Copyright 2025 Google
// SPDX-License-Identifier: MIT

use crate::common::*;
use regex::Regex;
use std::collections::HashMap;

const NUM_BYTES_IN_U64: usize = 8;
const NUM_BYTES_IN_U32: usize = 4;

pub fn to_pascal_case(s: &str) -> String {
    let mut pascal = String::new();
    let mut capitalize = true;
    for c in s.chars() {
        if c == '_' {
            capitalize = true;
        } else if capitalize {
            pascal.push(c.to_ascii_uppercase());
            capitalize = false;
        } else {
            pascal.push(c);
        }
    }
    pascal
}

pub fn split(s: &str, p: &str) -> Vec<String> {
    s.split(p).map(|s| s.to_string()).collect()
}

fn calculate_member_size(
    members: &[Member],
    type_sizes: &HashMap<String, usize>,
    constants: &HashMap<String, Constant>,
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

            let c = constants
                .get(count_name)
                .ok_or_else(|| ApiGenError::ConstantNotFound(count_name.to_string()))?;

            let count =
                c.value
                    .parse::<usize>()
                    .map_err(|_| ApiGenError::InvalidConstantValue {
                        name: c.name.clone(),
                        value: c.value.clone(),
                    })?;
            size += base_type_size * count;
        } else {
            return Err(ApiGenError::TypeNotFound(member.type_name.clone()));
        }
    }
    Ok(size)
}

fn add_padding(members: &mut Vec<Member>, size: usize) {
    let padding = (NUM_BYTES_IN_U64 - (size % NUM_BYTES_IN_U64)) % NUM_BYTES_IN_U64;
    if padding == NUM_BYTES_IN_U32 {
        members.push(Member {
            type_name: format!("u32"),
            qualifier: String::new(),
            name: "padding".to_string(),
        });
    } else if padding > 0 {
        members.push(Member {
            type_name: format!("[u8; {}]", padding),
            qualifier: String::new(),
            name: "padding".to_string(),
        });
    }
}

pub fn calculate_all_sizes(api: &mut Api) -> Result<(), ApiGenError> {
    let mut type_sizes: HashMap<String, usize> = HashMap::from([
        ("u8".to_string(), 1),
        ("u32".to_string(), 4),
        ("u64".to_string(), 8),
        ("usize".to_string(), 8), // Assuming 64-bit target
    ]);

    let constants: HashMap<String, Constant> = api
        .definition_items
        .values()
        .filter_map(|item| {
            if let DefinitionItem::Constant(c) = item {
                Some((c.name.clone(), c.clone()))
            } else {
                None
            }
        })
        .collect();

    // We need to loop until all sizes are resolved, as extensible structs can depend on other
    // structs that haven't been processed yet.
    let mut changed = true;
    while changed {
        changed = false;
        // Pass 1: Structs
        for item in api.definition_items.values_mut() {
            if let DefinitionItem::Struct(s) = item {
                if !type_sizes.contains_key(&s.common.name) {
                    if let Ok(size) =
                        calculate_member_size(&s.common.members, &type_sizes, &constants)
                    {
                        type_sizes.insert(s.common.name.clone(), size);
                        changed = true;
                    }
                }
            }
        }

        // Pass 2: Extensible Structs
        for item in api.definition_items.values_mut() {
            if let DefinitionItem::ExtensibleStructs(es) = item {
                for s in &mut es.structs {
                    if !type_sizes.contains_key(&s.common.name) {
                        if let Ok(size) =
                            calculate_member_size(&s.common.members, &type_sizes, &constants)
                        {
                            let total_size = size + 4 + 8; // stype + pNext
                            type_sizes.insert(s.common.name.clone(), total_size);
                            changed = true;
                        }
                    }
                }
            }
        }
    }

    // Pass 3: Protocols
    for item in api.definition_items.values_mut() {
        if let DefinitionItem::Protocol(p) = item {
            for req in &mut p.requests {
                let size = calculate_member_size(&req.members, &type_sizes, &constants)?;
                add_padding(&mut req.members, size);
            }
            for res in &mut p.responses {
                let size = calculate_member_size(&res.members, &type_sizes, &constants)?;
                add_padding(&mut res.members, size);
            }
        }
    }

    Ok(())
}
