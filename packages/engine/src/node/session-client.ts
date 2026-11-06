import { createConnection } from "node:net";

/** Null means refused before execution; a lost reply must never replay a run. */
export function runInSession(argv: string[], socketPath: string | undefined, build: string | undefined, signal?: AbortSignal): Promise<{ exitCode: number; stdout: string; stderr: string } | null> {
  if (!socketPath || !build || process.env.OG_SESSION === "0") return Promise.resolve(null);
  if (signal?.aborted) return Promise.reject(new Error("session request cancelled before sending"));
  const payload = JSON.stringify({ build, cwd: process.cwd(), env: process.env, argv }) + "\n";
  if (new TextEncoder().encode(payload).byteLength > 1024 * 1024) return Promise.reject(new Error("session request exceeds 1 MiB"));
  return new Promise((resolve, reject) => {
    let sent = false;
    let accepted = false;
    let complete = false;
    let input = "";
    let stdout = "";
    let stderr = "";
    let bytes = 0;
    const socket = createConnection(socketPath, () => {
      sent = true;
      socket.write(payload);
    });
    const cancel = () => {
      socket.destroy();
      reject(new Error("session request cancelled; outcome may be partial, not retried"));
    };
    signal?.addEventListener("abort", cancel, { once: true });
    socket.on("close", () => signal?.removeEventListener("abort", cancel));
    socket.setEncoding("utf8");
    socket.on("error", (error) => {
      if (!sent && "code" in error && (error.code === "ENOENT" || error.code === "ECONNREFUSED")) resolve(null);
      else reject(new Error(`session request interrupted; outcome unknown, not retried: ${error.message}`));
    });
    socket.on("end", () => {
      if (!complete) reject(new Error("session disconnected before its result; outcome unknown, not retried"));
    });
    socket.on("data", (chunk) => {
      bytes += new TextEncoder().encode(chunk).byteLength;
      if (bytes > 20 * 1024 * 1024) {
        socket.destroy();
        reject(new Error("session reply exceeds 20 MiB; outcome unknown, not retried"));
        return;
      }
      input += chunk;
      for (;;) {
        const end = input.indexOf("\n");
        if (end < 0) break;
        const line = input.slice(0, end);
        input = input.slice(end + 1);
        try {
          const value: unknown = JSON.parse(line);
          if (typeof value !== "object" || value === null || !("event" in value)) throw new Error("invalid session reply");
          if (value.event === "accepted" && !accepted) { accepted = true; continue; }
          else if (value.event === "stdout" && "text" in value && typeof value.text === "string") stdout += value.text;
          else if (value.event === "stderr" && "text" in value && typeof value.text === "string") stderr += value.text;
          else if (value.event === "exit" && "code" in value && typeof value.code === "number" && Number.isInteger(value.code) && value.code >= 0 && value.code <= 255) {
            complete = true;
            socket.end();
            resolve({ exitCode: value.code, stdout, stderr });
          } else if (value.event === "rejected" && !accepted && "reason" in value && typeof value.reason === "string") {
            complete = true;
            socket.end();
            process.stderr.write(`og session: ${value.reason}; running alone\n`);
            resolve(null);
          } else throw new Error("invalid session reply");
        } catch (error) {
          socket.destroy();
          reject(error);
        }
      }
    });
  });
}
