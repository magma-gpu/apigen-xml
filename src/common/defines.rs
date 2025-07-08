// Copyright 2025 Google
// SPDX-License-Identifier: MIT

use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub enum DefinitionItem {
    Constant(Constant),
    Struct(StructDef),
    Enum(Enum),
    Flag(Flag),
    ExtensibleStruct(ExtensibleStruct),
    ExtensibleStructs(ExtensibleStructs),
    Object(Object),
    Function(Function),
    Protocol(Protocol),
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct Definition {
    pub name: String,
    pub items: Vec<String>,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct GeneratedFile {
    pub out_path: String,
    pub file_name: String,
    pub file_type: String,
    pub includes: Vec<String>,
    pub instantiations: Vec<String>,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct Copyright {
    pub spdx: String,
    pub holder: String,
    pub year: u32,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct Constant {
    pub type_name: String,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct StructCommon {
    pub name: String,
    pub members: Vec<Member>,
    pub array_info: Vec<ArrayInfo>,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct StructDef {
    #[serde(flatten)]
    pub common: StructCommon,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct ArrayInfo {
    pub array_member_name: String,
    pub array_base_type: String,
    pub count_member_name: String,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Member {
    pub type_name: String,
    pub qualifier: String,
    pub name: String,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct Enum {
    pub name: String,
    pub type_name: String,
    pub entries: Vec<EnumEntry>,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct EnumEntry {
    pub name: String,
    pub value: String,
}

impl From<SType> for EnumEntry {
    fn from(stype: SType) -> Self {
        EnumEntry {
            name: stype.name,
            value: stype.value,
        }
    }
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct Flag {
    pub name: String,
    pub type_name: String,
    pub entries: Vec<EnumEntry>,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct SType {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct ExtensibleStruct {
    pub stype: SType,
    #[serde(flatten)]
    pub common: StructCommon,
    #[serde(default)]
    pub padding: Option<Member>,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct ExtensibleStructs {
    pub stypes_name: String,
    pub structs: Vec<ExtensibleStruct>,
    pub protocol_struct: StructCommon,
    pub ffi_struct: StructCommon,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct Object {
    pub name: String,
    pub ffi: String,
    pub rust: String,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct Function {
    pub name: String,
    pub ret: String,
    pub members: Vec<Member>,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct Opcode {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct Request {
    pub opcode: Opcode,
    pub members: Vec<Member>,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct Response {
    pub opcode: Opcode,
    pub members: Vec<Member>,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct Protocol {
    pub name: String,
    pub protocol_struct_name: String,
    pub requests: Vec<Request>,
    pub responses: Vec<Response>,
}
