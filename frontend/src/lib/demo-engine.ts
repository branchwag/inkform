import type { GenerationResult, ValidationReport } from "./engine-types";

const LATIN_EXTENDED_GLYPH_TARGET = 119;
const PREVIEW_VERSION = "svg-v2";

export function validateUpload(file: File): ValidationReport {
  const notes = [
    `Prepared ${LATIN_EXTENDED_GLYPH_TARGET} target glyph slots for Latin Extended.`,
    "A local fallback preview was used because the Rust/WASM module was unavailable."
  ];

  if (file.size === 0) {
    return {
      accepted: false,
      notes: [...notes, "The selected file is empty."],
      glyphTargetCount: LATIN_EXTENDED_GLYPH_TARGET
    };
  }

  if (!file.type.startsWith("image/")) {
    return {
      accepted: false,
      notes: [...notes, "Please upload an image file rather than a document or archive."],
      glyphTargetCount: LATIN_EXTENDED_GLYPH_TARGET
    };
  }

  if (file.size < 32_000) {
    notes.push("The sample is small; generation may succeed, but the result should be reviewed.");
  }

  return {
    accepted: true,
    notes,
    glyphTargetCount: LATIN_EXTENDED_GLYPH_TARGET
  };
}

export function generateDemoResult(file: File, previewText: string): GenerationResult {
  const validation = validateUpload(file);
  const normalizedPreview = previewText.trim() || "Grüße aus Inkform";
  const unsupportedCharacters = [...normalizedPreview].filter((character) => {
    return character.codePointAt(0) === undefined || character.codePointAt(0)! > 0x017f;
  });

  return {
    validation,
    artifact: {
      familyName: "Inkform Preview",
      scriptPackId: "latin-extended",
      glyphCount: validation.glyphTargetCount,
      binaryLabel: `inkform-demo-${file.name.replace(/\s+/g, "-").toLowerCase()}`,
      binaryHash: `${file.size.toString(16)}-${normalizedPreview.length.toString(16)}`,
      downloadName: "inkform-preview-package.txt",
      mimeType: "text/plain;charset=utf-8",
      bytes: Array.from(
        new TextEncoder().encode(
          [
            "Inkform preview package",
            `source=${file.name}`,
            `glyph_count=${validation.glyphTargetCount}`,
            `preview=${normalizedPreview}`
          ].join("\n")
        )
      )
    },
    preview: {
      renderPlan: `Preview '${normalizedPreview}' with ${validation.glyphTargetCount} glyph targets.`,
      unsupportedCharacters,
      previewVersion: PREVIEW_VERSION,
      svgMarkup: ""
    }
  };
}
