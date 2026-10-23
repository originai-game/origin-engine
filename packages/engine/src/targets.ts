// Depth S8's additions (IDL `render` render targets and custom post
// passes, `load.post`): thin calls into the host's binding symbols, typed by
// the generated declarations through index.ts.

import type { HostAbi } from "./generated/abi.ts";
import { boundHost } from "./runtime.ts";

type W = { readonly id: number };

function abi(): HostAbi<Uint32Array, unknown> {
  const h = boundHost();
  if (!h) throw new Error("no engine host is loaded: start the game through @origingame/engine's web or node loader");
  return h.abi;
}

const parse = <T>(text: string): T => JSON.parse(text) as T;
const json = (value: unknown): string | undefined => (value === undefined ? undefined : JSON.stringify(value));

export const targetsRender = Object.freeze({
  renderTarget: (world: W, desc?: unknown) => parse(abi().og_render_render_target(world.id, json(desc))),
  setPostParam: (world: W, post: number, name: string, value: ReadonlyArray<number>) =>
    parse(abi().og_render_set_post_param(world.id, post, name, JSON.stringify(value))),
  postErrors: (world: W) => parse(abi().og_render_post_errors(world.id)),
});

export const targetsLoad = Object.freeze({
  post: (world: W, path: string) => parse(abi().og_load_post(world.id, path)),
});
