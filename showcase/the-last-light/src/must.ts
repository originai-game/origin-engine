/** A result's value; a refusal is a bug in the scene, so it throws with the engine's reason. */
export function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}
