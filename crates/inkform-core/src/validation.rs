use crate::domain::{ProcessingStage, SampleImage, SampleQuality, ScriptPack, ValidationReport};
use crate::error::{InkformError, InkformErrorKind};

/// Validate whether a sample image is suitable for the current script pack.
///
/// # Errors
///
/// Returns an error when the sample dimensions are too small, the image payload
/// is empty, the script pack is empty, or the sample quality is incomplete.
pub fn validate_sample(
    sample_image: &SampleImage,
    script_pack: &ScriptPack,
) -> Result<ValidationReport, InkformError> {
    if sample_image.width < 512 || sample_image.height < 512 {
        return Err(InkformError::new(
            InkformErrorKind::InvalidInput,
            "sample image resolution is too small",
        ));
    }

    if sample_image.bytes.is_empty() {
        return Err(InkformError::new(
            InkformErrorKind::InvalidInput,
            "sample image cannot be empty",
        ));
    }

    if script_pack.glyphs.is_empty() {
        return Err(InkformError::new(
            InkformErrorKind::InvalidInput,
            "script pack must contain at least one glyph",
        ));
    }

    let accepted = sample_image.quality != SampleQuality::Incomplete;
    let mut notes = vec![format!(
        "Targeting {} glyphs for script pack {}.",
        script_pack.glyph_count(),
        script_pack.display_name
    )];

    if sample_image.quality == SampleQuality::Noisy {
        notes.push(String::from(
            "Input quality is noisy; result should still be reviewed before export.",
        ));
    }

    if !accepted {
        return Err(InkformError::new(
            InkformErrorKind::LowQualitySample,
            "sample is incomplete and cannot be processed",
        ));
    }

    Ok(ValidationReport {
        accepted,
        stage: ProcessingStage::Ingest,
        notes,
        glyph_target_count: script_pack.glyph_count(),
    })
}
