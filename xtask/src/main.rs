use clap::Parser;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use xtask::error::XtaskError;
use xtask::gen::firmware::{gen_firmware, Encryption};
use xtask::gen::image::gen_image;
use xtask::{Cli, Command};

/// Main function for the xtask utility.
fn main() -> Result<(), XtaskError> {
    let cli = Cli::parse();
    match cli.command {
        Command::GenFirmware {
            input,
            output,
            encryption,
        } => {
            // Parse encryption type, defaulting to "none" if not specified
            let encryption = encryption
                .unwrap_or("none".to_string())
                .parse::<Encryption>()?;

            // Read input file
            let input_file = Path::new(&input);

            // Check if input file exists
            if !input_file.is_file() {
                panic!("Input file does not exist");
            };

            // Extract file name without extension
            let file_name = input_file.file_name().expect("Failed to get file name");
            let file_name_str = file_name
                .to_str()
                .expect("Failed to convert file name to string");
            let (name, _) = split_file_name(file_name_str);

            // Read input file contents
            let mut input_file = File::open(input_file).expect("Failed to open input file");
            let mut input_data = vec![];
            input_file.read_to_end(&mut input_data)?;

            // Generate firmware
            let firmware = gen_firmware(&input_data, encryption)?;

            // Generate firmware image
            let image = gen_image(&firmware)?;

            // Close input file
            drop(input_file);

            // Determine output file name
            let output_file = output.unwrap_or_else(|| format!("{}.img", name));

            // Write output file
            fs::write(output_file, image)?;

            println!("Firmware generation successful!");
        }
    }
    Ok(())
}

/// Split a file name into its name and extension parts.
fn split_file_name(file_name: &str) -> (&str, Option<&str>) {
    match file_name.rfind('.') {
        Some(dot_index) => (&file_name[..dot_index], Some(&file_name[dot_index + 1..])),
        None => (file_name, None),
    }
}
