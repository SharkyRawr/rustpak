#[cfg(test)]
mod tests {
    use rustpak::Pak;
    use std::error::Error;

    #[test]
    fn pak_from_file() -> Result<(), Box<dyn Error>> {
        let pak = Pak::from_file("extras.pak".to_string());
        match pak {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
