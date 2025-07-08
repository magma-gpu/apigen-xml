// Copyright © 2025 Google
// SPDX-License-Identifier: MIT

// --- Constants ---

pub const MAGMA_MAX_MEMORY_HEAPS: usize = 32;
pub const MAGMA_MAX_MEMORY_TYPES: usize = 16;

// --- Structs ---

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct MagmaCreateBufferInfo {
    pub memory_type_idx: u32,
    pub alignment: u32,
    pub common_flags: u32,
    pub vendor_flags: u32,
    pub size: u64,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct MagmaMemoryType {
    pub property_flags: u32,
    pub heap_idx: u32,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct MagmaHeap {
    pub heap_flags: u64,
    pub heap_idx: u64,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct MagmaHeapBudget {
    pub budget: u64,
    pub usage: u64,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct MagmaMemoryProperties {
    pub memory_type_count: u32,
    pub memory_heap_count: u32,
    pub memory_types: [u32; MAGMA_MAX_MEMORY_TYPES],
    pub memory_heaps: [u32; MAGMA_MAX_MEMORY_HEAPS],
}

// --- Enums ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MagmaOpcode {
    MagmaCreateDevice = 0,
    MagmaDestroyDevice = 1,
    MagmaCreateBuffer = 2,
    MagmaDestroyBuffer = 3,
    MagmaCreateContext = 4,
    MagmaDestroyContext = 5,
}

// --- Commands ---

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct MagmaCreateDeviceRequest {
    pub device_id: u32,
    pub create_buffer_info: MagmaCreateBufferInfo,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct MagmaCreateBufferRequest {
    pub device_id: u32,
    pub create_buffer_info: MagmaCreateBufferInfo,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct magma_command_hdr {
    pub proto: u32,
    pub size: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MagmaCreateDeviceCmd {
    pub hdr: magma_command_hdr,
    pub device_id: u32,
    pub create_buffer_info: MagmaCreateBufferInfo,
    pub padding: [u8; 4],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MagmaCreateBufferCmd {
    pub hdr: magma_command_hdr,
    pub device_id: u32,
    pub create_buffer_info: MagmaCreateBufferInfo,
    pub padding: [u8; 4],
}
