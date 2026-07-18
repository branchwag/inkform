import { StatusCard } from "../components/status-card";
import { GeneratorWorkbench } from "../components/generator-workbench";
const steps = [
  {
    title: "Write naturally",
    body: "Use any clear sample of your handwriting, whether it is a note, a label, or a dedicated practice page."
  },
  {
    title: "Build your character set",
    body: "Inkform studies the strokes in your upload and turns them into a reusable digital letter style."
  },
  {
    title: "Preview and save",
    body: "Try your handwriting on real words, then download the generated result to keep working with it."
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
            Turn your handwriting into a digital typeface.
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
            Upload a handwriting sheet, preview your words, and save the result in one simple flow.
            Inkform is built to make personal handwriting feel ready for digital use from any clear photo you already have.
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
              See how it works
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

        <GeneratorWorkbench />
      </section>
    </main>
  );
}
