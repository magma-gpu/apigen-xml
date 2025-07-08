# apigen-xml

The purpose of apigen-xml` is to auto-generate efficient, zero-copy encoders and decoders
for an XML-based API.

![apigen logo](images/apigen-logo.png)

This project is a modern re-interpretation of the original`apigen` used for gfxstream GLES, which
can be found [here](https://android.googlesource.com/platform/hardware/google/gfxstream/+/refs/heads/main/codegen/generic-apigen/).

While many graphics APIs are defined in XML (e.g., by Khronos), those files are often very large and
not optimized for generating a full encode/decode process. For example, tools often filter a subset
of extensions they want to support.

This tool uses an intermediary XML format with a schema specifically designed for `apigen-xml`.

The goal is to generate:

- Headers
- C-FFI to Rust bindings
- Rust protocols with encode/decode logic
- Rust to C-FFI (if necessary)

## Schema Elements

The key elements of the XML schema are:

- **enums, structs, constants**: Plain old data structures.
- **extensible_structs**: These have pointers in them for FFI and need special logic for
  encode/decode.
- **functions**: Usually with a C representations.
- **protocols**: Opcodes, commands, often with extensible structs.
- **definitions**: A block containing plain old data, extensible structs, and protocols.
- **generated_files**: Specifies which definitions to include and how to generate the final files.

## How To Use

### Running the generator

To generate the API files, run the following command. Note that `${out_dir}` must be an absolute path, as shell expansions like `~` are not supported.

```bash
./target/debug/apigen-xml --filename=xml/magma.xml --out-dir=${out_dir}
```

An output directory will be created if it did not previously exist.

### Formatting XML

After modifying an XML file, ensure it is correctly formatted by running:

```bash
xmllint --format xml/magma.xml > magma_formatted.xml && mv magma_formatted.xml xml/magma.xml
```
