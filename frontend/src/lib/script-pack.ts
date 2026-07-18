export const templateColumns = 17;
export const templateRows = 7;

export const latinExtendedGlyphs = [
  ..."ABCDEFGHIJKLMNOPQRSTUVWXYZ",
  ..."abcdefghijklmnopqrstuvwxyz",
  ..."0123456789",
  ..." .,;:!?'-_\"()[]{}@/#&%+*=<>",
  ..."ÄÖÜäöüßÀÁÂÃÅÆÇÈÉÊËÌÍÎÏÑÒÓÔÕØÙÚÛÝ",
  ..."àáâãåæçèéêëìíîïñòóôõøùúûýÿ"
] as const;

export function chunkGlyphs<T>(items: readonly T[], size: number): T[][] {
  const rows: T[][] = [];
  for (let index = 0; index < items.length; index += size) {
    rows.push([...items.slice(index, index + size)]);
  }
  return rows;
}

export function buildTemplateSvg(): string {
  const width = 1400;
  const height = 1900;
  const padding = 70;
  const cellWidth = 74;
  const cellHeight = 210;
  const rows = chunkGlyphs(latinExtendedGlyphs, templateColumns);

  const title = "Inkform Handwriting Sheet";
  const subtitle = "Write one character per box using the printed guide below.";

  const cells = rows
    .map((row, rowIndex) =>
      row
        .map((glyph, columnIndex) => {
          const x = padding + columnIndex * cellWidth;
          const y = 220 + rowIndex * cellHeight;
          const baselineY = y + 156;
          const glyphLabel = escapeSvg(glyph === " " ? "space" : glyph);

          return [
            `<rect x="${x}" y="${y}" width="${cellWidth - 8}" height="${cellHeight - 16}" rx="12" ry="12" fill="white" stroke="#d8c8b3" stroke-width="2"/>`,
            `<line x1="${x + 12}" y1="${baselineY}" x2="${x + cellWidth - 20}" y2="${baselineY}" stroke="#d8c8b3" stroke-width="1.5" stroke-dasharray="6 6"/>`,
            `<text x="${x + 10}" y="${y + 24}" font-size="16" fill="#8e6248" font-family="Georgia, serif">${glyphLabel}</text>`
          ].join("");
        })
        .join("")
    )
    .join("");

  return `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">
  <rect width="100%" height="100%" fill="#f7f0e5"/>
  <text x="${padding}" y="96" font-size="42" fill="#472b1f" font-family="Georgia, serif">${title}</text>
  <text x="${padding}" y="134" font-size="22" fill="#745d50" font-family="Georgia, serif">${subtitle}</text>
  <text x="${padding}" y="168" font-size="18" fill="#745d50" font-family="Georgia, serif">Use dark ink, keep each character centered, and photograph the page flat.</text>
  ${cells}
</svg>`;
}

function escapeSvg(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&apos;");
}
