use crate::domain::{FontArtifact, GlyphCandidate, SampleImage, ScriptPack};
use crate::error::{InkformError, InkformErrorKind};
use crate::extraction::extract_handwriting_with_transcript;
use crate::ttf::{build_ttf, build_ttf_with_transcript};
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
    generate_font_with_transcript(sample_image, script_pack, None)
}

/// Generate a font artifact while mapping confirmed sample text to glyph anchors.
///
/// # Errors
///
/// Returns an error when validation fails or when the generation pipeline
/// cannot continue after validation.
pub fn generate_font_with_transcript(
    sample_image: &SampleImage,
    script_pack: &ScriptPack,
    transcript: Option<&str>,
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
    let anchor_count = transcript
        .and_then(|value| extract_handwriting_with_transcript(sample_image, Some(value)))
        .map_or(0, |result| {
            result
                .glyphs
                .iter()
                .filter(|glyph| glyph.character.is_some())
                .count()
        });
    let family_name = format!("Inkform-{:08X}", sample_identity(sample_image));

    let binary = match transcript {
        Some(transcript) if !transcript.trim().is_empty() => build_ttf_with_transcript(
            &family_name,
            sample_image,
            script_pack,
            &glyphs,
            Some(transcript),
        ),
        _ => build_ttf(&family_name, sample_image, script_pack, &glyphs),
    };

    Ok(FontArtifact {
        family_name,
        script_pack_id: script_pack.id.clone(),
        glyphs,
        anchor_count,
        binary,
    })
}

fn sample_identity(sample_image: &SampleImage) -> u32 {
    let mut hash = 0x811C_9DC5_u32;

    for byte in sample_image
        .width
        .to_be_bytes()
        .into_iter()
        .chain(sample_image.height.to_be_bytes())
        .chain(sample_image.bytes.iter().copied())
    {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }

    hash
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
