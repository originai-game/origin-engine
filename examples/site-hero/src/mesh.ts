// Small helpers shared by the scene's modules.

export type V3 = { x: number; y: number; z: number };

export const v3 = (x: number, y: number, z: number): V3 => ({ x, y, z });

export function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}
