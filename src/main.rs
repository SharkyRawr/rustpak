mod lib;
use std::error::Error;

use lib::Pak;

extern crate clap;
use clap::{Arg, App, SubCommand};

fn main() {

    let matches = App::new("Pak")
        .version("0.1")
        .author("Sophie Luna Schumann <me@sophie.lgbt>")
        .about("Quake/Half-Life Pak file manipulation utility")
        
        .subcommand(SubCommand::with_name("list")
            .about("List files inside .pak")
            .arg(Arg::with_name("path")
                .help("Path to .pak file")
                .index(1)
                .required(true))
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("list") {
        let path = matches.value_of("path").unwrap();
        match list_pak_file(path) {
            Ok(_) => {}
            Err(e) => {eprintln!("Pak file error: {}", e)}
        }
    }
}

fn list_pak_file(path: &str) -> Result<(), Box<dyn Error>> {
    let pak = Pak::from_file(path.to_string())?;
    //println!("Pak file found: {}", pak);
    //println!("Listing all files in {}:", pak.pak_path);
    pak.files.iter().for_each(|i| println!("{}", i.name));
    Ok(())
}
