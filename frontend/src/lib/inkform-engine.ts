import { generateDemoResult } from "./demo-engine";
import type { GenerationResult } from "./engine-types";

type WasmModule = {
  default: (options?: { module_or_path?: string | URL }) => Promise<unknown>;
  generate_font_json: (
    bytes: Uint8Array,
    width: number,
    height: number,
    previewText: string
  ) => string;
};

export type EngineMode = "wasm" | "fallback";

export type EngineRun = {
  engineMode: EngineMode;
  result: GenerationResult;
};

export async function generateInkformResult(
  file: File,
  previewText: string
): Promise<EngineRun> {
  const wasmModule = await loadWasmModule();
  if (wasmModule !== null) {
    try {
      const [bytes, dimensions] = await Promise.all([readFileBytes(file), readImageSize(file)]);
      const payload = wasmModule.generate_font_json(
        bytes,
        dimensions.width,
        dimensions.height,
        previewText
      );

      return {
        engineMode: "wasm",
        result: parseGenerationResult(payload)
      };
    } catch {
      return {
        engineMode: "fallback",
        result: generateDemoResult(file, previewText)
      };
    }
  }

  return {
    engineMode: "fallback",
    result: generateDemoResult(file, previewText)
  };
}

async function loadWasmModule(): Promise<WasmModule | null> {
  if (typeof window === "undefined") {
    return null;
  }

  try {
    const dynamicImport = (specifier: string) => import(/* webpackIgnore: true */ specifier);
    const wasmExports = (await dynamicImport("/wasm/inkform_wasm.js")) as unknown as WasmModule;
    await wasmExports.default({ module_or_path: "/wasm/inkform_wasm_bg.wasm" });
    return wasmExports;
  } catch {
    return null;
  }
}

async function readFileBytes(file: File): Promise<Uint8Array> {
  const buffer = await file.arrayBuffer();
  return new Uint8Array(buffer);
}

async function readImageSize(file: File): Promise<{ width: number; height: number }> {
  if (typeof createImageBitmap === "function") {
    const imageBitmap = await createImageBitmap(file);
    const size = {
      width: imageBitmap.width,
      height: imageBitmap.height
    };
    imageBitmap.close();
    return size;
  }

  return await new Promise((resolve, reject) => {
    const image = new Image();
    const objectUrl = URL.createObjectURL(file);

    image.onload = () => {
      resolve({
        width: image.naturalWidth,
        height: image.naturalHeight
      });
      URL.revokeObjectURL(objectUrl);
    };

    image.onerror = () => {
      URL.revokeObjectURL(objectUrl);
      reject(new Error("Could not decode image dimensions."));
    };

    image.src = objectUrl;
  });
}

function parseGenerationResult(payload: string): GenerationResult {
  const parsed = JSON.parse(payload) as GenerationResult;

  if (typeof parsed !== "object" || parsed === null) {
    throw new Error("WASM generation payload was not an object.");
  }

  return parsed;
}
