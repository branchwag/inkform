"use client";

import { buildTemplateSvg } from "../lib/script-pack";

type TemplateDownloadProps = {
  label?: string;
};

export function TemplateDownload({ label = "Download handwriting sheet" }: TemplateDownloadProps) {
  function handleDownload() {
    const svg = buildTemplateSvg();
    const blob = new Blob([svg], { type: "image/svg+xml;charset=utf-8" });
    const objectUrl = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = objectUrl;
    anchor.download = "inkform-handwriting-sheet.svg";
    anchor.click();
    URL.revokeObjectURL(objectUrl);
  }

  return (
    <button
      type="button"
      onClick={handleDownload}
      style={{
        padding: "0.9rem 1.2rem",
        borderRadius: "999px",
        border: "1px solid var(--border)",
        background: "var(--surface-strong)",
        color: "var(--foreground)",
        cursor: "pointer"
      }}
    >
      {label}
    </button>
  );
}
