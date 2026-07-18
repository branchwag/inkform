import { readdir, readFile, stat } from "node:fs/promises";
import { join } from "node:path";
import { NextResponse } from "next/server";

const downloadsDirectory = "/home/whiterabbit/Downloads";
const imageExtensions = new Set([".jpg", ".jpeg", ".png", ".webp"]);

type CandidateFile = {
  modifiedMs: number;
  name: string;
  path: string;
};

export async function GET(): Promise<Response> {
  if (process.env.NODE_ENV !== "development") {
    return NextResponse.json({ error: "Not found." }, { status: 404 });
  }

  const entries = await readdir(downloadsDirectory, { withFileTypes: true });
  const candidates = await Promise.all(
    entries
      .filter((entry) => entry.isFile())
      .filter((entry) => {
        const lowerName = entry.name.toLowerCase();
        return [...imageExtensions].some((extension) => lowerName.endsWith(extension));
      })
      .map(async (entry) => {
        const path = join(downloadsDirectory, entry.name);
        const metadata = await stat(path);

        return {
          modifiedMs: metadata.mtimeMs,
          name: entry.name,
          path
        } satisfies CandidateFile;
      })
  );

  candidates.sort((left, right) => right.modifiedMs - left.modifiedMs);
  const latest = candidates[0];
  if (latest === undefined) {
    return NextResponse.json({ error: "No image files found in Downloads." }, { status: 404 });
  }

  const bytes = await readFile(latest.path);
  const mimeType = latest.name.toLowerCase().endsWith(".png")
    ? "image/png"
    : latest.name.toLowerCase().endsWith(".webp")
      ? "image/webp"
      : "image/jpeg";

  return new NextResponse(bytes, {
    headers: {
      "Content-Type": mimeType,
      "X-Inkform-Debug-File": latest.name
    }
  });
}
