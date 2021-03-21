extern crate byteorder;
use std::{borrow::Borrow, error::Error, rc::Rc};

use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug)]
pub struct PakHeader {
    pub id: String, // Should be "PACK" (not null-terminated).
    pub offset: u32, // Index to the beginning of the file table.
    pub size: u32 // Size of the file table.
}

impl PakHeader {
    pub fn from_u8(buf: &Vec<u8>) -> PakHeader {
        PakHeader{
            id: String::from_utf8((&buf[0..4]).to_vec()).unwrap(),
            offset: LittleEndian::read_u32(&buf[4..8]),
            size: LittleEndian::read_u32(&buf[8..12]),
        }
    }

}

#[derive(Debug)]
pub struct PakFileEntry {
    pub name: String, // 56 byte null-terminated string	Includes path. Example: "maps/e1m1.bsp".
    pub offset: u32, // The offset (from the beginning of the pak file) to the beginning of this file's contents.
    pub size: u32, // The size of this file.
    data: Rc<Vec<u8>>
}

impl PakFileEntry {
    pub fn from_u8(header_buf: &Vec<u8>, file_buf: &Vec<u8>) -> PakFileEntry {
        let namebuf = (&header_buf[0..56]).to_vec();

        let nul_range_end = namebuf.iter()
            .position(|&c| c == b'\0')
            .unwrap_or(namebuf.len()); // default to length if no `\0` present

        let offset = LittleEndian::read_u32(&header_buf[56..60]);
        let size = LittleEndian::read_u32(&header_buf[60..64]);
        
        PakFileEntry{
            name: String::from_utf8((&header_buf[0..nul_range_end]).to_vec()).unwrap().trim().to_string(),
            offset: offset,
            size: size,
            data: Rc::new((&file_buf[offset as usize..(offset+size) as usize]).to_vec())
        }
    }

    pub fn save_to(&self, path: String) -> Result<(), std::io::Error> {
        let data: &Vec<u8> = self.data.borrow();
        std::fs::write(path, data)
    }
}

#[derive(Debug)]
pub struct Pak {
    pub pak_path: String,
    pub header: PakHeader,
    pub files: Vec<PakFileEntry>,
    buf: Vec<u8>
}

impl Pak {
    pub fn new(path: String) -> Result<Pak, Box<dyn Error>> {
        let bytes = std::fs::read(path.to_string())?;
        let pakheader = PakHeader::from_u8(&bytes);
        let num_files = pakheader.size / 64;
        //println!("{:?}", pakheader);
        //println!("Pak with {} files.", num_files);
        
        let file_table_offset = pakheader.offset;
        let mut my_offset: u32 = 0;
        let mut pakfiles: Vec<PakFileEntry> = Vec::new();

        for _i in 0..num_files {

            let file_entry = PakFileEntry::from_u8(&(&bytes[(file_table_offset+my_offset) as usize..(file_table_offset+my_offset+64) as usize]).to_vec(),
             &bytes);
            pakfiles.push(file_entry);

            my_offset += 64;
        }

        Ok(Pak{
            pak_path: path.to_string(),
            header: pakheader,
            files: pakfiles,
            buf: bytes
        })
    }
}

impl std::fmt::Display for Pak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Pak structure from file {} with {} files>", self.pak_path, self.files.len())
    }
}