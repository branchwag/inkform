declare module "/wasm/inkform_wasm.js" {
  export function generate_font_json(
    bytes: Uint8Array,
    width: number,
    height: number,
    previewText: string
  ): string;
}
