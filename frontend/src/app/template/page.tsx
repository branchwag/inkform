import Link from "next/link";
import { TemplateDownload } from "../../components/template-download";
import { chunkGlyphs, latinExtendedGlyphs, templateColumns } from "../../lib/script-pack";

const rows = chunkGlyphs(latinExtendedGlyphs, templateColumns);

export default function TemplatePage() {
  return (
    <main style={{ maxWidth: "1180px", margin: "0 auto", padding: "3rem 1.5rem 4rem" }}>
      <section
        style={{
          background: "var(--surface)",
          border: "1px solid var(--border)",
          borderRadius: "28px",
          boxShadow: "0 24px 60px var(--shadow)",
          padding: "2rem"
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
          Handwriting sheet
        </p>
        <h1 style={{ margin: "0.75rem 0 1rem", fontSize: "clamp(2rem, 5vw, 3.5rem)" }}>
          Use the Inkform template for cleaner extraction.
        </h1>
        <p style={{ marginTop: 0, color: "var(--muted)", lineHeight: 1.7, maxWidth: "52rem" }}>
          This page makes the current glyph-to-cell mapping explicit. Print the sheet, write one
          character per box, and upload a flat, evenly lit photo or scan for the strongest results.
        </p>
        <div style={{ display: "flex", gap: "1rem", flexWrap: "wrap", marginBottom: "1.5rem" }}>
          <TemplateDownload />
          <Link
            href="/"
            style={{
              padding: "0.9rem 1.2rem",
              borderRadius: "999px",
              border: "1px solid var(--border)",
              background: "var(--surface-strong)"
            }}
          >
            Back to generator
          </Link>
        </div>

        <div
          style={{
            display: "grid",
            gap: "0.75rem",
            background: "#fffaf2",
            border: "1px solid var(--border)",
            borderRadius: "24px",
            padding: "1rem",
            overflowX: "auto"
          }}
        >
          {rows.map((row, rowIndex) => (
            <div
              key={`row-${rowIndex}`}
              style={{
                display: "grid",
                gridTemplateColumns: `repeat(${templateColumns}, minmax(56px, 1fr))`,
                gap: "0.5rem",
                minWidth: "1020px"
              }}
            >
              {row.map((glyph) => (
                <div
                  key={`${rowIndex}-${glyph}`}
                  style={{
                    height: "132px",
                    borderRadius: "16px",
                    border: "1px solid #dbcbb5",
                    background: "white",
                    padding: "0.45rem",
                    display: "flex",
                    flexDirection: "column",
                    justifyContent: "space-between"
                  }}
                >
                  <span style={{ color: "#8e6248", fontSize: "0.85rem" }}>
                    {glyph === " " ? "space" : glyph}
                  </span>
                  <div
                    style={{
                      borderBottom: "1px dashed #d6c0ab",
                      marginBottom: "0.65rem"
                    }}
                  />
                </div>
              ))}
            </div>
          ))}
        </div>
      </section>
    </main>
  );
}
