extern crate byteorder;
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
    name: String, // 56 byte null-terminated string	Includes path. Example: "maps/e1m1.bsp".
    offset: u32, // The offset (from the beginning of the pak file) to the beginning of this file's contents.
    size: u32 // The size of this file.
}

impl PakFileEntry {
    pub fn from_u8(buf: &Vec<u8>) -> PakFileEntry {
        let namebuf = (&buf[0..56]).to_vec();

        let nul_range_end = namebuf.iter()
            .position(|&c| c == b'\0')
            .unwrap_or(namebuf.len()); // default to length if no `\0` present
        
        PakFileEntry{
            name: String::from_utf8((&buf[0..nul_range_end]).to_vec()).unwrap().trim().to_string(),
            offset: LittleEndian::read_u32(&buf[56..60]),
            size: LittleEndian::read_u32(&buf[60..64])
        }
    }
}

fn main() {
    let bytes = std::fs::read("extras.pak").unwrap();
    let pakheader = PakHeader::from_u8(&bytes);
    let num_files = pakheader.size / 64;
    println!("{:?}", pakheader);
    println!("Pak with {} files.", num_files);
    
    let file_table_offset = pakheader.offset;
    let mut my_offset: u32 = 0;

    for i in 0..num_files {

        let file_entry = PakFileEntry::from_u8(&(&bytes[(file_table_offset+my_offset) as usize..(file_table_offset+my_offset+64) as usize]).to_vec());
        println!("File {}: {:?}", i, file_entry);

        my_offset += 64;
    }
}
