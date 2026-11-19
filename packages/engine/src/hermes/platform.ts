// og.platform in a store package (shell.ts): the provider the package
// bundled with the game (`package.ts --provider`), else the local document,
// saved in the player's data directory so the store and leaderboards keep
// across runs offline.
//
// A store's provider module default-exports a `ShellProviderFactory`
// (@origingame/engine/provider). Its native half, when the package has one
// (`--provider-lib`), is a library with one C entry, `og_provider_call(method,
// json)` (crates/og-host-c/src/platform.rs); the factory reaches it through
// `native.call`. The store's credentials and SDK stay in that library and
// module, never in the engine.

import { localPlatform } from "../platform-local.ts";
import type { PlatformProvider, ShellNative, ShellProviderFactory } from "../provider.ts";

/** The shell's services: the save document's load and save, the provider library. */
export interface ShellServices {
  load(game: string): string | null;
  save(game: string, document: string): boolean;
  open(path: string): { ok: boolean; reason?: string };
  call(method: string, json: string): string | null;
  warn(message: string): void;
}

/** The provider for this package: the bundled one over its library, else the local document. */
export function shellPlatform(env: Readonly<Record<string, string | undefined>>, services: ShellServices, factory?: ShellProviderFactory): PlatformProvider {
  const game = env.OG_TITLE ?? "Origin";
  // The document lives in memory, written through to the data directory: a
  // full or read-only disk loses the save, not the game (one warning until a
  // save works again).
  let text = services.load(game);
  let refused = false;
  const local = localPlatform({
    load: () => text,
    save: (t) => {
      text = t;
      const saved = services.save(game, t);
      if (!saved && !refused) services.warn(`og.platform: could not save ${game}'s document in the data directory`);
      refused = !saved;
    },
  });
  if (!factory) return local;
  let native: ShellNative | null = null;
  const lib = env.OG_PROVIDER_LIB;
  if (lib) {
    const path = env.OG_BIN ? `${env.OG_BIN}/${lib}` : lib;
    const opened = services.open(path);
    if (!opened.ok) throw new Error(`the provider library: ${opened.reason}`);
    native = {
      call: (method, args) => {
        const out = services.call(method, JSON.stringify(args ?? null));
        return out === null ? null : (JSON.parse(out) as unknown);
      },
    };
  }
  return factory({ native, game, env, local });
}
