use crate::domain::{
    FontArtifact, GenerationReport, GlyphCandidate, ProcessingStage, SampleImage, ScriptPack,
};
use crate::error::{InkformError, InkformErrorKind};
use crate::validation::validate_sample;

/// Generate a font artifact from a validated handwriting sample.
///
/// # Errors
///
/// Returns an error when validation fails or when the generation pipeline
/// cannot continue after validation.
pub fn generate_font(
    sample_image: &SampleImage,
    script_pack: &ScriptPack,
) -> Result<FontArtifact, InkformError> {
    let validation_report = validate_sample(sample_image, script_pack)?;

    if !validation_report.accepted {
        return Err(InkformError::new(
            InkformErrorKind::GenerationFailure,
            "sample did not pass validation",
        ));
    }

    let glyphs = script_pack
        .glyphs
        .iter()
        .map(|character| GlyphCandidate {
            character: *character,
            confidence_percent: confidence_for(*character, sample_image.bytes.len()),
        })
        .collect::<Vec<_>>();

    let generation_report = GenerationReport {
        stage: ProcessingStage::Assemble,
        accepted_glyphs: glyphs.len(),
        warnings: if sample_image.bytes.len() < 1024 {
            vec![String::from(
                "Sample byte payload is unusually small for a guided sheet.",
            )]
        } else {
            Vec::new()
        },
    };

    let binary = build_placeholder_font_binary(script_pack, &generation_report);

    Ok(FontArtifact {
        family_name: String::from("Inkform Preview"),
        script_pack_id: script_pack.id.clone(),
        glyphs,
        binary,
    })
}

fn confidence_for(character: char, sample_size: usize) -> u8 {
    let base = if character.is_ascii_uppercase() {
        96
    } else if character.is_ascii_lowercase() {
        94
    } else if character.is_numeric() {
        93
    } else {
        90
    };

    if sample_size >= 2048 {
        base
    } else {
        base.saturating_sub(4)
    }
}

fn build_placeholder_font_binary(
    script_pack: &ScriptPack,
    generation_report: &GenerationReport,
) -> Vec<u8> {
    format!(
        "INKFORM:{}:{}:{}",
        script_pack.id, generation_report.stage as u8, generation_report.accepted_glyphs
    )
    .into_bytes()
}
