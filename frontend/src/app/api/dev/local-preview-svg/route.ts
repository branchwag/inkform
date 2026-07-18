import { readdir, stat } from "node:fs/promises";
import { join } from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { NextResponse } from "next/server";

const executeFile = promisify(execFile);
const downloadsDirectory = "/home/whiterabbit/Downloads";
const imageExtensions = [".jpg", ".jpeg", ".png", ".webp"];
const previewText = "The quick brown fox jumps over the lazy dog.";

type CandidateFile = {
  modifiedMs: number;
  name: string;
  path: string;
};

async function latestDownloadsImage(): Promise<CandidateFile | null> {
  const entries = await readdir(downloadsDirectory, { withFileTypes: true });
  const candidates = await Promise.all(
    entries
      .filter((entry) => entry.isFile())
      .filter((entry) => {
        const lowerName = entry.name.toLowerCase();
        return imageExtensions.some((extension) => lowerName.endsWith(extension));
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
  return candidates[0] ?? null;
}

export async function GET(): Promise<Response> {
  if (process.env.NODE_ENV !== "development") {
    return NextResponse.json({ error: "Not found." }, { status: 404 });
  }

  const latest = await latestDownloadsImage();
  if (latest === null) {
    return NextResponse.json({ error: "No image files found in Downloads." }, { status: 404 });
  }

  const command = [
    "run",
    "-p",
    "inkform-core",
    "--example",
    "dump_preview_svg",
    "--",
    latest.path,
    previewText
  ];

  const { stdout } = await executeFile("cargo", command, {
    cwd: "/home/whiterabbit/CodingStuff/OpenAI_hackathon",
    maxBuffer: 8 * 1024 * 1024
  });

  return new NextResponse(stdout, {
    headers: {
      "Content-Type": "image/svg+xml; charset=utf-8",
      "X-Inkform-Debug-File": latest.name
    }
  });
}
