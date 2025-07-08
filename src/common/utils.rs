// Copyright 2025 Google
// SPDX-License-Identifier: MIT

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
