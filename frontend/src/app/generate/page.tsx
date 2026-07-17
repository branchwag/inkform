const checklist = [
  "Use an evenly lit photo or a flatbed scan.",
  "Keep every glyph inside its guide box.",
  "Avoid shadows, blur, and folded paper.",
  "Fill the full Latin-extended sheet for the current v1 flow."
];

export default function GeneratePage() {
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
          Guided input
        </p>
        <h1 style={{ margin: "0.75rem 0 1rem", fontSize: "clamp(2rem, 5vw, 3.5rem)" }}>
          Prepare a handwriting sheet for generation.
        </h1>
        <p style={{ lineHeight: 1.7, color: "var(--muted)", marginTop: 0 }}>
          This page is the product scaffold for the upload flow. The Rust engine already exposes
          validation, generation, and preview primitives; the next implementation step is wiring
          the compiled WASM package into this guided form.
        </p>

        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fit, minmax(220px, 1fr))",
            gap: "1rem",
            marginTop: "1.5rem"
          }}
        >
          <div
            style={{
              padding: "1.25rem",
              border: "1px solid var(--border)",
              borderRadius: "20px",
              background: "var(--surface-strong)"
            }}
          >
            <h2 style={{ marginTop: 0 }}>Checklist</h2>
            <ul style={{ margin: 0, paddingLeft: "1.2rem", lineHeight: 1.7 }}>
              {checklist.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </div>

          <div
            style={{
              padding: "1.25rem",
              border: "1px solid var(--border)",
              borderRadius: "20px",
              background: "var(--surface-strong)"
            }}
          >
            <h2 style={{ marginTop: 0 }}>Pipeline status</h2>
            <p style={{ color: "var(--muted)", lineHeight: 1.7 }}>
              The current scaffold validates sample dimensions, tracks script-pack targets, builds
              a placeholder artifact, and produces preview plans. Real raster segmentation and font
              assembly are the next milestones.
            </p>
          </div>
        </div>
      </section>
    </main>
  );
}
