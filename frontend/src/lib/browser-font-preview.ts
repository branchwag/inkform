import type { FontArtifact } from "./engine-types";

export type LoadedBrowserFont = {
  cleanup: () => void;
  familyName: string;
  fontFace: FontFace;
};

type FontLoadStrategy = {
  cleanup: () => void;
  label: string;
  source: ArrayBuffer | string;
};

export async function loadGeneratedBrowserFont(
  artifact: FontArtifact
): Promise<LoadedBrowserFont> {
  const familyName = `${artifact.familyName}-${artifact.binaryLabel}-${artifact.binaryHash}`;
  const bytes = Uint8Array.from(artifact.bytes);
  const blob = new Blob([bytes], {
    type: artifact.mimeType
  });
  const objectUrl = URL.createObjectURL(blob);

  const strategies = await buildStrategies(blob, bytes, objectUrl);
  const errors: string[] = [];

  for (const strategy of strategies) {
    try {
      const fontFace = new FontFace(familyName, strategy.source);
      const loadedFontFace = await fontFace.load();
      return {
        cleanup: strategy.cleanup,
        familyName,
        fontFace: loadedFontFace
      };
    } catch (error) {
      strategy.cleanup();
      const message = error instanceof Error ? error.message : "Unknown font load failure.";
      errors.push(`${strategy.label}: ${message}`);
    }
  }

  throw new Error(`All browser font load strategies failed. ${errors.join(" | ")}`);
}

async function buildStrategies(
  blob: Blob,
  bytes: Uint8Array,
  objectUrl: string
): Promise<FontLoadStrategy[]> {
  const dataUrl = await readBlobAsDataUrl(blob);
  const arrayBuffer = await blob.arrayBuffer();

  return [
    {
      label: "blob-url",
      source: `url("${objectUrl}") format("truetype")`,
      cleanup: () => {
        URL.revokeObjectURL(objectUrl);
      }
    },
    {
      label: "data-url",
      source: `url("${dataUrl}") format("truetype")`,
      cleanup: () => {
        URL.revokeObjectURL(objectUrl);
      }
    },
    {
      label: "array-buffer",
      source: arrayBuffer,
      cleanup: () => {
        URL.revokeObjectURL(objectUrl);
      }
    },
    {
      label: "manual-data-url",
      source: `url("${buildManualDataUrl(bytes, blob.type)}") format("truetype")`,
      cleanup: () => {
        URL.revokeObjectURL(objectUrl);
      }
    }
  ];
}

async function readBlobAsDataUrl(blob: Blob): Promise<string> {
  return await new Promise((resolve, reject) => {
    const reader = new FileReader();

    reader.onload = () => {
      if (typeof reader.result === "string") {
        resolve(reader.result);
        return;
      }

      reject(new Error("FileReader did not return a string data URL."));
    };

    reader.onerror = () => {
      reject(reader.error ?? new Error("FileReader failed while preparing preview font."));
    };

    reader.readAsDataURL(blob);
  });
}

function buildManualDataUrl(bytes: Uint8Array, mimeType: string): string {
  let binary = "";

  for (const value of bytes) {
    binary += String.fromCharCode(value);
  }

  return `data:${mimeType};base64,${btoa(binary)}`;
}
