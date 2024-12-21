use clap::Parser;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::Path;
use colored::Colorize;
use rayon::prelude::*;
use zip::result::ZipResult;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// input files 
    #[arg(required = true)] // Ensure at least one file is provided
    files: Vec<String>,

    /// extract files
    #[arg(short, long)]
    extract: bool,
    /// list contents (default if no other argument is specified)
    #[arg(short, long)]
    list: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    if args.list || (!args.list && !args.extract) {
        // Process files in parallel
        args.files
            .par_iter()
            .for_each(|file| match File::open(file) {
                Ok(f) => {
                    if let Err(e) = list_zip_contents(f) {
                        eprintln!("Error processing file {}: {}", file, e);
                    }
                }
                Err(e) => eprintln!("Error opening file {}: {}", file, e),
            });
    }

    if args.extract {
        args.files
            .par_iter()
            .for_each(|file| match File::open(file) {
                Ok(f) => {
                    if let Err(e) = extract_zip_contents(f, Path::new(".")) {
                        eprintln!("Error extracting file {}: {}", file, e);
                    }
                }
                Err(e) => eprintln!("Error opening file {}: {}", file, e),
            });
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

fn extract_zip_contents(reader: impl Read + Seek, output_dir: &Path) -> ZipResult<()> {
    let mut zip = zip::ZipArchive::new(reader)?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let out_path = output_dir.join(file.name());

        if file.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&out_path)?;
            io::copy(&mut file, &mut outfile)?;
        }

        // Preserve permissions if possible
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&out_path, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    println!(
        "{}: Extracted {} files",
        "Success".green(),
        zip.len()
    );

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
