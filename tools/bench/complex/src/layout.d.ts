// The types of layout.js (plain JavaScript so the three.js page imports it as served).

export interface Pose { x: number; y: number; z: number; yaw: number; scale: number }
export interface ModelEntry { file: string; count: number; moving?: boolean }
export interface Character { model: string; clip: string; x: number; y: number; z: number; yaw: number; speed: number; time: number }
export interface Light {
  kind: "spot" | "point";
  x: number; y: number; z: number;
  tx?: number; ty?: number; tz?: number;
  color: [number, number, number];
  intensity: number;
  distance: number;
  angle?: number;
  penumbra?: number;
}
export interface Emitter { fx: "fire" | "smoke"; x: number; y: number; z: number; seed: number }
export interface Fx {
  max: number; rate: number; life: [number, number]; speed: [number, number]; cone: number; radius: number;
  size: [number, number]; sizeOverLife: [number, number]; color: number[][]; gravity: number; blend: "additive" | "alpha"; texture: string;
}
export interface View { eye: { x: number; y: number; z: number }; at: { x: number; y: number; z: number }; fov: number; near: number; far: number }

export const SIZE: number;
export const GRID: number;
export const LAKE: { x: number; z: number; r: number };
export const VILLAGE: { x: number; z: number; r: number; y: number };
export const FIELD: { x: number; z: number };
export const PARK: { x: number; z: number };
export const SUN: { x: number; y: number; z: number };
export const SHADOW_DISTANCE: number;
export const MODELS: ModelEntry[];
export const WALKER_CLIPS: string[];
export const VILLAGER_CLIPS: string[];
export const CAMPFIRES: { x: number; z: number }[];
export const FX: { fire: Fx; smoke: Fx };
export function height(x: number, z: number): number;
export function slope(x: number, z: number): number;
export function rng(seed: number): () => number;
export function placements(): Record<string, Pose[]>;
export function ghostPose(i: number, t: number): Pose;
export function characters(): Character[];
export function lights(): Light[];
export function emitters(): Emitter[];
export function cameraAt(t: number): View;
