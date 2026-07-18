const checklist = [
  "Use an evenly lit photo or a flatbed scan.",
  "Keep the handwriting large enough to read clearly.",
  "Avoid shadows, blur, and folded paper.",
  "Include a few words or lines so Inkform has more stroke detail to study."
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
          Upload tips
        </p>
        <h1 style={{ margin: "0.75rem 0 1rem", fontSize: "clamp(2rem, 5vw, 3.5rem)" }}>
          Get a stronger result from a freeform handwriting photo.
        </h1>
        <p style={{ lineHeight: 1.7, color: "var(--muted)", marginTop: 0 }}>
          Inkform works from ordinary handwriting samples, not a required template. These tips are
          here to help users capture cleaner input while the Rust engine keeps improving its
          freeform extraction and font assembly.
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
              The current build validates uploads, studies handwriting structure, generates a TTF
              artifact, and previews typed text in the browser. The main remaining gap is making
              the generated letterforms look more like the uploaded hand.
            </p>
          </div>
        </div>
      </section>
    </main>
  );
}
