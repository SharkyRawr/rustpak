use std::error::Error;

use rustpak::Pak;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version = "0.1")]
#[command(author = "Sophie Luna Schumann <me@sophie.lgbt>")]
#[command(about = "Quake/Half-Life Pak file manipulation utility")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List {
        #[arg(help = "Path to .pak file")]
        pakfile: String,
    },
    Extract {
        #[arg(help = "Path to .pak file")]
        pakfile: String,
        #[arg(help = "Filename to extract")]
        path: String,
        #[arg(help = "Path to save to")]
        outfile: Option<String>,
        #[arg(
            short,
            long,
            help = "Recreate the directory structure -- This can potentially overwrite a lot of files you care about!!"
        )]
        recursive: bool,
    },
    Append {
        #[arg(help = "Path to .pak file")]
        pakfile: String,
        #[arg(help = "File to append")]
        path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { pakfile } => match list_pak_file(pakfile) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Pak file error: {}", e)
            }
        },
        Commands::Extract {
            pakfile,
            path,
            outfile,
            recursive,
        } => {
            let outfile = outfile.unwrap_or_else(|| path.clone());
            match extract_file_from_pak_to_path(pakfile, path.clone(), outfile, recursive) {
                Ok(finalpath) => {
                    eprintln!("Extracted: '{}' to '{}'", &path, finalpath)
                }
                Err(e) => {
                    eprintln!("Pak file error: {}", e)
                }
            }
        }
        Commands::Append { pakfile, path } => {
            add_file_to_pak(pakfile, path).unwrap();
        }
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
