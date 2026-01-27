# rustpak

[![Crates.io](https://img.shields.io/crates/v/rustpak)](https://crates.io/crates/rustpak)
[![Documentation](https://img.shields.io/docsrs/rustpak)](https://docs.rs/rustpak)
[![License](https://img.shields.io/badge/license-CC--BY--NC--SA--4.0-blue.svg)](https://creativecommons.org/licenses/by-nc-sa/4.0/)

Rust library and CLI tool for reading and writing GoldSrc `.pak` archive files.

## About

The `.pak` format is a simple archive format used by Quake, Half-Life, and other GoldSrc engine games to package game assets. This library provides functionality to parse, create, and modify `.pak` archives.

## Features

- Parse existing `.pak` files
- Create new `.pak` archives
- Add, remove, and extract files
- CLI tool for common operations
- Full binary format validation
- Supports files with directory structures

## Installation

### CLI Tool

```bash
cargo install rustpak
```

### Library

Add to your `Cargo.toml`:

```toml
[dependencies]
rustpak = "0.1"
```

## Usage

### CLI

List files in a `.pak` archive:

```bash
rustpak list archive.pak
```

Extract a file:

```bash
rustpak extract archive.pak maps/e1m1.bsp output/e1m1.bsp
```

Extract with directory structure:

```bash
rustpak extract archive.pak maps/e1m1.bsp output/ -r
```

Append a file to an archive:

```bash
rustpak append archive.pak new_file.dat
```

### Library

```rust
use rustpak::Pak;
use rustpak::PakFileEntry;

// Load an existing archive
let mut pak = Pak::from_file("archive.pak".to_string())?;

// List all files
for file in &pak.files {
    println!("{} - {} bytes", file.name, file.size);
}

// Add a new file
let data = b"Hello, world!".to_vec();
let entry = PakFileEntry::new(
    "new_file.txt".to_string(),
    12 + (pak.files.len() as u32 + 1) * 64,
    data,
);
pak.add_file(entry)?;

// Save modifications
pak.save("archive_modified.pak".to_string())?;
```

Extract a file:

```rust
use rustpak::Pak;

let pak = Pak::from_file("archive.pak".to_string())?;

for file in &pak.files {
    if file.name == "maps/e1m1.bsp" {
        file.save_to("output/e1m1.bsp".to_string(), false)?;
        break;
    }
}
```

## Pak File Format

A `.pak` file consists of three parts:

1. **Header** (12 bytes)
   - Magic: "PACK" (4 bytes)
   - Directory offset: Offset to directory (4 bytes)
   - Directory size: Size of directory (4 bytes)

2. **Directory** (64 bytes per file)
   - File name: Null-terminated string, 56 bytes
   - File offset: Offset to file data (4 bytes)
   - File size: Size of file data (4 bytes)

3. **File Data**
   - Raw file contents at offsets specified in directory

## Documentation

Full API documentation is available at [docs.rs](https://docs.rs/rustpak)

## License

This project is licensed under the [Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International License (CC BY-NC-SA 4.0)](https://creativecommons.org/licenses/by-nc-sa/4.0/).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Based on the original Quake `.pak` format specification
- Inspired by various GoldSrc modding tools
