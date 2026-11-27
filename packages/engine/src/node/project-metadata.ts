// Ordinary project identity. The launcher and node tools share this reader;
// no TS execution, generated projection, hashes or source-format detection.
import { readFileSync } from "node:fs";
import { join } from "node:path";

export const METADATA = "og-project.json";
export type ProjectErrorCode = "project_metadata_invalid" | "no_pin";
export class ProjectMetadataError extends Error {
  readonly retryable = false;
  readonly code: ProjectErrorCode;
  constructor(code: ProjectErrorCode, message: string) { super(message); this.code = code; }
}
export interface ProjectMetadata {
  readonly version: 1;
  readonly engine: string;
  readonly name: string;
  readonly look?: string | { readonly $ogRef: string };
}
const invalid = (message: string): never => { throw new ProjectMetadataError("project_metadata_invalid", `og-project.json: ${message}`); };

export function parseProjectMetadata(text: string): ProjectMetadata {
  let value: unknown;
  try { value = JSON.parse(text); } catch (e) { return invalid((e as Error).message); }
  if (typeof value !== "object" || value === null || Array.isArray(value)) return invalid("expected an object");
  const v = value as Record<string, unknown>;
  if (typeof v.engine !== "string" || !/^(?:\d+\.\d+\.\d+|dev\.\d{8}\.[1-9]\d*)$/.test(v.engine)) {
    throw new ProjectMetadataError("no_pin", "og-project.json: engine must name a concrete engine version");
  }
  if (Object.keys(v).some((key) => !["version", "engine", "name", "look"].includes(key))) invalid("unknown project field");
  if (v.version !== 1) invalid("version must be 1");
  if (typeof v.name !== "string" || !v.name) invalid("name must not be empty");
  if (Object.hasOwn(v, "look")) {
    const look = v.look;
    if (typeof look === "string") {
      if (!look) invalid("look must not be empty");
    } else if (typeof look !== "object" || look === null || Array.isArray(look)
      || Object.keys(look).length !== 1 || !("$ogRef" in look)
      || typeof look.$ogRef !== "string" || !look.$ogRef) invalid("look must be a preset string or {$ogRef: content ID}");
  }
  return v as unknown as ProjectMetadata;
}

export function readProjectMetadata(root: string): ProjectMetadata {
  try { return parseProjectMetadata(readFileSync(join(root, METADATA), "utf8")); }
  catch (e) {
    if (e instanceof ProjectMetadataError) throw e;
    return invalid((e as Error).message);
  }
}
