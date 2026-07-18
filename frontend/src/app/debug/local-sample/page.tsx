"use client";

import { useEffect, useState } from "react";
import { generateInkformResult } from "../../../lib/inkform-engine";
import type { EngineMode } from "../../../lib/inkform-engine";
import type { GenerationResult } from "../../../lib/engine-types";

const previewText = "The quick brown fox jumps over the lazy dog.";

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
              {state.result.preview.svgMarkup.length > 0 ? (
                <div
                  style={{
                    marginTop: "0.75rem",
                    border: "1px solid var(--border)",
                    borderRadius: "18px",
                    overflow: "hidden",
                    background: "#f7efe3"
                  }}
                  dangerouslySetInnerHTML={{ __html: state.result.preview.svgMarkup }}
                />
              ) : (
                <p style={{ marginTop: "0.75rem", color: "#932f1a" }}>SVG preview missing.</p>
              )}
              <p style={{ marginBottom: 0, color: "var(--muted)", lineHeight: 1.7 }}>
                {state.result.preview.renderPlan}
              </p>
            </article>
          </div>
        ) : null}
      </section>
    </main>
  );
}
