#[cfg(test)]
mod tests {
    use std::error::Error;
    use rustpak::Pak;

    #[test]
    fn pak_from_file() -> Result<(), Box<dyn Error>> {
        let pak = Pak::from_file("extras.pak".to_string());
        match pak {
            Ok(_) => Ok(()),
            Err(e) => Err(e)
        }
    }
}