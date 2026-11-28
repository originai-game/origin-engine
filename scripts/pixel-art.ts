// Pixel canvases for the deterministic fixture art generators.
import { writeFileSync, mkdirSync } from "node:fs";
import { dirname } from "node:path";
import { encodePngBest } from "../packages/engine/src/node/png.ts";
export const rgb = (h: string) => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16));
export const mix = (a: number[], b: number[], t: number) => a.map((x, i) => Math.trunc(x + (b[i]! - x) * t));
export class PixelArt {
  rgba: Uint8Array;
  constructor(
    readonly width: number,
    readonly height: number,
  ) {
    this.rgba = new Uint8Array(width * height * 4);
  }
  put(x: number, y: number, color: string | number[], alpha = 255): void {
    if (x < 0 || x >= this.width || y < 0 || y >= this.height) return;
    const c = typeof color === "string" ? rgb(color) : color;
    this.rgba.set([c[0]!, c[1]!, c[2]!, c[3] ?? alpha], (y * this.width + x) * 4);
  }
  get(x: number, y: number): number[] {
    return [...this.rgba.slice((y * this.width + x) * 4, (y * this.width + x) * 4 + 4)];
  }
  rect(x: number, y: number, w: number, h: number, color: string | number[]): void {
    for (let j = y; j < y + h; j++) for (let i = x; i < x + w; i++) this.put(i, j, color);
  }
  blit(src: PixelArt, x: number, y: number): void {
    for (let j = 0; j < src.height; j++) for (let i = 0; i < src.width; i++) this.put(i + x, j + y, src.get(i, j));
  }
  bounds(): [number, number, number, number] {
    let x0 = this.width,
      y0 = this.height,
      x1 = 0,
      y1 = 0;
    for (let y = 0; y < this.height; y++)
      for (let x = 0; x < this.width; x++)
        if (this.rgba[(y * this.width + x) * 4 + 3]) {
          x0 = Math.min(x0, x);
          y0 = Math.min(y0, y);
          x1 = Math.max(x1, x + 1);
          y1 = Math.max(y1, y + 1);
        }
    return [x0, y0, x1, y1];
  }
  crop([x0, y0, x1, y1]: number[]): PixelArt {
    const out = new PixelArt(x1! - x0!, y1! - y0!);
    for (let y = 0; y < out.height; y++)
      for (let x = 0; x < out.width; x++) {
        if (x + x0! >= 0 && y + y0! >= 0 && x + x0! < this.width && y + y0! < this.height) out.put(x, y, this.get(x + x0!, y + y0!));
      }
    return out;
  }
  outlined(): PixelArt {
    const src = this.rgba.slice();
    for (let y = 0; y < this.height; y++)
      for (let x = 0; x < this.width; x++) {
        if (src[(y * this.width + x) * 4 + 3]) continue;
        for (const [dx, dy] of [
          [1, 0],
          [-1, 0],
          [0, 1],
          [0, -1],
        ]) {
          const nx = x + dx!,
            ny = y + dy!;
          if (nx >= 0 && nx < this.width && ny >= 0 && ny < this.height && src[(ny * this.width + nx) * 4 + 3]) {
            this.put(x, y, [42, 27, 30]);
            break;
          }
        }
      }
    return this;
  }
  save(path: string): void {
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, encodePngBest(this));
  }
}
