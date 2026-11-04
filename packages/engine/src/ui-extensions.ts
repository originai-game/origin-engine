/** Host-neutral kit namespaces beside the frozen og.ui API (docs/ui.md §17). */
const extensions = new Set<string>();

/** Register an explicitly imported kit, preserving its own function signatures. */
export function registerUiExtension<T extends object>(name: string, components: T): Readonly<T> {
  if (!/^[a-z][a-z0-9-]*$/.test(name)) throw new Error(`Invalid UI extension namespace: ${name}`);
  // Re-imports during hot reload may register a fresh object. Callers hold their
  // imported namespace, not a mutable global pointer to another kit's methods.
  extensions.add(name);
  return Object.freeze(components);
}

/** Names registered by explicitly imported kits; never evaluates extension code. */
export function uiExtensionNames(): readonly string[] {
  return [...extensions].sort();
}
