// Copyright 2025 Google
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use crate::common::*;
use crate::generator::types::FileType;

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

pub fn calculate_type_sizes(api: &Api, file_type: &FileType) -> HashMap<String, usize> {
    let mut type_sizes = HashMap::from([
        ("u8".to_string(), 1),
        ("u32".to_string(), 4),
        ("u64".to_string(), 8),
        ("usize".to_string(), 8), // Assuming 64-bit target
    ]);

    let constants_map: HashMap<String, usize> = api
        .definitions
        .iter()
        .flat_map(|def| &def.constants)
        .map(|c| (c.name.clone(), c.value.parse().unwrap_or(0)))
        .collect();

    let mut unresolved_structs: Vec<(&String, &Vec<Member>, bool)> = api
        .definitions
        .iter()
        .flat_map(|def| {
            def.structs
                .iter()
                .map(|s| (&s.name, &s.members, false))
                .chain(
                    def.extensible_structs
                        .iter()
                        .map(|s| (&s.name, &s.members, true)),
                )
        })
        .collect();

    while !unresolved_structs.is_empty() {
        let mut resolved_in_pass = false;

        unresolved_structs.retain(|(name, members, is_extensible)| {
            let mut struct_size = 0;
            let mut can_resolve = true;

            for member in *members {
                if let Some(size) = type_sizes.get(&member.type_name) {
                    struct_size += size;
                } else if member.type_name.starts_with('[') {
                    let parts: Vec<&str> = member
                        .type_name
                        .trim_matches(|c| c == '[' || c == ']')
                        .split(';')
                        .map(|p| p.trim())
                        .collect();
                    if parts.len() == 2 {
                        let base_type = parts[0];
                        let count_name = parts[1];
                        if let (Some(count), Some(base_type_size)) =
                            (constants_map.get(count_name), type_sizes.get(base_type))
                        {
                            struct_size += base_type_size * count;
                        } else {
                            can_resolve = false;
                            break;
                        }
                    } else {
                        can_resolve = false;
                        break;
                    }
                } else {
                    can_resolve = false;
                    break;
                }
            }

            if can_resolve {
                if *is_extensible {
                    if let FileType::Protocol = file_type {
                        struct_size += 8; // For structure_type and size fields
                        struct_size += (8 - (struct_size % 8)) % 8; // 8-byte alignment
                    }
                }
                type_sizes.insert((*name).clone(), struct_size);
                resolved_in_pass = true;
                false
            } else {
                true
            }
        });

        if !resolved_in_pass {
            break;
        }
    }

    type_sizes
}
