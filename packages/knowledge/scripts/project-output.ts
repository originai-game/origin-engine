// og project --json has one project object, optionally preceded by the
// launcher's install events (docs/release.md §1). Kept with knowledge so a
// standalone platform mount and the engine's host tools read the same stream.

export function readProjectOutput<T extends object>(result: { error?: Error; status: number | null; stdout: string; stderr: string }): T {
  if (result.error) throw result.error;
  let project: T | undefined;
  for (const line of result.stdout.split(/\r?\n/).filter((line) => line.trim() !== "")) {
    const value: unknown = JSON.parse(line);
    if (typeof value !== "object" || value === null || Array.isArray(value)) throw new Error("og project: expected a JSON object on each stdout line");
    if ("event" in value) {
      if (value.event === "error") {
        const error = value as { code?: unknown; message?: unknown };
        throw new Error(`og project: ${error.code}: ${error.message}`);
      }
      continue;
    }
    if (project !== undefined) throw new Error("og project: expected one project object, received multiple");
    project = value as T;
  }
  if (result.status !== 0) throw new Error(result.stderr.trim() || `og project exited ${result.status}`);
  if (project === undefined) throw new Error("og project: no project object in stdout");
  return project;
}
