use clap::Parser;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use colored::Colorize;


#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// input files 
    files: Vec<String>,

    /// extract the files
    #[arg(short, long)]
    extract: bool,
    /// list the contents of the files
    #[arg(short, long)]
    list: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut files: Vec<File> = Vec::new();
    for file in &args.files{
        let file = File::open(file)?;
        files.push(file);
    }
    if args.list{ // List files
        for file in &files{
            let _ = list_zip_contents(file);
        }
    }
    Ok(())
}

fn list_zip_contents(reader: impl Read + Seek) -> zip::result::ZipResult<()> {
    let mut zip = zip::ZipArchive::new(reader)?;
    if zip.is_empty(){
        println!("File is empty");
        return Ok(());
    }
    println!("Files: {}\n--------", zip.len()); // TODO: implement more info here, like a title
    for i in 0..zip.len() {
        let file = zip.by_index(i)?;
        if file.is_dir(){
            println!("{}", file.name().blue());
        }
        else if file.is_symlink(){
            println!("{}", file.name().cyan());
        }
        else{
            println!("{} {}", file.name(), format_bytes(file.size()).cyan());
        }
    }

    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = UNITS[0];

    for &current_unit in &UNITS {
        if size < 1024.0 {
            unit = current_unit;
            break;
        }
        size /= 1024.0;
    }

    format!("{:.2} {}", size, unit)
}
