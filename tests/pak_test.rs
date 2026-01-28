#[cfg(test)]
mod tests {
    use byteorder::{LittleEndian, ReadBytesExt};
    use rustpak::{Pak, PakFileEntry, PakFileError};
    use std::error::Error;
    use std::io::Cursor;
    use std::io::Read;

    fn verify_pak_structure(pak: &Pak, expected_file_count: usize) {
        assert_eq!(pak.header.id, "PACK", "Header should have PACK magic");
        assert_eq!(
            pak.files.len(),
            expected_file_count,
            "File count should match expected"
        );
    }

    fn verify_binary_format(
        test_file: &str,
        expected_file_count: u32,
    ) -> Result<(), Box<dyn Error>> {
        let data = std::fs::read(test_file)?;

        assert!(data.len() >= 12, "Pak file too small for header");

        let mut cursor = Cursor::new(&data);

        let mut magic = [0u8; 4];
        cursor.read_exact(&mut magic)?;
        assert_eq!(
            String::from_utf8_lossy(&magic),
            "PACK",
            "File should start with PACK magic"
        );

        let offset = cursor.read_u32::<LittleEndian>()?;
        let size = cursor.read_u32::<LittleEndian>()?;

        assert_eq!(
            size,
            expected_file_count * 64,
            "Size field should equal file count * 64"
        );

        cursor.set_position(offset as u64);
        for i in 0..expected_file_count {
            let mut name_buf = vec![0u8; 56];
            cursor.read_exact(&mut name_buf)?;

            let nul_pos = name_buf.iter().position(|&c| c == b'\0').unwrap_or(56);
            let name = String::from_utf8_lossy(&name_buf[..nul_pos]);
            assert!(!name.is_empty(), "File {} should have a valid name", i);

            let file_offset = cursor.read_u32::<LittleEndian>()?;
            let file_size = cursor.read_u32::<LittleEndian>()?;

            assert!(
                file_offset >= 12,
                "File offset {} should be after header (>= 12)",
                i
            );
            assert!(
                file_offset as usize + file_size as usize <= data.len(),
                "File {} data should be within file bounds",
                i
            );
        }

        Ok(())
    }

    #[test]
    fn pak_from_file() -> Result<(), Box<dyn Error>> {
        let pak = Pak::from_file("extras.pak".to_string())?;
        verify_pak_structure(&pak, pak.files.len());
        Ok(())
    }

    #[test]
    fn pak_add_file_and_verify_structure() -> Result<(), Box<dyn Error>> {
        let mut pak = Pak::new();
        pak.add_file(PakFileEntry::new(
            "test.txt".to_string(),
            12 + 64,
            &[b'H', b'i'],
        ))?;

        assert_eq!(pak.files.len(), 1, "Should have 1 file");
        assert_eq!(pak.files[0].name, "test.txt");
        assert_eq!(pak.files[0].size, 2);
        assert_eq!(*pak.files[0].get_data(), vec![b'H', b'i']);

        Ok(())
    }

    #[test]
    fn pak_add_duplicate_file() -> Result<(), Box<dyn Error>> {
        let mut pak = Pak::new();
        pak.add_file(PakFileEntry::new("test.txt".to_string(), 0, &[b'H']))?;
        let result = pak.add_file(PakFileEntry::new("test.txt".to_string(), 0, &[b'H']));

        if result.is_err() {
            Ok(())
        } else {
            Err(Box::new(PakFileError {
                msg: "Failed".to_string(),
            }))
        }
    }

    #[test]
    fn pak_delete_file_and_verify_structure() -> Result<(), Box<dyn Error>> {
        let mut pak = Pak::new();
        pak.add_file(PakFileEntry::new("test.txt".to_string(), 0, &[b'H']))?;
        assert_eq!(pak.files.len(), 1);

        pak.remove_file("test.txt")?;
        assert_eq!(pak.files.len(), 0, "File should be removed");

        Ok(())
    }

    #[test]
    #[should_panic]
    fn pak_delete_file_nonexisting() {
        let mut pak = Pak::new();
        pak.add_file(PakFileEntry::new("test.txt".to_string(), 0, &[b'H']))
            .unwrap();
        pak.remove_file("doesnotexist.txt").unwrap();
    }

    #[test]
    fn pak_save_and_verify_binary_format() -> Result<(), Box<dyn Error>> {
        let test_file = "test_save.pak";

        let mut pak = Pak::new();
        pak.add_file(PakFileEntry::new(
            "test.txt".to_string(),
            12 + 64,
            "Hello World".as_bytes(),
        ))?;
        pak.save(test_file.to_string())?;

        verify_binary_format(test_file, 1)?;

        let reloaded = Pak::from_file(test_file.to_string())?;
        verify_pak_structure(&reloaded, 1);
        assert_eq!(reloaded.files[0].name, "test.txt");
        assert_eq!(
            *reloaded.files[0].get_data(),
            "Hello World".as_bytes().to_vec()
        );

        std::fs::remove_file(test_file)?;
        Ok(())
    }

    #[test]
    fn pak_save_and_load() -> Result<(), Box<dyn Error>> {
        let test_string = "Hello World".as_bytes().to_vec();
        let test_file = "test.pak";

        let mut pak = Pak::new();
        pak.add_file(PakFileEntry::new(
            "test.txt".to_string(),
            12 + 64,
            &test_string,
        ))?;
        pak.save(test_file.to_string())?;

        let pak = Pak::from_file(test_file.to_string())?;
        verify_pak_structure(&pak, 1);

        let f = pak
            .files
            .iter()
            .find(|f| f.name == "test.txt")
            .ok_or_else(|| {
                Box::new(PakFileError {
                    msg: "File not found after save/load".to_string(),
                }) as Box<dyn Error>
            })?;

        assert_eq!(*f.get_data(), test_string);

        std::fs::remove_file(test_file)?;
        Ok(())
    }

    #[test]
    fn pak_multiple_files_save_and_verify() -> Result<(), Box<dyn Error>> {
        let test_file = "test_multi.pak";

        let data1 = "Content 1".as_bytes().to_vec();
        let data2 = "Content 2".as_bytes().to_vec();
        let data3 = "Content 3".as_bytes().to_vec();

        let mut pak = Pak::new();
        pak.add_file(PakFileEntry::new(
            "file1.txt".to_string(),
            12 + (3 * 64),
            &data1,
        ))?;
        pak.add_file(PakFileEntry::new(
            "file2.txt".to_string(),
            12 + (3 * 64) + data1.len() as u32,
            &data2,
        ))?;
        pak.add_file(PakFileEntry::new(
            "file3.txt".to_string(),
            12 + (3 * 64) + data1.len() as u32 + data2.len() as u32,
            &data3,
        ))?;

        pak.save(test_file.to_string())?;

        verify_binary_format(test_file, 3)?;

        let reloaded = Pak::from_file(test_file.to_string())?;
        verify_pak_structure(&reloaded, 3);

        assert_eq!(reloaded.files[0].name, "file1.txt");
        assert_eq!(*reloaded.files[0].get_data(), data1);
        assert_eq!(reloaded.files[1].name, "file2.txt");
        assert_eq!(*reloaded.files[1].get_data(), data2);
        assert_eq!(reloaded.files[2].name, "file3.txt");
        assert_eq!(*reloaded.files[2].get_data(), data3);

        std::fs::remove_file(test_file)?;
        Ok(())
    }

    #[test]
    fn pak_roundtrip_preserves_integrity() -> Result<(), Box<dyn Error>> {
        let test_file = "test_roundtrip.pak";
        let original_data = b"This is test data with special chars: \x00\x01\x02\xff".to_vec();

        let mut pak = Pak::new();
        pak.add_file(PakFileEntry::new(
            "binary.dat".to_string(),
            12 + 64,
            &original_data,
        ))?;

        pak.save(test_file.to_string())?;

        let reloaded = Pak::from_file(test_file.to_string())?;
        verify_pak_structure(&reloaded, 1);

        let loaded_data = reloaded.files[0].get_data();
        assert_eq!(
            *loaded_data, original_data,
            "Binary data should be preserved"
        );

        std::fs::remove_file(test_file)?;
        Ok(())
    }

    #[test]
    fn pak_header_structure() -> Result<(), Box<dyn Error>> {
        let test_file = "test_header.pak";

        let mut pak = Pak::new();
        pak.add_file(PakFileEntry::new(
            "test.txt".to_string(),
            12 + 64,
            "Test".as_bytes(),
        ))?;

        pak.save(test_file.to_string())?;

        let data = std::fs::read(test_file)?;
        assert!(data.len() >= 12, "Header should be 12 bytes");

        assert_eq!(&data[0..4], b"PACK", "Magic bytes should be PACK");

        let mut cursor = Cursor::new(&data[4..8]);
        let offset = cursor.read_u32::<LittleEndian>()?;
        assert_eq!(offset, 12, "Offset should point after header");

        std::fs::remove_file(test_file)?;
        Ok(())
    }
}
