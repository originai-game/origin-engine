// Local, sequential fresh boots. The socket protocol is separate from the
// public probe: stdout/stderr and exit status remain the command's own.
import { createServer } from "node:net";
import { freemem } from "node:os";
import { native } from "./index.ts";
import { run } from "./run.ts";

declare module "node:os" {
  export function freemem(): number;
}

type Request = { build: string; cwd: string; argv: string[]; env: Record<string, string> };
type Event = { event: "stdout" | "stderr"; text: string } | { event: "exit"; code: number } | { event: "rejected"; reason: string };

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
  if (argv.some((arg) => ["--watch", "--port", "--room", "--handoff", "--realtime"].includes(arg))) throw new Error("interactive runs use a dedicated process");
  return { build, cwd, argv, env: environment };
}

const [socketPath, build] = process.argv.slice(2);
if (!socketPath || !build) throw new Error("og session requires a socket and build identity");
const environment = Object.entries(process.env).filter((entry): entry is [string, string] => entry[1] !== undefined);
let queue = Promise.resolve();
let pending = 0;
let idleSince = performance.now();
native().og_node_resident(true);

const server = createServer((socket) => {
  let input = "";
  let received = false;
  const send = (event: Event) => {
    socket.write(JSON.stringify(event) + "\n");
  };
  socket.setEncoding("utf8");
  socket.on("error", () => {
    socket.destroy();
  });
  socket.on("data", (chunk) => {
    if (received) return;
    input += chunk;
    if (input.length > 1024 * 1024) {
      received = true;
      socket.end(JSON.stringify({ event: "rejected", reason: "request exceeds 1 MiB" }) + "\n");
      return;
    }
    const newline = input.indexOf("\n");
    if (newline < 0) return;
    received = true;
    let call: Request;
    try {
      call = request(JSON.parse(input.slice(0, newline)));
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
    pending++;
    queue = queue.then(async () => {
      const cwd = process.cwd();
      const stdout = process.stdout.write;
      const stderr = process.stderr.write;
      try {
        process.chdir(call.cwd);
        const output = {
          stdout: {
            write: (text: string) => {
              send({ event: "stdout", text });
              return true;
            },
          },
          stderr: {
            write: (text: string) => {
              send({ event: "stderr", text });
              return true;
            },
          },
        };
        process.stdout.write = output.stdout.write;
        process.stderr.write = output.stderr.write;
        send({ event: "exit", code: await run(call.argv, output) });
      } catch (error) {
        send({ event: "stderr", text: `${error instanceof Error ? (error.stack ?? error.message) : String(error)}\n` });
        send({ event: "exit", code: 1 });
      } finally {
        process.stdout.write = stdout;
        process.stderr.write = stderr;
        process.chdir(cwd);
        pending--;
        idleSince = performance.now();
        socket.end();
      }
    });
  });
});
server.on("error", (error) => {
  process.stderr.write(`og session: ${error.message}\n`);
  process.exit(1);
});
server.listen(socketPath, () => process.stderr.write(`og session: listening at ${socketPath} (pid ${process.pid})\n`));
const stop = () => {
  server.close();
  void queue.finally(() => {
    native().og_node_resident(false);
    process.exit(0);
  });
};
process.on("SIGTERM", stop);
process.on("SIGINT", stop);
setInterval(() => {
  if (pending === 0 && (performance.now() - idleSince > 5 * 60_000 || freemem() < 256 * 1024 * 1024)) {
    process.stderr.write("og session: exiting on idle timeout or low memory\n");
    stop();
  }
}, 1000);
