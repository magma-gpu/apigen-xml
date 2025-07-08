// Copyright 2025 Google
// SPDX-License-Identifier: MIT

use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct Api {
    pub name: String,
    pub copyright: Copyright,
    pub version: u32,
    pub definitions: Vec<Definition>,
    pub generated_files: Vec<GeneratedFile>,
}

#[derive(Debug, Default, Serialize)]
pub struct Definition {
    pub name: String,
    pub constants: Vec<Constant>,
    pub structs: Vec<StructDef>,
    pub enums: Vec<Enum>,
    pub flags: Vec<Flag>,
    pub extensible_structs: Vec<ExtensibleStruct>,
    pub objects: Vec<Object>,
    pub functions: Vec<Function>,
    pub protocols: Vec<Protocol>,
}

#[derive(Debug, Default, Serialize)]
pub struct GeneratedFile {
    pub out_path: String,
    pub file_name: String,
    pub file_type: String,
    pub includes: Vec<String>,
    pub instantiations: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct Copyright {
    pub spdx: String,
    pub holder: String,
    pub year: u32,
}

#[derive(Debug, Default, Serialize)]
pub struct Constant {
    pub type_name: String,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Default, Serialize)]
pub struct StructDef {
    pub name: String,
    pub members: Vec<Member>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Member {
    pub type_name: String,
    pub qualifier: String,
    pub name: String,
}

#[derive(Debug, Default, Serialize)]
pub struct Enum {
    pub name: String,
    pub type_name: String,
    pub entries: Vec<EnumEntry>,
}

#[derive(Debug, Default, Serialize)]
pub struct EnumEntry {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Default, Serialize)]
pub struct Flag {
    pub name: String,
    pub type_name: String,
    pub entries: Vec<EnumEntry>,
}

#[derive(Debug, Default, Serialize)]
pub struct ExtensibleStruct {
    pub name: String,
    pub stype: String,
    pub members: Vec<Member>,
}

#[derive(Debug, Default, Serialize)]
pub struct Object {
    pub ffi: String,
    pub rust: String,
}

#[derive(Debug, Default, Serialize)]
pub struct Function {
    pub name: String,
    pub ret: String,
    pub members: Vec<Member>,
}

#[derive(Debug, Default, Serialize)]
pub struct Protocol {
    pub name: String,
    pub commands: Vec<Command>,
}

#[derive(Debug, Default, Serialize)]
pub struct Command {
    pub opcode: String,
    pub members: Vec<Member>,
}
