// CSS custom properties from @origingame/design (packages/design/tokens.json).
// The site is the stage: the dark scheme's colours, the shared type ramp,
// radii, spacing, motion and material values. Nothing here is a new value.
import tokens from "../../packages/design/tokens.json";

type Step = { size: string; lineHeight: string; letterSpacing?: string };

export function tokenCss(): string {
  const v: string[] = [];
  const colors = { ...tokens.color, ...tokens.dark.color } as Record<string, string>;
  for (const [k, val] of Object.entries(colors)) v.push(`--color-${k}:${val}`);
  for (const [k, val] of Object.entries(tokens.radius)) v.push(`--radius-${k}:${val}`);
  for (const [k, val] of Object.entries(tokens.space)) v.push(`--space-${k}:${val}`);
  for (const [k, val] of Object.entries(tokens.motion.duration)) v.push(`--dur-${k}:${val}`);
  for (const [k, val] of Object.entries(tokens.motion.easing)) v.push(`--ease-${k}:${val}`);
  for (const [k, val] of Object.entries(tokens.vars)) v.push(`--${k}:${val}`);
  for (const [k, s] of Object.entries(tokens.font.scale as Record<string, Step>)) {
    v.push(`--text-${k}:${s.size}`, `--leading-${k}:${s.lineHeight}`, `--tracking-${k}:${s.letterSpacing ?? "0em"}`);
  }
  v.push(`--font-sans:${tokens.font.sans}`, `--font-display:${tokens.font.display}`, `--font-mono:${tokens.font.mono}`);
  const darkShadow = (tokens.dark as { shadow?: Record<string, string> }).shadow ?? tokens.shadow;
  for (const [k, val] of Object.entries(darkShadow)) v.push(`--shadow-${k}:${val}`);
  const cjk = tokens.font.cjk;
  const zh = [`--cjk-tracking:${cjk.letterSpacing}`];
  for (const [k, val] of Object.entries(cjk.lineHeight)) zh.push(`--leading-${k}:${val}`, `--tracking-${k}:${cjk.letterSpacing}`);
  return `:root{${v.join(";")}}:root:lang(zh){${zh.join(";")}}`;
}
