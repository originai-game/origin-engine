import { readFileSync } from "node:fs";

/** File layout only: keep the command grammar and its escapes unchanged. */
export function readCommandsFile(file: string): string {
  return readFileSync(file, "utf8").split(/\r?\n/)
    .filter((line) => line.trim() !== "" && !line.trimStart().startsWith("#"))
    .join(";").trim();
}
