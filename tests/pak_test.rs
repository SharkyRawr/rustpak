#[cfg(test)]
mod tests {
    use rustpak::{Pak, PakFileEntry, PakFileError};
    use std::{error::Error};

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
}
