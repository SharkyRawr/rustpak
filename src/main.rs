mod lib;
use lib::Pak;

fn main() {
    let pak = Pak::from_file("extras.pak".to_string()).unwrap();
    println!("Pak file found: {}", pak);

    println!("Listing all files in {}:", pak.pak_path);
    pak.files.iter().for_each(|i| println!("\t{}", i.name));

    let test_file = &pak.files[0];
    println!(
        "Trying to save first file ({}) to 'test.bin'...",
        test_file.name
    );
    test_file.save_to("test.bin".to_string()).unwrap();
    println!("If it didn't crash, it succeeded! Yaaay!!! 🌈🦄🦊");
}
