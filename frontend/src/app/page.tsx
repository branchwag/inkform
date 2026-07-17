import { StatusCard } from "@/components/status-card";

const steps = [
  {
    title: "Guide the sample",
    body: "Users download a handwriting sheet, fill it out, and upload a clean photo or scan."
  },
  {
    title: "Process locally",
    body: "Inkform validates, segments, and normalizes the sample using the Rust engine in the browser."
  },
  {
    title: "Preview and export",
    body: "The generated font is previewed against custom text before export for real-world use."
  }
];

export default function HomePage() {
  return (
    <main
      style={{
        padding: "3rem 1.5rem 5rem",
        maxWidth: "1120px",
        margin: "0 auto"
      }}
    >
      <section
        style={{
          display: "grid",
          gap: "2rem",
          alignItems: "start"
        }}
      >
        <div
          style={{
            padding: "2rem",
            background: "var(--surface)",
            border: "1px solid var(--border)",
            borderRadius: "28px",
            boxShadow: "0 24px 60px var(--shadow)"
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
            Inkform
          </p>
          <h1 style={{ fontSize: "clamp(2.5rem, 7vw, 5rem)", margin: "0.75rem 0 1rem" }}>
            Turn a handwriting sheet into a usable font.
          </h1>
          <p
            style={{
              fontSize: "1.15rem",
              lineHeight: 1.7,
              color: "var(--muted)",
              maxWidth: "46rem",
              margin: 0
            }}
          >
            A Rust-first app designed for Vercel Hobby. The upload flow lives on the web, while the
            generation engine is structured for browser-side execution through WebAssembly.
          </p>
          <div style={{ display: "flex", flexWrap: "wrap", gap: "1rem", marginTop: "1.5rem" }}>
            <a
              href="#workflow"
              style={{
                padding: "0.9rem 1.2rem",
                borderRadius: "999px",
                background: "var(--accent)",
                color: "#fff8f2"
              }}
            >
              View workflow
            </a>
            <a
              href="https://openai.devpost.com/rules"
              style={{
                padding: "0.9rem 1.2rem",
                borderRadius: "999px",
                border: "1px solid var(--border)",
                background: "var(--surface-strong)"
              }}
            >
              Build Week rules
            </a>
          </div>
        </div>

        <section
          id="workflow"
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fit, minmax(220px, 1fr))",
            gap: "1rem"
          }}
        >
          {steps.map((step) => (
            <StatusCard key={step.title} title={step.title} body={step.body} />
          ))}
        </section>
      </section>
    </main>
  );
}
