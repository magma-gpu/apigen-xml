// Copyright 2025 Google
// SPDX-License-Identifier: MIT

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use xml::attribute::OwnedAttribute;
use xml::reader::{EventReader, XmlEvent};

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
                "name" => struct_def.name = read_text_content(parser)?,
                "member" => struct_def.members.push(parse_member(parser)?),
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "struct" => break,
            _ => {}
        }
    }
    Ok(struct_def)
}

/// Parses a single <command> element.
fn parse_command<R: std::io::Read>(parser: &mut EventReader<R>) -> Result<Command, ApiGenError> {
    let mut command = Command::default();
    loop {
        match parser.next()? {
            XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                "opcode" => command.opcode = read_text_content(parser)?,
                "member" => command.members.push(parse_member(parser)?),
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "command" => break,
            _ => {}
        }
    }
    Ok(command)
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
            XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                "name" => struct_def.name = read_text_content(parser)?,
                "stype" => struct_def.stype = read_text_content(parser)?,
                "member" => struct_def.members.push(parse_member(parser)?),
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "extensible_struct" => break,
            _ => {}
        }
    }
    Ok(struct_def)
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
                "command" => protocol.commands.push(parse_command(parser)?),
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "protocol" => break,
            _ => {}
        }
    }
    Ok(protocol)
}

/// Parses a <define> block and populates the api.
fn parse_define<R: std::io::Read>(parser: &mut EventReader<R>) -> Result<Definition, ApiGenError> {
    let mut def = Definition::default();
    loop {
        match parser.next()? {
            XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                "name" => def.name = read_text_content(parser)?,
                "enum" => def.enums.push(parse_enum(parser)?),
                "flags" => {
                    while let Some(f) =
                        parse_block_item(parser, "flags", "flag", |p| parse_flag(p))?
                    {
                        def.flags.push(f);
                    }
                }
                "constants" => {
                    while let Some(c) =
                        parse_block_item(parser, "constants", "constant", |p| parse_constant(p))?
                    {
                        def.constants.push(c);
                    }
                }
                "structs" => {
                    while let Some(s) =
                        parse_block_item(parser, "structs", "struct", |p| parse_struct(p))?
                    {
                        def.structs.push(s);
                    }
                }
                "extensible_struct" => def
                    .extensible_structs
                    .push(parse_extensible_struct(parser)?),
                "objects" => {
                    while let Some(o) =
                        parse_block_item(parser, "objects", "object", |p| parse_object(p))?
                    {
                        def.objects.push(o);
                    }
                }
                "function" => def.functions.push(parse_function(parser)?),
                "protocol" => def.protocols.push(parse_protocol(parser)?),
                _ => {}
            },
            XmlEvent::EndElement { name } if name.local_name == "define" => break,
            _ => {}
        }
    }
    Ok(def)
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
                "define" => api.definitions.push(parse_define(parser)?),
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
    parse_fn: F,
) -> Result<Option<T>, ApiGenError>
where
    R: std::io::Read,
    F: Fn(&mut EventReader<R>) -> Result<T, ApiGenError>,
{
    loop {
        match parser.next()? {
            XmlEvent::StartElement { name, .. } if name.local_name == item_name => {
                return Ok(Some(parse_fn(parser)?))
            }
            XmlEvent::EndElement { name } if name.local_name == block_name => {
                return Ok(None); // Sentinel to stop looping
            }
            _ => {}
        }
    }
}

pub fn parse_api(filename: &Path) -> Result<Api, ApiGenError> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut parser = EventReader::new(reader);
    parse_api_internal(&mut parser)
}
