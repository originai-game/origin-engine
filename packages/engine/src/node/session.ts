// Local, sequential fresh boots. The socket protocol is separate from the
// public probe: stdout/stderr and exit status remain the command's own.
import { createServer } from "node:net";
import { freemem } from "node:os";
import { dirname, join } from "node:path";
import { rmSync } from "node:fs";
import { Console } from "node:console";
import { Writable } from "node:stream";
import { native } from "./index.ts";
import { run } from "./run.ts";
import { freezeInputs } from "./session-inputs.ts";

declare module "node:os" {
  export function freemem(): number;
}

type Request = { build: string; cwd: string; argv: string[]; env: Record<string, string> };
type Event = { event: "accepted" | "busy" } | { event: "stdout" | "stderr"; text: string } | { event: "exit"; code: number } | { event: "rejected"; reason: string };

function request(value: unknown): Request {
  if (typeof value !== "object" || value === null || !("build" in value) || !("cwd" in value) || !("argv" in value) || !("env" in value))
    throw new Error("expected build, cwd, argv and env");
  const { build, cwd, argv, env } = value;
  if (typeof build !== "string" || typeof cwd !== "string" || !Array.isArray(argv) || !argv.every((arg): arg is string => typeof arg === "string"))
    throw new Error("invalid session request");
  if (typeof env !== "object" || env === null) throw new Error("invalid environment");
  const environment: Record<string, string> = {};
  for (const [key, val] of Object.entries(env)) {
    if (typeof val !== "string") throw new Error("invalid environment value");
    environment[key] = val;
  }
  if (argv.some((arg) => ["--watch", "--port", "--room", "--handoff", "--restore", "--realtime", "--trace-append"].includes(arg))) throw new Error("interactive, restore and trace-append runs use a dedicated process");
  return { build, cwd, argv, env: environment };
}

const [socketPath, build] = process.argv.slice(2);
if (!socketPath || !build) throw new Error("og session requires a socket and build identity");
const environment = Object.entries(process.env).filter((entry): entry is [string, string] => entry[1] !== undefined);
let queue = Promise.resolve();
let pending = 0;
let admitted = 0;
let stopping = false;
let idleSince = performance.now();
const inputs = join(dirname(socketPath), "inputs");
const active = new Set<AbortController>();
const log = process.stderr.write.bind(process.stderr);
native().og_node_resident(true);

const server = createServer((socket) => {
  let input = "";
  let inputBytes = 0;
  let received = false;
  const abort = new AbortController();
  const send = (event: Event) => {
    if (!socket.destroyed) socket.write(JSON.stringify(event) + "\n");
  };
  socket.setEncoding("utf8");
  socket.setTimeout(10_000, () => socket.destroy());
  socket.on("close", () => abort.abort());
  socket.on("error", () => {
    socket.destroy();
  });
  socket.on("data", (chunk) => {
    if (received) return;
    input += chunk;
    inputBytes += new TextEncoder().encode(chunk).byteLength;
    if (inputBytes > 1024 * 1024) {
      received = true;
      socket.end(JSON.stringify({ event: "rejected", reason: "request exceeds 1 MiB" }) + "\n");
      return;
    }
    const newline = input.indexOf("\n");
    if (newline < 0) return;
    received = true;
    socket.setTimeout(0);
    if (stopping || admitted >= 128 || pending >= 8) {
      send({ event: "busy" });
      send({ event: "stderr", text: "og session: admission limit reached; command was not executed\n" });
      send({ event: "exit", code: 75 });
      socket.end();
      return;
    }
    let call: Request;
    let frozen: ReturnType<typeof freezeInputs>;
    try {
      call = request(JSON.parse(input.slice(0, newline)));
      input = "";
      if (call.build !== build) throw new Error("session belongs to a different engine build");
      // Native libraries may cache environment settings. Never run a request
      // under a different environment and silently change its result.
      const ignored = new Set(["_", "SHLVL", "PWD", "OLDPWD"]);
      const actual = environment.filter(([key]) => !ignored.has(key)).sort();
      const expected = Object.entries(call.env)
        .filter(([key]) => !ignored.has(key))
        .sort();
      if (JSON.stringify(actual) !== JSON.stringify(expected)) throw new Error("session environment differs from the command");
    } catch (error) {
      send({ event: "rejected", reason: error instanceof Error ? error.message : String(error) });
      socket.end();
      return;
    }
    try {
      frozen = freezeInputs(call.argv, call.cwd, inputs);
    } catch (error) {
      // A changed or incomplete generation must not cold-fallback with the
      // stale prepared bundle. Nothing ran, but preparation needs repeating.
      send({ event: "stderr", text: `og session: input admission failed: ${String(error)}\n` });
      send({ event: "exit", code: 75 });
      socket.end();
      return;
    }
    pending++;
    admitted++;
    active.add(abort);
    send({ event: "accepted" });
    queue = queue.then(async () => {
      const cwd = process.cwd();
      const stdout = process.stdout.write;
      const stderr = process.stderr.write;
      const console = globalThis.console;
      let written = 0;
      try {
        if (abort.signal.aborted) return;
        process.chdir(call.cwd);
        const write = (event: "stdout" | "stderr", text: string) => {
          const wire = JSON.stringify({ event, text }) + "\n";
          written += new TextEncoder().encode(wire).byteLength;
          if (written > 16 * 1024 * 1024) throw new Error("session output exceeds 16 MiB");
          if (socket.writableLength > 1024 * 1024) {
            socket.destroy();
            throw new Error("session output consumer stopped reading");
          }
          if (!socket.destroyed) socket.write(wire);
          return true;
        };
        const output = {
          stdout: { write: (text: string) => write("stdout", text) },
          stderr: { write: (text: string) => write("stderr", text) },
        };
        process.stdout.write = output.stdout.write;
        process.stderr.write = output.stderr.write;
        const stream = (event: "stdout" | "stderr") => new Writable({
          write(chunk, _encoding, done) {
            write(event, new TextDecoder().decode(chunk));
            done();
          },
        });
        globalThis.console = new Console({ stdout: stream("stdout"), stderr: stream("stderr"), ignoreErrors: false });
        send({ event: "exit", code: await run(frozen.argv, output, { signal: abort.signal, ...(frozen.inputRoot ? { inputRoot: frozen.inputRoot } : {}) }) });
      } catch (error) {
        const text = error instanceof Error ? (error.stack ?? error.message) : String(error);
        send({ event: "stderr", text: new TextDecoder().decode(new TextEncoder().encode(text).subarray(0, 64 * 1024)) + "\n" });
        send({ event: "exit", code: 1 });
      } finally {
        process.stdout.write = stdout;
        process.stderr.write = stderr;
        globalThis.console = console;
        process.chdir(cwd);
        frozen.dispose();
        active.delete(abort);
        pending--;
        idleSince = performance.now();
        socket.setTimeout(10_000);
        socket.end();
        if (admitted >= 128 && pending === 0) stop(75);
      }
    }).catch((error: unknown) => {
      log(`og session: cleanup failed: ${String(error)}\n`);
      native().og_node_resident(false);
      process.exit(1);
    });
  });
});
server.on("error", (error) => {
  process.stderr.write(`og session: ${error.message}\n`);
  process.exit(1);
});
server.maxConnections = 32;
server.listen(socketPath, () => log(`og session: listening at ${socketPath} (pid ${process.pid})\n`));
const stop = (code = 0) => {
  if (stopping) return;
  stopping = true;
  for (const controller of active) controller.abort();
  const closed = new Promise<void>((resolve) => server.close(resolve));
  // Retirement must flush the last result before the supervisor replaces us.
  void Promise.all([queue, closed]).finally(() => {
    native().og_node_resident(false);
    rmSync(inputs, { recursive: true, force: true });
    process.exit(code);
  });
};
process.on("SIGTERM", () => stop());
process.on("SIGINT", () => stop());
setInterval(() => {
  if (pending === 0 && (performance.now() - idleSince > 5 * 60_000 || freemem() < 256 * 1024 * 1024)) {
    log("og session: exiting on idle timeout or low memory\n");
    stop();
  }
}, 1000);
