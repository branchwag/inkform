use inkform_core::{
    FontArtifact, InkformError, InkformErrorKind, PREVIEW_VERSION, PreviewRequest, PreviewResponse,
    SampleImage, SampleQuality, ScriptPack, ValidationReport, build_preview_svg,
    build_preview_svg_with_transcript, generate_font, generate_font_with_transcript, preview_text,
    validate_sample,
};
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmGenerationResult {
    pub font_artifact: FontArtifact,
    pub validation_report: ValidationReport,
}

/// Generate a JSON payload for browser-side preview and artifact display.
///
/// # Errors
///
/// Returns a JavaScript error when validation, generation, or preview creation
/// fails for the provided sample.
#[wasm_bindgen]
pub fn generate_font_json(
    bytes: Vec<u8>,
    width: u32,
    height: u32,
    preview_text: &str,
    transcript: &str,
) -> Result<String, JsError> {
    let sample = SampleImage {
        width,
        height,
        bytes,
        quality: SampleQuality::Clean,
    };
    let script_pack = ScriptPack::latin_extended();
    let validation_report =
        validate_sample(&sample, &script_pack).map_err(|error| map_to_js_error(&error))?;
    let transcript = (!transcript.trim().is_empty()).then_some(transcript);
    let font_artifact = generate_font_with_transcript(&sample, &script_pack, transcript)
        .map_err(|error| map_to_js_error(&error))?;
    let preview_response = preview_generated_text(&font_artifact, preview_text)
        .map_err(|error| map_to_js_error(&error))?;
    let preview_response = PreviewResponse {
        svg_markup: match transcript {
            Some(transcript) => build_preview_svg_with_transcript(
                &sample,
                &script_pack,
                &font_artifact.glyphs,
                preview_text,
                Some(transcript),
            ),
            None => build_preview_svg(&sample, &script_pack, &font_artifact.glyphs, preview_text),
        },
        ..preview_response
    };

    Ok(build_generation_payload(
        &validation_report,
        &font_artifact,
        &preview_response,
    ))
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

fn map_to_js_error(error: &InkformError) -> JsError {
    JsError::new(error.message())
}

fn build_generation_payload(
    validation_report: &ValidationReport,
    font_artifact: &FontArtifact,
    preview_response: &PreviewResponse,
) -> String {
    let validation_notes = validation_report
        .notes
        .iter()
        .map(|note| format!("\"{}\"", escape_json(note)))
        .collect::<Vec<_>>()
        .join(",");
    let unsupported_characters = preview_response
        .unsupported_characters
        .iter()
        .map(|character| format!("\"{}\"", escape_json(&character.to_string())))
        .collect::<Vec<_>>()
        .join(",");
    let binary_bytes = font_artifact
        .binary
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let binary_hash = simple_binary_hash(&font_artifact.binary);

    format!(
        concat!(
            "{{",
            "\"validation\":{{",
            "\"accepted\":{},",
            "\"notes\":[{}],",
            "\"glyphTargetCount\":{}",
            "}},",
            "\"artifact\":{{",
            "\"familyName\":\"{}\",",
            "\"scriptPackId\":\"{}\",",
            "\"glyphCount\":{},",
            "\"anchorCount\":{},",
            "\"binaryLabel\":\"{}\",",
            "\"binaryHash\":\"{}\",",
            "\"downloadName\":\"{}\",",
            "\"mimeType\":\"{}\",",
            "\"bytes\":[{}]",
            "}},",
            "\"preview\":{{",
            "\"renderPlan\":\"{}\",",
            "\"unsupportedCharacters\":[{}],",
            "\"previewVersion\":\"{}\",",
            "\"svgMarkup\":\"{}\"",
            "}}",
            "}}"
        ),
        validation_report.accepted,
        validation_notes,
        validation_report.glyph_target_count,
        escape_json(&font_artifact.family_name),
        escape_json(&font_artifact.script_pack_id),
        font_artifact.glyphs.len(),
        font_artifact.anchor_count,
        escape_json("inkform-wasm-artifact"),
        escape_json(&binary_hash),
        escape_json("inkform.ttf"),
        escape_json("font/ttf"),
        binary_bytes,
        escape_json(&preview_response.render_plan),
        unsupported_characters,
        escape_json(PREVIEW_VERSION),
        escape_json(&preview_response.svg_markup)
    )
}

fn escape_json(value: &str) -> String {
    value.chars().fold(String::new(), |mut escaped, character| {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(character),
        }

        escaped
    })
}

fn simple_binary_hash(bytes: &[u8]) -> String {
    let hash = bytes
        .iter()
        .enumerate()
        .fold(0_u64, |accumulator, (index, byte)| {
            let rotated = accumulator.rotate_left(5);
            let index_value = u64::try_from(index).unwrap_or(0);
            rotated ^ u64::from(*byte) ^ index_value.wrapping_mul(0x9E37_79B9)
        });
    format!("{hash:016x}")
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
