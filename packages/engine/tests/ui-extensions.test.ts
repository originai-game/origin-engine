import { expect, test } from "bun:test";
import { registerUiExtension, uiExtensionNames } from "../src/ui-extensions.ts";

test("kits retain typed namespaces across re-imports without replacing existing callers", () => {
  const components = { speed: (value: number, unit: string) => `${value} ${unit}` };
  const kit = registerUiExtension("test-racing", components);
  expect(kit.speed(87, "mph")).toBe("87 mph");
  expect(Object.isFrozen(kit)).toBe(true);
  expect(registerUiExtension("test-racing", components)).toBe(kit);
  const reloaded = registerUiExtension("test-racing", { speed: () => "new" });
  expect(reloaded.speed()).toBe("new");
  expect(kit.speed(87, "mph")).toBe("87 mph");
  expect(() => registerUiExtension("../test-racing", components)).toThrow("Invalid");
  expect(uiExtensionNames()).toContain("test-racing");
});
