"use client";

import Image from "next/image";
import { useEffect, useMemo, useRef, useState } from "react";
import { loadGeneratedBrowserFont } from "../lib/browser-font-preview";
import { generateInkformResult, type EngineMode } from "../lib/inkform-engine";
import type { GenerationResult } from "../lib/engine-types";

const starterText = "The quick brown fox jumps over the lazy dog.";
const currentPreviewVersion = "svg-v3";

export function GeneratorWorkbench() {
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [previewText, setPreviewText] = useState(starterText);
  const [result, setResult] = useState<GenerationResult | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [engineMode, setEngineMode] = useState<EngineMode | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);
  const [previewFontFamily, setPreviewFontFamily] = useState<string | null>(null);
  const [previewFontState, setPreviewFontState] = useState<"idle" | "loaded" | "failed">("idle");
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  const selectedSummary = useMemo(() => {
    if (selectedFile === null) {
      return "No sample selected yet.";
    }

    return `${selectedFile.name} · ${Math.ceil(selectedFile.size / 1024)} KB · ${selectedFile.type || "unknown type"}`;
  }, [selectedFile]);
  const hasCurrentSvgPreview =
    result !== null &&
    result.preview.previewVersion === currentPreviewVersion &&
    result.preview.svgMarkup.includes("<svg");
  const shouldShowSvgFallback = hasCurrentSvgPreview && previewFontState !== "loaded";
  const previewSvgDataUrl = useMemo(() => {
    if (!shouldShowSvgFallback || result === null) {
      return null;
    }

    return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(result.preview.svgMarkup)}`;
  }, [result, shouldShowSvgFallback]);

  function handleFileChange(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0] ?? null;
    setSelectedFile(file);
    setResult(null);
    setErrorMessage(null);
    setEngineMode(null);
    setPreviewFontFamily(null);
    setPreviewFontState("idle");
  }

  async function handleGenerate() {
    const activeFile = selectedFile ?? fileInputRef.current?.files?.[0] ?? null;
    if (activeFile === null) {
      setErrorMessage("Select a handwriting image before generating a preview.");
      setResult(null);
      return;
    }

    if (selectedFile === null) {
      setSelectedFile(activeFile);
    }

    setIsGenerating(true);
    setPreviewFontFamily(null);
    setPreviewFontState("idle");
    const { engineMode: nextEngineMode, result: nextResult } = await generateInkformResult(
      activeFile,
      previewText
    );
    setEngineMode(nextEngineMode);
    if (!nextResult.validation.accepted) {
      setErrorMessage(nextResult.validation.notes.at(-1) ?? "The sample could not be accepted.");
      setResult(nextResult);
      setIsGenerating(false);
      return;
    }

    setErrorMessage(null);
    setResult(nextResult);
    setIsGenerating(false);
  }

  function handleDownload() {
    if (result === null) {
      return;
    }

    const blob = new Blob([Uint8Array.from(result.artifact.bytes)], {
      type: result.artifact.mimeType
    });
    const objectUrl = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = objectUrl;
    anchor.download = result.artifact.downloadName;
    anchor.click();
    URL.revokeObjectURL(objectUrl);
  }

  useEffect(() => {
    if (
      result === null ||
      result.artifact.mimeType !== "font/ttf"
    ) {
      return;
    }

    let cancelled = false;
    let cleanupPreviewFont = () => {};
    let loadedFontFace: FontFace | null = null;

    void loadGeneratedBrowserFont(result.artifact)
      .then(({ cleanup, familyName, fontFace }) => {
        if (cancelled) {
          cleanup();
          return;
        }

        cleanupPreviewFont = cleanup;
        loadedFontFace = fontFace;
        document.fonts.add(fontFace);
        setPreviewFontFamily(familyName);
        setPreviewFontState("loaded");
      })
      .catch(() => {
        if (!cancelled) {
          setPreviewFontFamily(null);
          setPreviewFontState("failed");
        }
      });

    return () => {
      cancelled = true;
      if (loadedFontFace !== null) {
        document.fonts.delete(loadedFontFace);
      }
      cleanupPreviewFont();
    };
  }, [result]);

  return (
    <section
      style={{
        display: "grid",
        gap: "1.25rem",
        background: "var(--surface)",
        border: "1px solid var(--border)",
        borderRadius: "28px",
        boxShadow: "0 24px 60px var(--shadow)",
        padding: "1.5rem"
      }}
    >
      <div>
        <p
          style={{
            margin: 0,
            letterSpacing: "0.18em",
            textTransform: "uppercase",
            color: "var(--accent-deep)",
            fontSize: "0.78rem"
          }}
        >
          Create your font
        </p>
        <h2 style={{ marginBottom: "0.5rem" }}>
          Upload your handwriting and preview what it could become.
        </h2>
        <p style={{ color: "var(--muted)", lineHeight: 1.7, margin: 0 }}>
          Start with any clear photo or scan of your handwriting. Inkform checks the sample,
          pulls out stroke information, and prepares a downloadable result.
        </p>
      </div>

      <label style={{ display: "grid", gap: "0.5rem" }}>
        <span>Upload a handwriting photo</span>
        <input ref={fileInputRef} type="file" accept="image/*" onChange={handleFileChange} />
      </label>

      <p style={{ margin: 0, color: "var(--muted)" }}>{selectedSummary}</p>

      <label style={{ display: "grid", gap: "0.5rem" }}>
        <span>Preview text</span>
        <textarea
          value={previewText}
          onChange={(event) => setPreviewText(event.target.value)}
          rows={4}
          style={{
            borderRadius: "18px",
            border: "1px solid var(--border)",
            padding: "0.9rem 1rem",
            background: "var(--surface-strong)"
          }}
        />
      </label>

      <div style={{ display: "flex", gap: "1rem", alignItems: "center", flexWrap: "wrap" }}>
        <button
          type="button"
          onClick={handleGenerate}
          disabled={isGenerating}
          style={{
            padding: "0.9rem 1.2rem",
            borderRadius: "999px",
            background: "var(--accent)",
            color: "#fff8f2",
            border: 0,
            cursor: isGenerating ? "progress" : "pointer",
            opacity: isGenerating ? 0.72 : 1
          }}
        >
          {isGenerating ? "Creating your preview..." : "Create my font preview"}
        </button>
        {errorMessage ? <span style={{ color: "#932f1a" }}>{errorMessage}</span> : null}
      </div>

      {result ? (
        <div style={{ display: "grid", gap: "1rem" }}>
          <p style={{ margin: 0, color: "var(--muted)", fontSize: "0.95rem" }}>
            Rendering mode: {engineMode === "wasm" ? "Rust engine" : "Compatibility fallback"}
          </p>
          <article
            style={{
              padding: "1.25rem",
              borderRadius: "20px",
              border: "1px solid var(--border)",
              background: "var(--surface-strong)"
            }}
          >
            <p
              style={{
                margin: 0,
                letterSpacing: "0.14em",
                textTransform: "uppercase",
                color: "var(--accent-deep)",
                fontSize: "0.72rem"
              }}
            >
              Preview
            </p>
            {previewFontState === "loaded" ? (
              <h3
                style={{
                  marginBottom: "0.75rem",
                  fontSize: "1.8rem",
                  fontFamily:
                    previewFontFamily === null
                      ? 'Georgia, "Times New Roman", serif'
                      : `"${previewFontFamily}", Georgia, "Times New Roman", serif`
                }}
              >
                {previewText.trim() || starterText}
              </h3>
            ) : hasCurrentSvgPreview ? (
              <div
                style={{
                  marginTop: "0.75rem",
                  marginBottom: "0.75rem",
                  border: "1px solid var(--border)",
                  borderRadius: "18px",
                  overflow: "hidden",
                  background: "#f7efe3"
                }}
              >
                {previewSvgDataUrl !== null ? (
                  <Image
                    src={previewSvgDataUrl}
                    alt="Inkform preview"
                    unoptimized
                    width={1200}
                    height={320}
                    style={{ display: "block", width: "100%", height: "auto" }}
                  />
                ) : null}
              </div>
            ) : (
              <h3
                style={{
                  marginBottom: "0.75rem",
                  fontSize: "1.8rem",
                  fontFamily:
                    previewFontFamily === null
                      ? 'Georgia, "Times New Roman", serif'
                      : `"${previewFontFamily}", Georgia, "Times New Roman", serif`
                }}
              >
                {previewText.trim() || starterText}
              </h3>
            )}
            <p style={{ margin: 0, color: "var(--muted)", lineHeight: 1.7 }}>
              {result.preview.renderPlan}
            </p>
            {result.preview.previewVersion !== currentPreviewVersion ? (
              <p style={{ marginBottom: 0, color: "#8a5b2d", lineHeight: 1.7 }}>
                This preview result came from an older in-memory build. Generate the preview again
                to refresh it with the current renderer.
              </p>
            ) : null}
            {!hasCurrentSvgPreview && previewFontState === "failed" ? (
              <p style={{ marginBottom: 0, color: "#932f1a", lineHeight: 1.7 }}>
                The generated font file could not be loaded into the browser preview. Downloading
                may still work, but this on-page preview is currently falling back to the system
                serif font.
              </p>
            ) : null}
            {result.preview.unsupportedCharacters.length > 0 ? (
              <p style={{ marginBottom: 0, color: "var(--muted)" }}>
                Some characters still need extra support:{" "}
                {result.preview.unsupportedCharacters.join(", ")}
              </p>
            ) : null}
          </article>

          <div
            style={{
              display: "grid",
              gap: "1rem",
              gridTemplateColumns: "repeat(auto-fit, minmax(220px, 1fr))"
            }}
          >
            <article
              style={{
                padding: "1rem",
                borderRadius: "18px",
                border: "1px solid var(--border)",
                background: "var(--surface-strong)"
              }}
            >
              <h3 style={{ marginTop: 0 }}>What Inkform found</h3>
              <p style={{ marginTop: 0, color: "var(--muted)" }}>
                {result.validation.accepted
                  ? "Your sample looks ready to work with."
                  : "Your sample needs another pass."}
              </p>
              <ul style={{ margin: 0, paddingLeft: "1.2rem", lineHeight: 1.7 }}>
                {result.validation.notes.map((note) => (
                  <li key={note}>{note}</li>
                ))}
              </ul>
            </article>

            <article
              style={{
                padding: "1rem",
                borderRadius: "18px",
                border: "1px solid var(--border)",
                background: "var(--surface-strong)"
              }}
            >
              <h3 style={{ marginTop: 0 }}>Download</h3>
              <p style={{ marginTop: 0, color: "var(--muted)", lineHeight: 1.7 }}>
                {result.artifact.mimeType === "font/ttf"
                  ? "Save your generated font as a TrueType file."
                  : "Save the generated preview package so you can keep refining the result."}
              </p>
              <button
                type="button"
                onClick={handleDownload}
                style={{
                  padding: "0.85rem 1.1rem",
                  borderRadius: "999px",
                  background: "var(--accent)",
                  color: "#fff8f2",
                  border: 0,
                  cursor: "pointer"
                }}
              >
                {result.artifact.mimeType === "font/ttf" ? "Download my font" : "Download preview file"}
              </button>
              <p style={{ margin: "0.85rem 0 0", color: "var(--muted)", fontSize: "0.95rem" }}>
                File name: {result.artifact.downloadName}
              </p>
            </article>
          </div>

          {engineMode === "fallback" ? (
            <p style={{ margin: 0, color: "var(--muted)", fontSize: "0.95rem" }}>
              Inkform used a fallback preview path this time. If the Rust build has already been
              generated, restarting the dev server should switch this to the browser-side Rust engine.
            </p>
          ) : null}
          {engineMode === "wasm" ? (
            <p style={{ margin: 0, color: "var(--muted)", fontSize: "0.95rem" }}>
              This preview was generated with the browser-side Rust engine.
            </p>
          ) : null}
        </div>
      ) : null}
    </section>
  );
}
