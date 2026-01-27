//! Rustpak - A library for reading and writing GoldSrc .pak archive files
//!
//! This library provides functionality to work with .pak files used by
//! Quake, Half-Life, and other GoldSrc engine games. The .pak format
//! is a simple archive format containing a header, a file table, and file data.
//!
//! # Basic Usage
//!
//! ```no_run
//! use rustpak::Pak;
//!
//! // Load an existing pak file
//! let pak = Pak::from_file("data.pak".to_string()).unwrap();
//!
//! // List all files
//! for file in &pak.files {
//!     println!("{} - {} bytes", file.name, file.size);
//! }
//!
//! // Save modifications
//! pak.save("data_modified.pak".to_string()).unwrap();
//! ```

extern crate byteorder;
use std::{
    borrow::Borrow,
    error::Error,
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom},
    path,
};

use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};

/// Header structure for .pak archive files
///
/// The header is always 12 bytes and contains magic identifier,
/// offset to the file table, and size of the file table.
#[derive(Debug)]
#[repr(C)]
pub struct PakHeader {
    /// Should be "PACK" (not null-terminated).
    pub id: String,
    /// Index to the beginning of the file table.
    pub offset: u32,
    /// Size of the file table.
    pub size: u32,
}

impl Default for PakHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl PakHeader {
    /// Creates a new PakHeader with default values
    ///
    /// Returns a header with "PACK" magic and zeroed offset/size.
    pub fn new() -> PakHeader {
        PakHeader {
            id: "PACK".to_string(),
            offset: 0,
            size: 0,
        }
    }

    /// Parses a PakHeader from a byte slice
    ///
    /// # Arguments
    ///
    /// * `buf` - A byte slice containing at least 12 bytes of header data
    ///
    /// # Panics
    ///
    /// Panics if the buffer is too short or contains invalid UTF-8 for the magic bytes
    pub fn from_u8(buf: &[u8]) -> PakHeader {
        PakHeader {
            id: String::from_utf8(buf[0..4].to_vec()).unwrap(),
            offset: LittleEndian::read_u32(&buf[4..8]),
            size: LittleEndian::read_u32(&buf[8..12]),
        }
    }

    /// Writes the header to a writer
    ///
    /// # Arguments
    ///
    /// * `writer` - Any type implementing `std::io::Write`
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the writer fails
    #[allow(dead_code)]
    pub fn write_to<W: io::Write>(&self, mut writer: W) -> Result<(), Box<dyn Error>> {
        writer.write_all(self.id.as_bytes())?;
        writer.write_u32::<LittleEndian>(self.offset)?;
        writer.write_u32::<LittleEndian>(self.size)?;
        Ok(())
    }
}

/// Represents a single file entry within a .pak archive
///
/// Each file entry consists of metadata (name, offset, size) and the
/// actual file data. Entries in the file table are always 64 bytes,
/// with the name padded to 56 bytes with null terminators.
#[derive(Debug)]
#[repr(C)]
pub struct PakFileEntry {
    /// 56 byte null-terminated string including path.
    /// Example: "maps/e1m1.bsp"
    pub name: String,
    /// The offset from the beginning of the pak file to this file's contents
    pub offset: u32,
    /// The size of this file in bytes
    pub size: u32,
    /// The raw file data
    data: Vec<u8>,
}

impl PakFileEntry {
    /// Parses a PakFileEntry from header buffer and full file data
    ///
    /// # Arguments
    ///
    /// * `header_buf` - 64 bytes containing the file entry metadata
    /// * `file_buf` - Full .pak file data to extract file contents from
    ///
    /// # Panics
    ///
    /// Panics if buffer sizes are insufficient or data is out of bounds
    pub fn from_u8(header_buf: &[u8], file_buf: &[u8]) -> PakFileEntry {
        let namebuf = header_buf[0..56].to_vec();

        let nul_range_end = namebuf
            .iter()
            .position(|&c| c == b'\0')
            .unwrap_or(namebuf.len()); // default to length if no `\0` present

        let offset = LittleEndian::read_u32(&header_buf[56..60]);
        let size = LittleEndian::read_u32(&header_buf[60..64]);

        PakFileEntry {
            name: String::from_utf8(header_buf[0..nul_range_end].to_vec())
                .unwrap()
                .trim()
                .to_string(),
            offset,
            size,
            data: (file_buf[offset as usize..(offset + size) as usize]).to_vec(),
        }
    }

    /// Extracts this file's contents to disk
    ///
    /// # Arguments
    ///
    /// * `path` - Destination path for the extracted file
    /// * `with_full_path` - If true, creates directory structure from path;
    ///   if false, only uses filename
    ///
    /// # Errors
    ///
    /// Returns an error if file operations fail
    pub fn save_to(&self, path: String, with_full_path: bool) -> Result<String, std::io::Error> {
        let data: &Vec<u8> = self.data.borrow();
        let mut path = path::Path::new(&path);

        if with_full_path {
            fs::create_dir_all(path.parent().unwrap())?;
        } else {
            path = path::Path::new(path.file_name().unwrap().to_str().unwrap())
        }

        std::fs::write(path, data)?;
        Ok(path.to_str().unwrap().to_string())
    }

    /// Returns a reference to the file's raw data
    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

    /// Creates a new PakFileEntry with the given parameters
    ///
    /// # Arguments
    ///
    /// * `name` - The file name/path (will be stored in file table)
    /// * `offset` - Byte offset within the .pak file where data will be stored
    /// * `data` - The actual file contents
    #[allow(dead_code)]
    pub fn new(name: String, offset: u32, data: Vec<u8>) -> PakFileEntry {
        PakFileEntry {
            name,
            offset,
            size: data.len() as u32,
            data: data.to_vec(),
        }
    }

    /// Writes the file entry metadata (64 bytes) to a writer
    ///
    /// The name is padded with null bytes to 56 bytes, followed by
    /// offset (4 bytes) and size (4 bytes).
    ///
    /// # Arguments
    ///
    /// * `writer` - Any type implementing `std::io::Write`
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails
    #[allow(dead_code)]
    pub fn write_to<W: io::Write>(&self, mut writer: W) -> Result<(), Box<dyn Error>> {
        let mut buf = self.name.as_bytes().to_vec();
        //buf.fill_with(self.name.as_bytes());
        while buf.len() < 56 {
            buf.push(0_u8);
        }
        writer.write_all(buf.as_slice())?;
        writer.write_u32::<LittleEndian>(self.offset)?;
        writer.write_u32::<LittleEndian>(self.size)?;

        Ok(())
    }
}

/// Represents a complete .pak archive file
///
/// Contains the archive header and collection of file entries.
/// Provides methods for reading, writing, and manipulating archives.
#[derive(Debug)]
pub struct Pak {
    /// Path to the .pak file on disk (if loaded from file)
    pub pak_path: String,
    /// Archive header containing PACK magic, offset, and size
    pub header: PakHeader,
    /// All files contained in the archive
    pub files: Vec<PakFileEntry>,
}

impl Default for Pak {
    fn default() -> Self {
        Self::new()
    }
}

impl Pak {
    /// Creates a new empty Pak archive
    ///
    /// Returns a Pak with empty path, default header, and no files.
    #[allow(dead_code)]
    #[no_mangle]
    pub fn new() -> Pak {
        Pak {
            pak_path: "".to_string(),
            header: PakHeader::new(),
            files: Vec::new(),
        }
    }

    /// Loads a .pak archive from disk
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the .pak file to load
    ///
    /// # Errors
    ///
    /// Returns an error if the file doesn't exist, can't be read,
    /// or has an invalid format
    #[no_mangle]
    pub fn from_file(path: String) -> Result<Pak, Box<dyn Error>> {
        let bytes = std::fs::read(&path)?;
        let pakheader = PakHeader::from_u8(&bytes);
        let num_files = pakheader.size / 64;

        let file_table_offset = pakheader.offset;
        let mut my_offset: u32 = 0;
        let mut pakfiles: Vec<PakFileEntry> = Vec::new();

        for _i in 0..num_files {
            let file_entry = PakFileEntry::from_u8(
                &bytes[(file_table_offset + my_offset) as usize
                    ..(file_table_offset + my_offset + 64) as usize],
                &bytes,
            );
            pakfiles.push(file_entry);

            my_offset += 64;
        }

        Ok(Pak {
            pak_path: path,
            header: pakheader,
            files: pakfiles,
        })
    }

    /// Adds a file entry to the archive
    ///
    /// # Arguments
    ///
    /// * `file` - The PakFileEntry to add
    ///
    /// # Errors
    ///
    /// Returns an error if a file with the same name already exists
    #[allow(dead_code)]
    #[no_mangle]
    pub fn add_file(&mut self, file: PakFileEntry) -> Result<&mut Pak, Box<dyn Error>> {
        match self.files.iter().find(|f| f.name.eq(&file.name)) {
            Some(_) => Err(Box::new(PakFileError {
                msg: "File already exists".to_string(),
            })),
            None => {
                self.files.push(file);
                Ok(self)
            }
        }
    }

    /// Removes a file from the archive by name
    ///
    /// # Arguments
    ///
    /// * `filename` - Name of the file to remove
    ///
    /// # Errors
    ///
    /// Returns an error if the file is not found
    #[allow(dead_code)]
    #[no_mangle]
    pub fn remove_file(&mut self, filename: String) -> Result<(), Box<dyn Error>> {
        if let Some(p) = self.files.iter().position(|p| p.name.eq(&filename)) {
            self.files.remove(p);
            Ok(())
        } else {
            Err(Box::new(PakFileError {
                msg: "file entry not found".to_string(),
            }))
        }
    }

    /// Saves the archive to a file
    ///
    /// Writes the archive in standard .pak format:
    /// - 12-byte header
    /// - File table entries (64 bytes each)
    /// - File data at specified offsets
    ///
    /// # Arguments
    ///
    /// * `filename` - Path where the .pak file should be saved
    ///
    /// # Errors
    ///
    /// Returns an error if file creation or writing fails
    #[allow(dead_code)]
    #[no_mangle]
    pub fn save(&self, filename: String) -> Result<(), Box<dyn Error>> {
        let mut hdr = PakHeader::new();
        hdr.offset = 12;
        hdr.size = (self.files.len() * 64) as u32;

        let mut f = File::create(filename)?;
        hdr.write_to(&f)?;

        for file in self.files.iter() {
            file.write_to(&f)?;
        }

        for file in self.files.iter() {
            f.seek(SeekFrom::Start(file.offset as u64))?;
            io::Write::write(&mut f, file.data.as_slice())?;
        }

        Ok(())
    }

    /// Appends a file from disk to the archive
    ///
    /// Reads a file from disk and adds it to the archive with the
    /// path specified in `pakfilepath`.
    ///
    /// # Arguments
    ///
    /// * `infilepath` - Path to the file on disk to read
    /// * `pakfilepath` - Path/name to store within the .pak archive
    ///
    /// # Errors
    ///
    /// Returns an error if the input file doesn't exist or can't be read,
    /// or if a file with that name already exists in the archive
    pub fn append_file(
        &mut self,
        infilepath: String,
        pakfilepath: String,
    ) -> Result<(), Box<dyn Error>> {
        let newfilepath = path::Path::new(&infilepath);
        if !newfilepath.exists() {
            return Err(Box::new(PakFileError {
                msg: "File does not exist!".to_string(),
            }));
        }

        fn get_last_offset(path: String) -> u32 {
            let f = File::open(path).unwrap();
            f.metadata().unwrap().len() as u32
        }

        let last_offset = get_last_offset(self.pak_path.clone());

        fn get_file_data(path: String) -> Vec<u8> {
            let mut f = File::open(path).unwrap();
            let mut vec: Vec<u8> = Vec::new();
            let buf: &mut Vec<u8> = vec.as_mut();
            f.read_to_end(buf).unwrap();
            buf.to_vec()
        }
        let data = get_file_data(infilepath);

        let fe = PakFileEntry::new(pakfilepath.to_string(), last_offset, data);
        self.add_file(fe).unwrap();
        Ok(())
    }
}

impl std::fmt::Display for Pak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<Pak structure from file {} with {} files>",
            self.pak_path,
            self.files.len()
        )
    }
}

/// Error type for .pak file operations
///
/// Used to report errors during file loading, saving, and manipulation.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct PakFileError {
    /// Error message describing what went wrong
    pub msg: String,
}

impl std::fmt::Display for PakFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for PakFileError {}
