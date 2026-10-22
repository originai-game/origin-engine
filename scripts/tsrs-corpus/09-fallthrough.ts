// Corpus 09-fallthrough: noFallthroughCasesInSwitch
export function f(n: number): number {
  let r = 0;
  switch (n) {
    case 0:
      r = 1;
    case 1:
      r = 2;
      break;
  }
  return r;
}
