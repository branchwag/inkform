use inkform_core::{
    FontArtifact, InkformError, InkformErrorKind, PreviewRequest, PreviewResponse, SampleImage,
    SampleQuality, ScriptPack, ValidationReport, generate_font, preview_text, validate_sample,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmGenerationResult {
    pub font_artifact: FontArtifact,
    pub validation_report: ValidationReport,
}

/// Validate sample bytes against the default Latin-extended script pack.
///
/// # Errors
///
/// Returns an error when the provided dimensions or sample bytes are not valid
/// for the core validation pipeline.
pub fn validate_sample_bytes(
    bytes: Vec<u8>,
    width: u32,
    height: u32,
) -> Result<ValidationReport, InkformError> {
    validate_sample(
        &SampleImage {
            width,
            height,
            bytes,
            quality: SampleQuality::Clean,
        },
        &ScriptPack::latin_extended(),
    )
}

/// Generate a font artifact and validation report from raw sample bytes.
///
/// # Errors
///
/// Returns an error when validation fails or the core generation pipeline
/// cannot produce an artifact from the supplied sample.
pub fn generate_font_from_bytes(
    bytes: Vec<u8>,
    width: u32,
    height: u32,
) -> Result<WasmGenerationResult, InkformError> {
    let sample = SampleImage {
        width,
        height,
        bytes,
        quality: SampleQuality::Clean,
    };
    let script_pack = ScriptPack::latin_extended();
    let validation_report = validate_sample(&sample, &script_pack)?;
    let font_artifact = generate_font(&sample, &script_pack)?;

    Ok(WasmGenerationResult {
        font_artifact,
        validation_report,
    })
}

/// Build preview data for text rendered with a generated font artifact.
///
/// # Errors
///
/// Returns an error when the provided preview text is empty or invalid for the
/// underlying preview pipeline.
pub fn preview_generated_text(
    font_artifact: &FontArtifact,
    text: &str,
) -> Result<PreviewResponse, InkformError> {
    if text.trim().is_empty() {
        return Err(InkformError::new(
            InkformErrorKind::InvalidInput,
            "preview text cannot be empty",
        ));
    }

    preview_text(
        font_artifact,
        &PreviewRequest {
            text: String::from(text),
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::{generate_font_from_bytes, preview_generated_text, validate_sample_bytes};

    #[test]
    fn validates_wrapper_input() {
        let report_result = validate_sample_bytes(vec![3; 1024], 1600, 2200);
        assert!(
            report_result.is_ok(),
            "wrapper should validate a clean sample: {report_result:?}"
        );
        let report = match report_result {
            Ok(report) => report,
            Err(error) => panic!("unexpected wrapper validation error: {error}"),
        };

        assert!(report.accepted);
    }

    #[test]
    fn generates_wrapper_result() {
        let result_value = generate_font_from_bytes(vec![1; 2048], 1600, 2200);
        assert!(
            result_value.is_ok(),
            "wrapper should generate a font: {result_value:?}"
        );
        let result = match result_value {
            Ok(result) => result,
            Err(error) => panic!("unexpected wrapper generation error: {error}"),
        };

        assert_eq!(result.font_artifact.script_pack_id, "latin-extended");
    }

    #[test]
    fn previews_text_from_wrapper() {
        let result_value = generate_font_from_bytes(vec![1; 2048], 1600, 2200);
        assert!(
            result_value.is_ok(),
            "wrapper should generate a font: {result_value:?}"
        );
        let result = match result_value {
            Ok(result) => result,
            Err(error) => panic!("unexpected wrapper generation error: {error}"),
        };
        let preview_result = preview_generated_text(&result.font_artifact, "Inkform");
        assert!(
            preview_result.is_ok(),
            "preview should be generated: {preview_result:?}"
        );
        let preview = match preview_result {
            Ok(preview) => preview,
            Err(error) => panic!("unexpected wrapper preview error: {error}"),
        };

        assert_eq!(preview.unsupported_characters.len(), 0);
    }
}
