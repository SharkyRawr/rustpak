use std::error::Error;

use rustpak::Pak;

use clap::{Arg, Command};

fn main() {
    let matches = Command::new("Pak")
        .version("0.1")
        .author("Sophie Luna Schumann <me@sophie.lgbt>")
        .about("Quake/Half-Life Pak file manipulation utility")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("list")
                .about("List files inside .pak")
                .arg(
                    Arg::new("pakfile")
                        .help("Path to .pak file")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("extract")
                .about("Extract files from Pak")
                .arg(
                    Arg::new("pakfile")
                        .help("Path to .pak file")
                        .index(1)
                        .required(true),
                )
                .arg(
                    Arg::new("path")
                        .help("Filename to extract")
                        .index(2)
                        .required(true),
                )
                .arg(
                    Arg::new("outfile")
                        .help("Path to save to")
                        .index(3)
                        .required(false),
                )
                .arg(
                    Arg::new("recursive")
                        .help("Recreate the directory structure -- This can potentially overwrite a lot of files you care about!!")
                        .short('r')
                        .long("recursive")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
        Command::new("append")
            .about("Append files to Pak")
            .arg(
                Arg::new("pakfile")
                    .help("Path to .pak file")
                    .index(1)
                    .required(true),
            )
            .arg(
                Arg::new("path")
                    .help("File to append")
                    .index(2)
                    .required(true),
            ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("list") {
        let pakfile = matches.get_one::<String>("pakfile").unwrap();
        match list_pak_file(pakfile.to_string()) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Pak file error: {}", e)
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("extract") {
        let pakfile = matches.get_one::<String>("pakfile").unwrap().to_string();
        let path = matches.get_one::<String>("path").unwrap().to_string();
        let mut outfile = path.to_string();
        if let Some(option_outfile) = matches.get_one::<String>("outfile") {
            outfile = option_outfile.clone().to_string();
        }

        let recursive = matches.get_flag("recursive");

        match extract_file_from_pak_to_path(pakfile, path.clone(), outfile, recursive) {
            Ok(finalpath) => {
                eprintln!("Extracted: '{}' to '{}'", &path, finalpath)
            }
            Err(e) => {
                eprintln!("Pak file error: {}", e)
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("append") {
        add_file_to_pak(
            matches.get_one::<String>("pakfile").unwrap().to_string(),
            matches.get_one::<String>("path").unwrap().to_string(),
        )
        .unwrap();
    }
}

fn extract_file_from_pak_to_path(
    pakfile: String,
    path: String,
    outfile: String,
    recursive: bool,
) -> Result<String, Box<dyn Error>> {
    let pak = Pak::from_file(pakfile)?;
    match pak.files.iter().find(|pf| pf.name.eq(&path)) {
        Some(pakfile) => match pakfile.save_to(outfile.to_string(), recursive) {
            Ok(path) => Ok(path),
            Err(e) => Err(Box::new(e)),
        },
        None => Err("File not found in PakFile or other error!".into()),
    }
}

fn list_pak_file(pakfile: String) -> Result<(), Box<dyn Error>> {
    let pak = Pak::from_file(pakfile)?;
    pak.files
        .iter()
        .for_each(|i| println!("{} - {} bytes", i.name, i.size));
    Ok(())
}

fn add_file_to_pak(pakpath: String, filepath: String) -> Result<(), Box<dyn Error>> {
    let mut pak = Pak::from_file(pakpath.clone())?;
    pak.append_file(filepath.clone(), filepath)?;
    pak.save(pakpath)
}
