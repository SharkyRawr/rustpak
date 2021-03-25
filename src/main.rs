mod lib;
use std::error::Error;

use lib::Pak;

extern crate clap;
use clap::{App, Arg, SubCommand};

fn main() {
    let matches = App::new("Pak")
        .version("0.1")
        .author("Sophie Luna Schumann <me@sophie.lgbt>")
        .about("Quake/Half-Life Pak file manipulation utility")
        .subcommand(
            SubCommand::with_name("list")
                .about("List files inside .pak")
                .arg(
                    Arg::with_name("pakfile")
                        .help("Path to .pak file")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("extract")
                .about("Extract files from Pak")
                .arg(
                    Arg::with_name("pakfile")
                        .help("Path to .pak file")
                        .index(1)
                        .required(true),
                )
                .arg(
                    Arg::with_name("path")
                        .help("Filename to extract")
                        .index(2)
                        .required(true),
                )
                .arg(
                    Arg::with_name("outfile")
                        .help("Path to save to")
                        .index(3)
                        .required(false),
                )
                .arg(
                    Arg::with_name("recursive")
                        .help("Recreate the directory structure -- This can potentially overwrite a lot of files you care about!!")
                        .short("r")
                        .long("recursive")
                        .required(false)
                        .takes_value(false),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("list") {
        let pakfile = matches.value_of("pakfile").unwrap();
        match list_pak_file(pakfile) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Pak file error: {}", e)
            }
        }
    }

    if let Some(matches) = matches.subcommand_matches("extract") {
        let pakfile = matches.value_of("pakfile").unwrap();
        let path = matches.value_of("path").unwrap();
        let mut outfile: &str = path;
        if let Some(option_outfile) = matches.value_of("outfile") {
            outfile = option_outfile;
        }

        let mut recursive = false;
        if matches.is_present("recursive") {
            recursive = true;
        }

        match extract_file_from_pak_to_path(pakfile, path, outfile, recursive) {
            Ok(finalpath) => {
                eprintln!("Extracted: '{}' to '{}'", path, finalpath)
            }
            Err(e) => {
                eprintln!("Pak file error: {}", e)
            }
        }
    }
}

fn extract_file_from_pak_to_path(
    pakfile: &str,
    path: &str,
    outfile: &str,
    recursive: bool,
) -> Result<String, Box<dyn Error>> {
    let pak = Pak::from_file(pakfile.to_string())?;
    match pak.files.iter().find(|pf| pf.name.eq(path)) {
        Some(pakfile) => match pakfile.save_to(outfile.to_string(), recursive) {
            Ok(path) => Ok(path),
            Err(e) => {
                panic!("Pak error! {}", e)
            }
        },
        None => {
            panic!("File not found in PakFile or other error!");
        }
    }
}

fn list_pak_file(pakfile: &str) -> Result<(), Box<dyn Error>> {
    let pak = Pak::from_file(pakfile.to_string())?;
    pak.files.iter().for_each(|i| println!("{}", i.name));
    Ok(())
}
