use inkform_core::{
    SampleImage, SampleQuality, ScriptPack, build_preview_svg_with_transcript,
    extract_handwriting_with_transcript, generate_font_with_transcript,
};
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
    let mut arguments = env::args().skip(1);
    let image_path = arguments.next().ok_or_else(|| {
        String::from("usage: cargo run -p inkform-core --example inspect_sample -- <image-path>")
    })?;
    let transcript = arguments.next();

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
    let extracted_component_count =
        extract_handwriting_with_transcript(&sample, None).map_or(0, |result| result.glyphs.len());
    let artifact = generate_font_with_transcript(&sample, &script_pack, transcript.as_deref())
        .map_err(|error| format!("font generation failed for '{image_path}': {error}"))?;
    let face = Face::parse(&artifact.binary, 0)
        .map_err(|error| format!("generated TTF failed to parse for '{image_path}': {error:?}"))?;

    let output_path = String::from("/tmp/inkform-inspect.ttf");
    fs::write(&output_path, &artifact.binary)
        .map_err(|error| format!("could not write '{output_path}': {error}"))?;
    let preview_path = String::from("/tmp/inkform-inspect.svg");
    let preview = build_preview_svg_with_transcript(
        &sample,
        &script_pack,
        &artifact.glyphs,
        "Hello! The quick brown fox jumps over the lazy dog.",
        transcript.as_deref(),
    );
    fs::write(&preview_path, preview)
        .map_err(|error| format!("could not write '{preview_path}': {error}"))?;

    println!("sample={image_path}");
    println!("dimensions={}x{}", sample.width, sample.height);
    println!("glyph_count={}", artifact.glyphs.len());
    println!("anchor_count={}", artifact.anchor_count);
    println!("extracted_component_count={extracted_component_count}");
    if let Some(extraction) = extract_handwriting_with_transcript(&sample, None) {
        for (index, glyph) in extraction.glyphs.iter().enumerate() {
            println!(
                "component={index} width_ratio={:.3} height_ratio={:.3} density={:.3} slant={:.3}",
                glyph.width_ratio, glyph.height_ratio, glyph.density, glyph.slant
            );
        }
    }
    if let Some(transcript) = transcript.as_deref()
        && let Some(extraction) = extract_handwriting_with_transcript(&sample, Some(transcript))
    {
        for (index, glyph) in extraction.glyphs.iter().enumerate() {
            let centerline_points = glyph.centerlines.iter().map(Vec::len).sum::<usize>();
            let centerline_lengths = glyph
                .centerlines
                .iter()
                .map(|path| path.len().to_string())
                .collect::<Vec<_>>()
                .join(",");
            println!(
                "anchor_component={index} character={:?} width_ratio={:.3} density={:.3} centerline_paths={} centerline_points={centerline_points} centerline_lengths=[{centerline_lengths}]",
                glyph.character,
                glyph.width_ratio,
                glyph.density,
                glyph.centerlines.len(),
            );
        }
    }
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
    println!("preview={preview_path}");

    Ok(())
}
