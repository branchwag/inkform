"use client";

import Image from "next/image";
import { useEffect, useMemo, useState } from "react";
import { loadGeneratedBrowserFont } from "../../../lib/browser-font-preview";
import { generateInkformResult } from "../../../lib/inkform-engine";
import type { EngineMode } from "../../../lib/inkform-engine";
import type { GenerationResult } from "../../../lib/engine-types";

const previewText = "The quick brown fox jumps over the lazy dog.";
const currentPreviewVersion = "svg-v3";

type DebugState =
  | { status: "loading" }
  | { status: "error"; message: string }
  | {
      status: "ready";
      engineMode: EngineMode;
      fileName: string;
      result: GenerationResult;
    };

export default function LocalSampleDebugPage() {
  const [state, setState] = useState<DebugState>({ status: "loading" });
  const [previewFontFamily, setPreviewFontFamily] = useState<string | null>(null);
  const [previewFontState, setPreviewFontState] = useState<"idle" | "loaded" | "failed">("idle");
  const [previewFontError, setPreviewFontError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function run() {
      try {
        const response = await fetch("/api/dev/local-sample", { cache: "no-store" });
        if (!response.ok) {
          throw new Error(`Debug sample request failed with ${response.status}.`);
        }

        const fileName = response.headers.get("X-Inkform-Debug-File") ?? "local-sample.jpg";
        const blob = await response.blob();
        const file = new File([blob], fileName, { type: blob.type || "image/jpeg" });
        const { engineMode, result } = await generateInkformResult(file, previewText);

        if (!cancelled) {
          setState({
            status: "ready",
            engineMode,
            fileName,
            result
          });
        }
      } catch (error) {
        if (!cancelled) {
          setState({
            status: "error",
            message: error instanceof Error ? error.message : "Unknown debug failure."
          });
        }
      }
    }

    void run();

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (state.status !== "ready" || state.result.artifact.mimeType !== "font/ttf") {
      return;
    }

    let cancelled = false;
    let cleanupPreviewFont = () => {};
    let loadedFontFace: FontFace | null = null;

    void loadGeneratedBrowserFont(state.result.artifact)
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
      .catch((error) => {
        if (!cancelled) {
          setPreviewFontFamily(null);
          setPreviewFontState("failed");
          setPreviewFontError(
            error instanceof Error ? error.message : "Unknown browser font load failure."
          );
        }
      });

    return () => {
      cancelled = true;
      if (loadedFontFace !== null) {
        document.fonts.delete(loadedFontFace);
      }
      cleanupPreviewFont();
    };
  }, [state]);

  const previewSvgDataUrl = useMemo(() => {
    if (
      state.status !== "ready" ||
      state.result.preview.previewVersion !== currentPreviewVersion ||
      !state.result.preview.svgMarkup.includes("<svg") ||
      previewFontState === "loaded"
    ) {
      return null;
    }

    return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(state.result.preview.svgMarkup)}`;
  }, [previewFontState, state]);

  return (
    <main
      style={{
        maxWidth: "1120px",
        margin: "0 auto",
        padding: "3rem 1.5rem 5rem",
        display: "grid",
        gap: "1.5rem"
      }}
    >
      <section
        style={{
          background: "var(--surface)",
          border: "1px solid var(--border)",
          borderRadius: "28px",
          boxShadow: "0 24px 60px var(--shadow)",
          padding: "1.5rem"
        }}
      >
        <p
          style={{
            margin: 0,
            letterSpacing: "0.18em",
            textTransform: "uppercase",
            color: "var(--accent-deep)",
            fontSize: "0.78rem"
          }}
        >
          Local Debug
        </p>
        <h1 style={{ marginBottom: "0.5rem" }}>Inkform auto-runs against the latest Downloads image.</h1>
        {state.status === "loading" ? <p style={{ margin: 0 }}>Generating preview...</p> : null}
        {state.status === "error" ? <p style={{ margin: 0, color: "#932f1a" }}>{state.message}</p> : null}
        {state.status === "ready" ? (
          <div style={{ display: "grid", gap: "1rem" }}>
            <p style={{ margin: 0 }}>
              File: <strong>{state.fileName}</strong>
              {" · "}
              Engine: <strong>{state.engineMode}</strong>
              {" · "}
              Preview version: <strong>{state.result.preview.previewVersion}</strong>
              {" · "}
              Font preview: <strong>{previewFontState}</strong>
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
                <h2
                  style={{
                    marginTop: "0.75rem",
                    marginBottom: "0.75rem",
                    fontSize: "clamp(2rem, 4vw, 3rem)",
                    fontWeight: 400,
                    fontFamily:
                      previewFontFamily === null
                        ? 'Georgia, "Times New Roman", serif'
                        : `"${previewFontFamily}", Georgia, "Times New Roman", serif`
                  }}
                >
                  {previewText}
                </h2>
              ) : previewSvgDataUrl !== null ? (
                <div
                  style={{
                    marginTop: "0.75rem",
                    border: "1px solid var(--border)",
                    borderRadius: "18px",
                    overflow: "hidden",
                    background: "#f7efe3"
                  }}
                >
                  <Image
                    src={previewSvgDataUrl}
                    alt="Inkform preview"
                    unoptimized
                    width={1200}
                    height={900}
                    style={{ display: "block", width: "100%", height: "auto" }}
                  />
                </div>
              ) : (
                <p style={{ marginTop: "0.75rem", color: "#932f1a" }}>SVG preview missing.</p>
              )}
              <p style={{ marginBottom: 0, color: "var(--muted)", lineHeight: 1.7 }}>
                {state.result.preview.renderPlan}
              </p>
              {previewFontError !== null ? (
                <p style={{ marginBottom: 0, color: "#932f1a", lineHeight: 1.7 }}>
                  Browser font load error: {previewFontError}
                </p>
              ) : null}
            </article>
          </div>
        ) : null}
      </section>
    </main>
  );
}
