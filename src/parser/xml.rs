// Copyright 2025 Google
// SPDX-License-Identifier: MIT

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
fn parse_constant<R: std::io::Read>(parser: &mut EventReader<R>) -> Result<Constant, ApiGenError> {
    let mut constant = Constant::default();
    loop {
        match parser.next()? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "type" => constant.type_name = read_text_content(parser)?,
                "item" => {
                    constant.name = find_attribute_value(&attributes, "name").ok_or_else(|| {
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
    Ok(constant)
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
fn parse_struct<R: std::io::Read>(parser: &mut EventReader<R>) -> Result<StructDef, ApiGenError> {
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
    Ok(struct_def)
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
fn parse_enum<R: std::io::Read>(parser: &mut EventReader<R>) -> Result<Enum, ApiGenError> {
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
    Ok(new_enum)
}

/// Parses a single <flag> block.
fn parse_flag<R: std::io::Read>(parser: &mut EventReader<R>) -> Result<Flag, ApiGenError> {
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
    Ok(new_flag)
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
) -> Result<(String, Vec<ExtensibleStruct>), ApiGenError> {
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
    Ok((stypes_name, parsed_structs))
}

/// Parses an <object> element.
fn parse_object<R: std::io::Read>(parser: &mut EventReader<R>) -> Result<Object, ApiGenError> {
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
    Ok(object)
}

/// Parses a <function> element.
fn parse_function<R: std::io::Read>(parser: &mut EventReader<R>) -> Result<Function, ApiGenError> {
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
    Ok(function)
}

/// Parses a <protocol> element.
fn parse_protocol<R: std::io::Read>(parser: &mut EventReader<R>) -> Result<Protocol, ApiGenError> {
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
    Ok(protocol)
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
                "name" => {
                    if def.name.is_empty() {
                        def.name = read_text_content(parser)?;
                    } else {
                        // This is a name of a member or something else, so we need to consume it
                        // without assigning to def.name.
                        read_text_content(parser)?;
                    }
                }
                "enum" => {
                    let new_enum = parse_enum(parser)?;
                    def.items.push(new_enum.name.clone());
                    api.add_enum(new_enum)?;
                }
                "flags" => {
                    while let Some(_) =
                        parse_block_item(parser, "flags", "flag", |p| -> Result<(), ApiGenError> {
                            let flag = parse_flag(p)?;
                            def.items.push(flag.name.clone());
                            api.add_flag(flag)?;
                            Ok(())
                        })?
                    {}
                }
                "constants" => {
                    while let Some(_) = parse_block_item(
                        parser,
                        "constants",
                        "constant",
                        |p| -> Result<(), ApiGenError> {
                            let constant = parse_constant(p)?;
                            def.items.push(constant.name.clone());
                            api.add_constant(constant)?;
                            Ok(())
                        },
                    )? {}
                }
                "structs" => {
                    while let Some(_) = parse_block_item(
                        parser,
                        "structs",
                        "struct",
                        |p| -> Result<(), ApiGenError> {
                            let new_struct = parse_struct(p)?;
                            def.items.push(new_struct.common.name.clone());
                            api.add_struct(new_struct)?;
                            Ok(())
                        },
                    )? {}
                }
                "extensible_structs" => {
                    let (stypes_name, parsed_structs) = parse_extensible_structs(parser)?;
                    for s in &parsed_structs {
                        def.items.push(s.common.name.clone());
                    }
                    def.items.push(stypes_name.clone());
                    api.add_extensible_structs(stypes_name, parsed_structs)?;
                }
                "objects" => {
                    while let Some(_) = parse_block_item(
                        parser,
                        "objects",
                        "object",
                        |p| -> Result<(), ApiGenError> {
                            let mut object = parse_object(p)?;
                            // Hack, object doesn't have a name.
                            object.name = object.ffi.clone();
                            def.items.push(object.name.clone());
                            api.add_object(object);
                            Ok(())
                        },
                    )? {}
                }
                "function" => {
                    let function = parse_function(parser)?;
                    def.items.push(function.name.clone());
                    api.add_function(function);
                }
                "protocol" => {
                    let protocol = parse_protocol(parser)?;
                    let protocol_struct_name =
                        format!("{}CommandHdr", to_pascal_case(&protocol.name));
                    def.items.push(protocol_struct_name);
                    def.items.push(protocol.name.clone());
                    api.add_protocol(protocol)?;
                }
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "define" => break,
            _ => {}
        }
    }
    api.add_definition(def);
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
    let mut api = Api::new();

    loop {
        match parser.next()? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "api" => {
                    let name = find_attribute_value(&attributes, "name")
                        .unwrap_or_else(|| "unknown".to_string());
                    api.set_name(name);
                }
                "copyright" => {
                    let copyright = parse_copyright(parser)?;
                    api.set_copyright(copyright);
                }
                "version" => {
                    let version = read_text_content(parser)?.parse()?;
                    api.set_version(version);
                }
                "define" => parse_define(parser, &mut api)?,
                "generated_file" => {
                    let gen_file = parse_generated_file(parser)?;
                    api.add_generated_file(gen_file);
                }
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
        XmlEvent::Characters(s) if s.trim().is_empty() => {
            // It's whitespace, try the next event.
            parse_block_item(parser, block_name, item_name, parse_fn)
        }
        XmlEvent::Whitespace(_) => {
            // It's whitespace, try the next event.
            parse_block_item(parser, block_name, item_name, parse_fn)
        }
        XmlEvent::Comment(_) => {
            // It's a comment, try the next event.
            parse_block_item(parser, block_name, item_name, parse_fn)
        }
        _ => Ok(None),
    }
}

pub fn parse_api(filename: &Path) -> Result<Api, ApiGenError> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut parser = EventReader::new(reader);
    parse_api_internal(&mut parser)
}
