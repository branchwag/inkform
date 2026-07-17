type StatusCardProps = {
  title: string;
  body: string;
};

export function StatusCard({ title, body }: StatusCardProps) {
  return (
    <article
      style={{
        border: "1px solid var(--border)",
        borderRadius: "20px",
        padding: "1.25rem",
        background: "var(--surface)",
        boxShadow: "0 18px 40px var(--shadow)"
      }}
    >
      <h3 style={{ margin: "0 0 0.5rem", fontSize: "1.1rem" }}>{title}</h3>
      <p style={{ margin: 0, lineHeight: 1.6, color: "var(--muted)" }}>{body}</p>
    </article>
  );
}
