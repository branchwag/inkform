"use client";

import { useMemo, useState } from "react";
import { generateInkformResult, type EngineMode } from "../lib/inkform-engine";
import type { GenerationResult } from "../lib/engine-types";

const starterText = "Grüße aus Inkform";

export function GeneratorWorkbench() {
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [previewText, setPreviewText] = useState(starterText);
  const [result, setResult] = useState<GenerationResult | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [engineMode, setEngineMode] = useState<EngineMode | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);

  const selectedSummary = useMemo(() => {
    if (selectedFile === null) {
      return "No sample selected yet.";
    }

    return `${selectedFile.name} · ${Math.ceil(selectedFile.size / 1024)} KB · ${selectedFile.type || "unknown type"}`;
  }, [selectedFile]);

  function handleFileChange(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0] ?? null;
    setSelectedFile(file);
    setResult(null);
    setErrorMessage(null);
    setEngineMode(null);
  }

  async function handleGenerate() {
    if (selectedFile === null) {
      setErrorMessage("Select a handwriting image before generating a preview.");
      setResult(null);
      return;
    }

    setIsGenerating(true);
    const { engineMode: nextEngineMode, result: nextResult } = await generateInkformResult(
      selectedFile,
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
          Start with a clean photo or scan of your handwriting sheet. Inkform checks the sample,
          builds a matching character set, and prepares a downloadable result.
        </p>
      </div>

      <label style={{ display: "grid", gap: "0.5rem" }}>
        <span>Upload your handwriting sheet</span>
        <input type="file" accept="image/*" onChange={handleFileChange} />
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
            <h3 style={{ marginBottom: "0.5rem", fontSize: "1.8rem" }}>
              {previewText.trim() || starterText}
            </h3>
            <p style={{ margin: 0, color: "var(--muted)", lineHeight: 1.7 }}>
              {result.preview.renderPlan}
            </p>
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
                Save the generated output so you can keep refining it. This is currently a preview
                package while the full font compiler is being completed.
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
                Download my file
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
