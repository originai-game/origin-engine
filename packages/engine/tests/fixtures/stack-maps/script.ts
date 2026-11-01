// A browser module that throws when called (tests/sourcemap.test.ts maps its minified stack).
interface Lantern {
  readonly lit: boolean;
}

function light(lantern: Lantern): never {
  throw new Error(`the lantern is ${lantern.lit ? "lit" : "out"}`);
}

export default function run(): void {
  light({ lit: false });
}
