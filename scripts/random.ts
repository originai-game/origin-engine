// MT19937 with integer-seed initialization, 53-bit uniform doubles and
// rejection-sampled bounded integers. Shared by the offline art generators
// to preserve their seeded lattices and discrete choices.
export class Random {
  private mt = new Uint32Array(624);
  private at = 624;
  constructor(seed: number) {
    const mt = this.mt;
    mt[0] = 19650218;
    for (let i = 1; i < 624; i++) mt[i] = (Math.imul(mt[i - 1]! ^ (mt[i - 1]! >>> 30), 1812433253) + i) >>> 0;
    let at = 1;
    for (let k = 0; k < 624; k++) {
      mt[at] = ((mt[at]! ^ Math.imul(mt[at - 1]! ^ (mt[at - 1]! >>> 30), 1664525)) + seed) >>> 0;
      if (++at >= 624) {
        mt[0] = mt[623]!;
        at = 1;
      }
    }
    for (let k = 0; k < 623; k++) {
      mt[at] = ((mt[at]! ^ Math.imul(mt[at - 1]! ^ (mt[at - 1]! >>> 30), 1566083941)) - at) >>> 0;
      if (++at >= 624) {
        mt[0] = mt[623]!;
        at = 1;
      }
    }
    mt[0] = 0x80000000;
  }
  uint32(): number {
    const mt = this.mt;
    if (this.at >= 624) {
      for (let i = 0; i < 624; i++) {
        const y = (mt[i]! & 0x80000000) | (mt[(i + 1) % 624]! & 0x7fffffff);
        mt[i] = mt[(i + 397) % 624]! ^ (y >>> 1) ^ (y & 1 ? 0x9908b0df : 0);
      }
      this.at = 0;
    }
    let y = mt[this.at++]!;
    y ^= y >>> 11;
    y ^= (y << 7) & 0x9d2c5680;
    y ^= (y << 15) & 0xefc60000;
    y ^= y >>> 18;
    return y >>> 0;
  }
  random(): number {
    return ((this.uint32() >>> 5) * 67108864 + (this.uint32() >>> 6)) / 9007199254740992;
  }
  randrange(start: number, stop?: number): number {
    if (stop == null) {
      stop = start;
      start = 0;
    }
    const n = stop - start,
      bits = Math.floor(Math.log2(n)) + 1;
    let r: number;
    do {
      r = this.uint32() >>> (32 - bits);
    } while (r >= n);
    return start + r;
  }
  randint(a: number, b: number): number {
    return this.randrange(a, b + 1);
  }
  uniform(a: number, b: number): number {
    return a + (b - a) * this.random();
  }
  choice<T>(values: readonly T[]): T {
    return values[this.randrange(values.length)]!;
  }
}
