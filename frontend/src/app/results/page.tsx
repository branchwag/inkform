const samplePreview = "Grüße aus Inkform";

export default function ResultsPage() {
  return (
    <main style={{ maxWidth: "960px", margin: "0 auto", padding: "3rem 1.5rem 4rem" }}>
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
          Preview state
        </p>
        <h1 style={{ margin: "0.75rem 0 1rem", fontSize: "clamp(2rem, 5vw, 3.5rem)" }}>
          Generated font preview scaffold
        </h1>
        <p style={{ lineHeight: 1.7, color: "var(--muted)", marginTop: 0 }}>
          This page represents the post-generation state. Once the WASM bindings are wired into the
          frontend, this route should render live output from the Rust engine instead of placeholder
          copy.
        </p>

        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fit, minmax(240px, 1fr))",
            gap: "1rem",
            marginTop: "1.5rem"
          }}
        >
          <div
            style={{
              padding: "1.5rem",
              borderRadius: "20px",
              border: "1px solid var(--border)",
              background: "var(--surface-strong)"
            }}
          >
            <h2 style={{ marginTop: 0 }}>Typed preview</h2>
            <p style={{ fontSize: "1.4rem", marginBottom: 0 }}>{samplePreview}</p>
          </div>
          <div
            style={{
              padding: "1.5rem",
              borderRadius: "20px",
              border: "1px solid var(--border)",
              background: "var(--surface-strong)"
            }}
          >
            <h2 style={{ marginTop: 0 }}>Current artifact</h2>
            <p style={{ color: "var(--muted)", lineHeight: 1.7, marginBottom: 0 }}>
              The Rust workspace currently emits a deterministic placeholder binary so the preview
              and test suite have a stable artifact shape to target while the real font compiler is
              implemented.
            </p>
          </div>
        </div>
      </section>
    </main>
  );
}
