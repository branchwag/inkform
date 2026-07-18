use std::env;
use std::error::Error;
use std::fs;
use std::io::{Error as IoError, ErrorKind};

use inkform_core::{SampleImage, SampleQuality, ScriptPack, generate_font};

fn main() -> Result<(), Box<dyn Error>> {
    let output_path = env::args_os().nth(1).ok_or_else(|| {
        IoError::new(
            ErrorKind::InvalidInput,
            "usage: write_validation_font <output-path>",
        )
    })?;
    let sample = SampleImage {
        width: 1600,
        height: 2200,
        bytes: vec![2; 1600],
        quality: SampleQuality::Clean,
    };
    let artifact = generate_font(&sample, &ScriptPack::latin_extended())?;

    fs::write(output_path, artifact.binary)?;
    Ok(())
}
