export type ScriptPackId = "latin-extended";

export type ValidationReport = {
  accepted: boolean;
  notes: string[];
  glyphTargetCount: number;
};

export type FontArtifact = {
  familyName: string;
  scriptPackId: ScriptPackId;
  glyphCount: number;
  anchorCount: number;
  binaryLabel: string;
  binaryHash: string;
  downloadName: string;
  mimeType: string;
  bytes: number[];
};

export type PreviewResponse = {
  renderPlan: string;
  unsupportedCharacters: string[];
  previewVersion: string;
  svgMarkup: string;
};

export type GenerationResult = {
  validation: ValidationReport;
  artifact: FontArtifact;
  preview: PreviewResponse;
};
