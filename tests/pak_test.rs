#[cfg(test)]
mod tests {
    use rustpak::{Pak, PakFileEntry, PakFileError};
    use std::error::Error;

    #[test]
    fn pak_from_file() -> Result<(), Box<dyn Error>> {
        let pak = Pak::from_file("extras.pak".to_string());
        match pak {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    #[test]
    fn pak_add_file() -> Result<(), Box<dyn Error>> {
        let mut pak = Pak::new();
        match pak.add_file(PakFileEntry::new("test.txt".to_string(), 0, vec![b'H'])) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    #[test]
    fn pak_add_duplicate_file() -> Result<(), Box<dyn Error>> {
        let mut pak = Pak::new();
        pak.add_file(PakFileEntry::new("test.txt".to_string(), 0, vec![b'H']))
            .unwrap();
        let result = pak.add_file(PakFileEntry::new("test.txt".to_string(), 0, vec![b'H']));
        if result.is_err() {
            Ok(())
        } else {
            Err(Box::new(PakFileError {
                msg: "Failed".to_string(),
            }))
        }
    }

    #[test]
    fn pak_delete_file() -> Result<(), Box<dyn Error>> {
        let mut pak = Pak::new();
        pak.add_file(PakFileEntry::new("test.txt".to_string(), 0, vec![b'H']))
            .unwrap();
        pak.remove_file("test.txt".to_string())
    }

    #[test]
    #[should_panic]
    fn pak_delete_file_nonexisting() -> () {
        let mut pak = Pak::new();
        pak.add_file(PakFileEntry::new("test.txt".to_string(), 0, vec![b'H']))
            .unwrap();
        pak.remove_file("doesnotexist.txt".to_string()).unwrap();
    }

    #[test]
    fn pak_save() -> Result<(), Box<dyn Error>> {
        let mut pak = Pak::new();
        pak.add_file(PakFileEntry::new(
            "test.txt".to_string(),
            12 + 64,
            "Hello World".as_bytes().to_vec(),
        ))
        .unwrap();
        pak.save("test.pak".to_string())
    }

    #[test]
    fn pak_save_and_load() -> Result<(), Box<dyn Error>> {
        let test_string = "Hello World".as_bytes().to_vec();
        let test_file = "test.pak";

        let mut pak = Pak::new();
        pak.add_file(PakFileEntry::new(
            "test.txt".to_string(),
            12 + 64,
            test_string.clone(),
        ))?;
        pak.save(test_file.to_string())?;

        let pak = Pak::from_file(test_file.to_string())?;
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
}
