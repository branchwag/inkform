use inkform_core::{SampleImage, SampleQuality, ScriptPack, generate_font};
use std::env;
use std::fs;
use ttf_parser::{Face, OutlineBuilder};

#[allow(clippy::struct_field_names)]
#[derive(Default)]
struct CountingBuilder {
    move_count: usize,
    line_count: usize,
    quad_count: usize,
    curve_count: usize,
    close_count: usize,
}

impl OutlineBuilder for CountingBuilder {
    fn move_to(&mut self, _x: f32, _y: f32) {
        self.move_count += 1;
    }

    fn line_to(&mut self, _x: f32, _y: f32) {
        self.line_count += 1;
    }

    fn quad_to(&mut self, _x1: f32, _y1: f32, _x: f32, _y: f32) {
        self.quad_count += 1;
    }

    fn curve_to(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _x: f32, _y: f32) {
        self.curve_count += 1;
    }

    fn close(&mut self) {
        self.close_count += 1;
    }
}

fn main() -> Result<(), String> {
    let image_path = env::args().nth(1).ok_or_else(|| {
        String::from("usage: cargo run -p inkform-core --example inspect_sample -- <image-path>")
    })?;

    let bytes = fs::read(&image_path)
        .map_err(|error| format!("could not read sample image '{image_path}': {error}"))?;
    let dimensions = image::image_dimensions(&image_path)
        .map_err(|error| format!("could not read image dimensions for '{image_path}': {error}"))?;

    let sample = SampleImage {
        width: dimensions.0,
        height: dimensions.1,
        bytes,
        quality: SampleQuality::Clean,
    };
    let script_pack = ScriptPack::latin_extended();
    let artifact = generate_font(&sample, &script_pack)
        .map_err(|error| format!("font generation failed for '{image_path}': {error}"))?;
    let face = Face::parse(&artifact.binary, 0)
        .map_err(|error| format!("generated TTF failed to parse for '{image_path}': {error:?}"))?;

    let output_path = String::from("/tmp/inkform-inspect.ttf");
    fs::write(&output_path, &artifact.binary)
        .map_err(|error| format!("could not write '{output_path}': {error}"))?;

    println!("sample={image_path}");
    println!("dimensions={}x{}", sample.width, sample.height);
    println!("glyph_count={}", artifact.glyphs.len());
    println!("binary_size={}", artifact.binary.len());
    println!("ttf_glyph_count={}", face.number_of_glyphs());
    println!("has_A={}", face.glyph_index('A').is_some());
    println!("has_ssharp={}", face.glyph_index('ß').is_some());
    for character in ['A', 'a', 'g', 'ß'] {
        let glyph_id = face
            .glyph_index(character)
            .ok_or_else(|| format!("missing glyph for '{character}'"))?;
        let bbox = face.glyph_bounding_box(glyph_id);
        let mut builder = CountingBuilder::default();
        let outlined = face.outline_glyph(glyph_id, &mut builder);
        println!(
            "glyph={character} bbox={bbox:?} outline_present={} moves={} lines={} quads={} cubics={} closes={}",
            outlined.is_some(),
            builder.move_count,
            builder.line_count,
            builder.quad_count,
            builder.curve_count,
            builder.close_count
        );
    }
    println!("output={output_path}");

    Ok(())
}
