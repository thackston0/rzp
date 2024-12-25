use clap::Parser;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::Path;
use colored::Colorize;
use rayon::prelude::*;
use zip::result::ZipResult;

#[derive(Parser)]
#[command(name = "rzp", version = "1.1.0", about = "rzp: a fast, multithreaded zip extractor", long_about = None)]
struct Args {
    /// Input files 
    #[arg(required = true)] // Ensure at least one file is provided
    files: Vec<String>,

    /// Extract files
    #[arg(short, long)]
    extract: bool,
    /// List contents (default if no other argument is specified)
    #[arg(short, long)]
    list: bool,
    /// Output path
    #[arg(short, long, default_value = ".")]
    output: String,
    /// Create directories matching the file name for each archive
    #[arg(short, long)]
    create_directories: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    if args.list || (!args.list && !args.extract) {
        // Process files in parallel
        args.files
            .par_iter()
            .for_each(|file| match File::open(file) {
                Ok(f) => {
                    if let Err(e) = list_zip_contents(f, file) {
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
                    if let Err(e) = extract_zip_contents(f, Path::new(&args.output), file, args.create_directories) {
                        eprintln!("Error extracting file {}: {}", file, e);
                    }
                }
                Err(e) => eprintln!("Error opening file {}: {}", file, e),
            });
    }


    Ok(())

}

fn list_zip_contents(reader: impl Read + Seek, file_name: &str) -> zip::result::ZipResult<()> {
    if !archive_is_valid(file_name){
        return Ok(());
    }
    let mut zip = zip::ZipArchive::new(reader)?;
    if zip.is_empty(){
        println!("File is empty");
        return Ok(());
    }
    println!("\n{} contains {} file(s)\n--------", file_name.cyan(), zip.len()); 
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

fn extract_zip_contents(reader: impl Read + Seek, output_dir: &Path, file_name: &str, create_directories: bool) -> ZipResult<()> {
    if !archive_is_valid(file_name){
        return Ok(());
    }
    // Decide if we need to create a subdirectory named after the ZIP file
    let base_output_dir = if create_directories {
        // Safely get just the file stem (e.g., "myarchive" from "myarchive.zip")
        let file_stem = Path::new(file_name)
            .file_stem()
            .unwrap_or_default();
        output_dir.join(file_stem)
    } else {
        output_dir.to_path_buf()
    };


    let mut zip = zip::ZipArchive::new(reader)?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let out_path = base_output_dir.join(file.name());

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

fn archive_is_valid(file_name: &str) -> bool {
    let Some(file_type) = infer::get_from_path(file_name).ok().flatten() else {
        eprintln!("{} {}", file_name.red(), "is not a zip file. Skipping...".red());
        return false;
    };
    if file_type.mime_type() == "application/zip" {
        true
    } else {
        eprintln!("{} {}", file_name.red(), "is an invalid archive. Skipping...".red());
        false
    }
}

