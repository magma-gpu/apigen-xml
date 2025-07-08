// Copyright 2025 Google
// SPDX-License-Identifier: MIT

use regex::Regex;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use xml::attribute::OwnedAttribute;
use xml::reader::{EventReader, XmlEvent};

use crate::common::utils::to_pascal_case;
use crate::common::*;

/// Helper to find a specific attribute's value from a list of attributes.
fn find_attribute_value(attributes: &[OwnedAttribute], name: &str) -> Option<String> {
    attributes
        .iter()
        .find(|attr| attr.name.local_name == name)
        .map(|attr| attr.value.clone())
}

/// Helper to read the character data between a start and end tag.
fn read_text_content<R: std::io::Read>(parser: &mut EventReader<R>) -> Result<String, ApiGenError> {
    let next_event = parser.next()?;
    if let XmlEvent::Characters(text) = next_event {
        parser.next()?; // Consume the closing EndElement tag.
        Ok(text.trim().to_string())
    } else {
        Ok(String::new())
    }
}

/// Parses a single <constant> element.
fn parse_constant<R: std::io::Read>(
    parser: &mut EventReader<R>,
    api: &mut Api,
    def: &mut Definition,
) -> Result<(), ApiGenError> {
    let mut constant = Constant::default();
    // This is a hack, constant doesn't have a name field, but we need to parse it to get the
    // item name.
    let mut item_name = String::default();
    loop {
        match parser.next()? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "type" => constant.type_name = read_text_content(parser)?,
                "item" => {
                    item_name = find_attribute_value(&attributes, "name").ok_or_else(|| {
                        ApiGenError::MissingAttribute("<item> missing 'name'".to_string())
                    })?;
                    constant.value =
                        find_attribute_value(&attributes, "value").ok_or_else(|| {
                            ApiGenError::MissingAttribute("<item> missing 'value'".to_string())
                        })?;
                }
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "constant" => break,
            _ => {}
        }
    }
    constant.name = item_name.clone();
    def.items.push(item_name.clone());
    api.definition_items
        .insert(item_name, DefinitionItem::Constant(constant));
    Ok(())
}

/// Parses a single <member> element.
fn parse_member<R: std::io::Read>(parser: &mut EventReader<R>) -> Result<Member, ApiGenError> {
    let mut member = Member::default();
    loop {
        match parser.next()? {
            XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                "type" => member.type_name = read_text_content(parser)?,
                "qualifier" => member.qualifier = read_text_content(parser)?,
                "name" => member.name = read_text_content(parser)?,
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "member" => break,
            _ => {}
        }
    }
    Ok(member)
}

/// Parses a <struct> element.
fn parse_struct<R: std::io::Read>(
    parser: &mut EventReader<R>,
    api: &mut Api,
    def: &mut Definition,
) -> Result<(), ApiGenError> {
    let mut struct_def = StructDef::default();
    loop {
        match parser.next()? {
            XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                "name" => struct_def.common.name = read_text_content(parser)?,
                "member" => struct_def.common.members.push(parse_member(parser)?),
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "struct" => break,
            _ => {}
        }
    }

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
    def.items.push(struct_def.common.name.clone());
    api.definition_items.insert(
        struct_def.common.name.clone(),
        DefinitionItem::Struct(struct_def),
    );
    Ok(())
}

/// Parses a single <request> element.
fn parse_request<R: std::io::Read>(parser: &mut EventReader<R>) -> Result<Request, ApiGenError> {
    let mut request = Request::default();
    loop {
        match parser.next()? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "opcode" => {
                    request.opcode.name =
                        find_attribute_value(&attributes, "name").ok_or_else(|| {
                            ApiGenError::MissingAttribute("<opcode> missing 'name'".to_string())
                        })?;
                    request.opcode.value =
                        find_attribute_value(&attributes, "value").ok_or_else(|| {
                            ApiGenError::MissingAttribute("<opcode> missing 'value'".to_string())
                        })?;
                }
                "member" => request.members.push(parse_member(parser)?),
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "request" => break,
            _ => {}
        }
    }
    Ok(request)
}

/// Parses a single <response> element.
fn parse_response<R: std::io::Read>(parser: &mut EventReader<R>) -> Result<Response, ApiGenError> {
    let mut response = Response::default();
    loop {
        match parser.next()? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "opcode" => {
                    response.opcode.name =
                        find_attribute_value(&attributes, "name").ok_or_else(|| {
                            ApiGenError::MissingAttribute("<opcode> missing 'name'".to_string())
                        })?;
                    response.opcode.value =
                        find_attribute_value(&attributes, "value").ok_or_else(|| {
                            ApiGenError::MissingAttribute("<opcode> missing 'value'".to_string())
                        })?;
                }
                "member" => response.members.push(parse_member(parser)?),
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "response" => break,
            _ => {}
        }
    }
    Ok(response)
}

/// Parses a single <enum> block.
fn parse_enum<R: std::io::Read>(
    parser: &mut EventReader<R>,
    api: &mut Api,
    def: &mut Definition,
) -> Result<(), ApiGenError> {
    let mut new_enum = Enum::default();
    loop {
        match parser.next()? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "enum_name" => new_enum.name = read_text_content(parser)?,
                "type" => new_enum.type_name = read_text_content(parser)?,
                "item" => {
                    let name = find_attribute_value(&attributes, "name").ok_or_else(|| {
                        ApiGenError::MissingAttribute("Enum <item> missing 'name'".to_string())
                    })?;
                    let value = find_attribute_value(&attributes, "value").ok_or_else(|| {
                        ApiGenError::MissingAttribute("Enum <item> missing 'value'".to_string())
                    })?;
                    new_enum.entries.push(EnumEntry { name, value });
                }
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "enum" => break,
            _ => {}
        }
    }
    def.items.push(new_enum.name.clone());
    api.definition_items
        .insert(new_enum.name.clone(), DefinitionItem::Enum(new_enum));
    Ok(())
}

/// Parses a single <flag> block.
fn parse_flag<R: std::io::Read>(
    parser: &mut EventReader<R>,
    api: &mut Api,
    def: &mut Definition,
) -> Result<(), ApiGenError> {
    let mut new_flag = Flag::default();
    loop {
        match parser.next()? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "flag_name" => new_flag.name = read_text_content(parser)?,
                "type" => new_flag.type_name = read_text_content(parser)?,
                "item" => {
                    let name = find_attribute_value(&attributes, "name").ok_or_else(|| {
                        ApiGenError::MissingAttribute("Flag <item> missing 'name'".to_string())
                    })?;
                    let value = find_attribute_value(&attributes, "value").ok_or_else(|| {
                        ApiGenError::MissingAttribute("Flag <item> missing 'value'".to_string())
                    })?;
                    new_flag.entries.push(EnumEntry { name, value });
                }
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "flag" => break,
            _ => {}
        }
    }
    def.items.push(new_flag.name.clone());
    api.definition_items
        .insert(new_flag.name.clone(), DefinitionItem::Flag(new_flag));
    Ok(())
}

/// Parses the <copyright> block.
fn parse_copyright<R: std::io::Read>(
    parser: &mut EventReader<R>,
) -> Result<Copyright, ApiGenError> {
    let mut copyright = Copyright::default();
    loop {
        match parser.next()? {
            XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                "spdx" => copyright.spdx = read_text_content(parser)?,
                "holder" => copyright.holder = read_text_content(parser)?,
                "year" => copyright.year = read_text_content(parser)?.parse()?,
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "copyright" => break,
            _ => {}
        }
    }
    Ok(copyright)
}

/// Parses an <extensible_struct> element.
fn parse_extensible_struct<R: std::io::Read>(
    parser: &mut EventReader<R>,
) -> Result<ExtensibleStruct, ApiGenError> {
    let mut struct_def = ExtensibleStruct::default();
    loop {
        match parser.next()? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "name" => struct_def.common.name = read_text_content(parser)?,
                "stype" => {
                    struct_def.stype.name =
                        find_attribute_value(&attributes, "name").ok_or_else(|| {
                            ApiGenError::MissingAttribute("<stype> missing 'name'".to_string())
                        })?;
                    struct_def.stype.value = find_attribute_value(&attributes, "value")
                        .ok_or_else(|| {
                            ApiGenError::MissingAttribute("<stype> missing 'value'".to_string())
                        })?;
                }
                "member" => struct_def.common.members.push(parse_member(parser)?),
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "extensible_struct" => break,
            _ => {}
        }
    }
    Ok(struct_def)
}

/// Parses an <extensible_structs> element.
fn parse_extensible_structs<R: std::io::Read>(
    parser: &mut EventReader<R>,
    api: &mut Api,
    def: &mut Definition,
) -> Result<(), ApiGenError> {
    let mut stypes_name = String::new();
    let mut parsed_structs: Vec<ExtensibleStruct> = Vec::new();

    loop {
        match parser.next()? {
            XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                "stypes" => stypes_name = read_text_content(parser)?,
                "extensible_struct" => {
                    parsed_structs.push(parse_extensible_struct(parser)?);
                }
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "extensible_structs" => break,
            _ => {}
        }
    }

    // Create and add the protocol struct for the container.
    let protocol_struct_name = format!("{}Protocol", to_pascal_case(&stypes_name));
    let protocol_struct = StructCommon {
        name: protocol_struct_name.clone(),
        members: vec![Member {
            type_name: stypes_name.clone(),
            qualifier: String::new(),
            name: "stype".to_string(),
        }],
        ..Default::default()
    };
    def.items.push(protocol_struct_name.clone());
    api.definition_items.insert(
        protocol_struct_name,
        DefinitionItem::Struct(StructDef {
            common: protocol_struct.clone(),
        }),
    );

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
    def.items.push(ffi_struct_name.clone());
    api.definition_items.insert(
        ffi_struct_name,
        DefinitionItem::Struct(StructDef {
            common: ffi_struct.clone(),
        }),
    );

    let mut stype_enum = Enum {
        name: stypes_name.clone(),
        type_name: "u32".to_string(),
        ..Default::default()
    };

    // Add the individual extensible structs as struct definitions and collect stypes.
    for s in &parsed_structs {
        def.items.push(s.common.name.clone());
        api.definition_items.insert(
            s.common.name.clone(),
            DefinitionItem::Struct(s.clone().into()),
        );
        stype_enum.entries.push(s.stype.clone().into());
    }

    let structs = ExtensibleStructs {
        stypes_name: stypes_name.clone(),
        structs: parsed_structs,
        protocol_struct,
        ffi_struct,
    };

    def.items.push(structs.stypes_name.clone());
    api.definition_items.insert(
        structs.stypes_name.clone(),
        DefinitionItem::ExtensibleStructs(structs),
    );

    def.items.push(stype_enum.name.clone());
    api.definition_items
        .insert(stype_enum.name.clone(), DefinitionItem::Enum(stype_enum));

    Ok(())
}

/// Parses an <object> element.
fn parse_object<R: std::io::Read>(
    parser: &mut EventReader<R>,
    api: &mut Api,
    def: &mut Definition,
) -> Result<(), ApiGenError> {
    let mut object = Object::default();
    loop {
        match parser.next()? {
            XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                "ffi" => object.ffi = read_text_content(parser)?,
                "rust" => object.rust = read_text_content(parser)?,
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "object" => break,
            _ => {}
        }
    }
    // Hack, object doesn't have a name.
    object.name = object.ffi.clone();
    def.items.push(object.name.clone());
    api.definition_items
        .insert(object.name.clone(), DefinitionItem::Object(object));
    Ok(())
}

/// Parses a <function> element.
fn parse_function<R: std::io::Read>(
    parser: &mut EventReader<R>,
    api: &mut Api,
    def: &mut Definition,
) -> Result<(), ApiGenError> {
    let mut function = Function::default();
    loop {
        match parser.next()? {
            XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                "name" => function.name = read_text_content(parser)?,
                "return" => function.ret = read_text_content(parser)?,
                "member" => function.members.push(parse_member(parser)?),
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "function" => break,
            _ => {}
        }
    }
    def.items.push(function.name.clone());
    api.definition_items
        .insert(function.name.clone(), DefinitionItem::Function(function));
    Ok(())
}

/// Parses a <protocol> element.
fn parse_protocol<R: std::io::Read>(
    parser: &mut EventReader<R>,
    api: &mut Api,
    def: &mut Definition,
) -> Result<(), ApiGenError> {
    let mut protocol = Protocol::default();
    loop {
        match parser.next()? {
            XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                "protocol_name" => protocol.name = read_text_content(parser)?,
                "request" => protocol.requests.push(parse_request(parser)?),
                "response" => protocol.responses.push(parse_response(parser)?),
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "protocol" => break,
            _ => {}
        }
    }

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
    api.definition_items.insert(
        protocol_struct_name.clone(),
        DefinitionItem::Struct(protocol_struct),
    );
    def.items.push(protocol_struct_name.clone());

    // Prepend the header member to all requests and responses.
    let header_member = Member {
        type_name: protocol_struct_name,
        qualifier: String::new(),
        name: "hdr".to_string(),
    };

    for req in &mut protocol.requests {
        req.members.insert(0, header_member.clone());
    }
    for res in &mut protocol.responses {
        res.members.insert(0, header_member.clone());
    }

    def.items.push(protocol.name.clone());
    api.definition_items
        .insert(protocol.name.clone(), DefinitionItem::Protocol(protocol));
    Ok(())
}

/// Parses a <define> block and populates the api.
fn parse_define<R: std::io::Read>(
    parser: &mut EventReader<R>,
    api: &mut Api,
) -> Result<(), ApiGenError> {
    let mut def = Definition::default();
    loop {
        match parser.next()? {
            XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                "name" => def.name = read_text_content(parser)?,
                "enum" => parse_enum(parser, api, &mut def)?,
                "flags" => {
                    while let Some(_) =
                        parse_block_item(parser, "flags", "flag", |p| parse_flag(p, api, &mut def))?
                    {
                    }
                }
                "constants" => {
                    while let Some(_) = parse_block_item(parser, "constants", "constant", |p| {
                        parse_constant(p, api, &mut def)
                    })? {}
                }
                "structs" => {
                    while let Some(_) = parse_block_item(parser, "structs", "struct", |p| {
                        parse_struct(p, api, &mut def)
                    })? {}
                }
                "extensible_structs" => parse_extensible_structs(parser, api, &mut def)?,
                "objects" => {
                    while let Some(_) = parse_block_item(parser, "objects", "object", |p| {
                        parse_object(p, api, &mut def)
                    })? {}
                }
                "function" => parse_function(parser, api, &mut def)?,
                "protocol" => parse_protocol(parser, api, &mut def)?,
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "define" => break,
            _ => {}
        }
    }
    api.definitions.insert(def.name.clone(), def);
    Ok(())
}

/// Parses a <generated_file> block.
fn parse_generated_file<R: std::io::Read>(
    parser: &mut EventReader<R>,
) -> Result<GeneratedFile, ApiGenError> {
    let mut gen_file = GeneratedFile::default();
    loop {
        match parser.next()? {
            XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                "out_path" => gen_file.out_path = read_text_content(parser)?,
                "file_name" => gen_file.file_name = read_text_content(parser)?,
                "file_type" => gen_file.file_type = read_text_content(parser)?,
                "include" => gen_file.includes.push(read_text_content(parser)?),
                "instantiate" => gen_file.instantiations.push(read_text_content(parser)?),
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "generated_file" => break,
            _ => {}
        }
    }
    Ok(gen_file)
}

/// Parses the entire <api> block.
fn parse_api_internal<R: std::io::Read>(parser: &mut EventReader<R>) -> Result<Api, ApiGenError> {
    let mut api = Api::default();

    loop {
        match parser.next()? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "api" => {
                    api.name = find_attribute_value(&attributes, "name")
                        .unwrap_or_else(|| "unknown".to_string());
                }
                "copyright" => api.copyright = parse_copyright(parser)?,
                "version" => api.version = read_text_content(parser)?.parse()?,
                "define" => parse_define(parser, &mut api)?,
                "generated_file" => api.generated_files.push(parse_generated_file(parser)?),
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "api" => break,
            XmlEvent::EndDocument => break,
            _ => {}
        }
    }
    Ok(api)
}

/// Generic helper to parse items within a block.
fn parse_block_item<R, T, F>(
    parser: &mut EventReader<R>,
    block_name: &str,
    item_name: &str,
    mut parse_fn: F,
) -> Result<Option<T>, ApiGenError>
where
    R: std::io::Read,
    F: FnMut(&mut EventReader<R>) -> Result<T, ApiGenError>,
{
    match parser.next()? {
        XmlEvent::StartElement { name, .. } if name.local_name == item_name => {
            Ok(Some(parse_fn(parser)?))
        }
        XmlEvent::EndElement { name } if name.local_name == block_name => Ok(None), // Sentinel
        _ => Ok(None),
    }
}

pub fn parse_api(filename: &Path) -> Result<Api, ApiGenError> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut parser = EventReader::new(reader);
    parse_api_internal(&mut parser)
}
