extern crate byteorder;
use std::{
    borrow::Borrow,
    error::Error,
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom},
    path,
};

use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};

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
    pub fn new() -> PakHeader {
        PakHeader {
            id: "PACK".to_string(),
            offset: 0,
            size: 0,
        }
    }

    pub fn from_u8(buf: &[u8]) -> PakHeader {
        PakHeader {
            id: String::from_utf8(buf[0..4].to_vec()).unwrap(),
            offset: LittleEndian::read_u32(&buf[4..8]),
            size: LittleEndian::read_u32(&buf[8..12]),
        }
    }

    #[allow(dead_code)]
    pub fn write_to<W: io::Write>(&self, mut writer: W) -> Result<(), Box<dyn Error>> {
        writer.write_all(self.id.as_bytes())?;
        writer.write_u32::<LittleEndian>(self.offset)?;
        writer.write_u32::<LittleEndian>(self.size)?;
        Ok(())
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct PakFileEntry {
    pub name: String, // 56 byte null-terminated string	Includes path. Example: "maps/e1m1.bsp".
    pub offset: u32, // The offset (from the beginning of the pak file) to the beginning of this file's contents.
    pub size: u32,   // The size of this file.
    data: Vec<u8>,
}

impl PakFileEntry {
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

    #[allow(dead_code)]
    pub fn new(name: String, offset: u32, data: Vec<u8>) -> PakFileEntry {
        PakFileEntry {
            name,
            offset,
            size: data.len() as u32,
            data: data.to_vec(),
        }
    }

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

#[derive(Debug)]
pub struct Pak {
    pub pak_path: String,
    pub header: PakHeader,
    pub files: Vec<PakFileEntry>,
}

impl Default for Pak {
    fn default() -> Self {
        Self::new()
    }
}

impl Pak {
    #[allow(dead_code)]
    #[no_mangle]
    pub fn new() -> Pak {
        Pak {
            pak_path: "".to_string(),
            header: PakHeader::new(),
            files: Vec::new(),
        }
    }

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

#[derive(Debug, Clone)]
#[repr(C)]
pub struct PakFileError {
    pub msg: String,
}

impl std::fmt::Display for PakFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for PakFileError {}
