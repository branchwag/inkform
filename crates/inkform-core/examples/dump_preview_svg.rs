use inkform_core::{GlyphCandidate, SampleImage, SampleQuality, ScriptPack, build_preview_svg};
use std::env;
use std::fs;

fn main() -> Result<(), String> {
    let image_path = env::args()
        .nth(1)
        .ok_or_else(|| String::from("usage: cargo run -p inkform-core --example dump_preview_svg -- <image-path> [preview-text]"))?;
    let preview_text = env::args()
        .nth(2)
        .unwrap_or_else(|| String::from("The quick brown fox jumps over the lazy dog."));

    let bytes = fs::read(&image_path)
        .map_err(|error| format!("could not read sample image '{image_path}': {error}"))?;
    let (width, height) = image::image_dimensions(&image_path)
        .map_err(|error| format!("could not read image dimensions for '{image_path}': {error}"))?;

    let sample = SampleImage {
        width,
        height,
        bytes,
        quality: SampleQuality::Clean,
    };
    let script_pack = ScriptPack::latin_extended();
    let glyphs = script_pack
        .glyphs
        .iter()
        .map(|character| GlyphCandidate {
            character: *character,
            confidence_percent: 100,
        })
        .collect::<Vec<_>>();
    let svg_markup = build_preview_svg(&sample, &script_pack, &glyphs, &preview_text);
    print!("{svg_markup}");

    Ok(())
}
