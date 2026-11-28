import { file } from '@origingame/engine/author';
import { images } from './parts.ts';

// Pixel: carved wood frames around parchment (Stardew Valley), a sword
// cursor and gold for the current choice (Blasphemous), whole art pixels
// and stepped motion. Light scheme: the parchment is the ground the text
// reads on, so the inks are its dark browns.
export default file.theme({
  "about": "Retro pixel: carved wood frames around parchment, a sword cursor, gold for the current choice, a pixel face, stepped motion (Stardew Valley, Blasphemous, Sonic Mania).",
  "color": { "source": "#e89a44", "accent": "#ffd23d", "neutral": "#6b3415", "variant": "fidelity", "scheme": "light" },
  "material": "pixel",
  "shape": "pixel",
  "type": "pixel",
  "motion": "stepped",
  "effects": "pixel-burst",
  "sound": "chiptune",
  "icons": "pixel",
  "pack": {
    "images": images,
    "theme": {
      "colors": {
        "fg": { "light": "#3b1d0b", "dark": "#ffe6b3" }, "fg-muted": { "light": "#6e3d18", "dark": "#e6c08a" }, "fg-subtle": { "light": "#9a6a3e", "dark": "#b38b5c" },
        "part-fg": "#3b1d0b", "part-on": "#fff1c2", "part-gold": "#ffd23d", "part-night": "#1d1430",
        "accent": "#2f8f37", "accent-fg": "#ffffff", "track": "#1d1430",
      },
      "duration": {"ready-pulse": 1100, "flash": 150, "damage": 750, "damage-stagger": 50, "trail-hold": 320, "trail": 0, "hit": 320, "arc": 550},
      "scale": {"ready-pulse": 1.0, "damage-pop": 1.35, "combo-pop": 1.2, "press": 0.96},
      "opacity": {"sweep": 0.72, "flash": 0.85},
      "stroke": {"arc": 12}
    },
    "variants": {
      "Ability.box": "size-15 compact:size-10", "Ability.box.ultimate": "size-22 compact:size-13",
      "Panel.frame": "nine-panel scheme-light text-part-fg",
      "Plate.frame": "nine-plate scheme-light text-part-fg",
      "Slot.frame": "nine-slot scheme-light text-part-fg",
      "Dialog.frame": "flex-col gap-4 w-115 max-w-full max-h-[85vh] overflow-y-auto m-auto px-7 pt-5 pb-7 compact:px-5 nine-panel scheme-light text-part-fg",
      "Dialog.title": "self-center px-6 py-1.5 nine-header font-display text-xl leading-tight text-part-on",
      "Dialog.body": "text-base leading-normal text-fg-muted",
      "Button.plate": "relative nine-button text-part-fg pb-1 hover:brightness-110 active:nine-button-down active:pb-0 active:pt-1 aria-pressed:nine-button-down selected:nine-button-down disabled:grayscale disabled:opacity-60",
      "Button.plate.primary": "nine-button-primary text-part-on active:nine-button-primary-down",
      "Button.plate.danger": "nine-button-danger text-part-on active:nine-button-danger-down",
      "Button.plate.ghost": "nine-none bg-transparent text-part-fg hover:bg-hover",
      "Button.plate.ghost-muted": "nine-none bg-transparent text-fg-muted hover:text-part-fg",
      "Button.plate.ghost-faint": "nine-none bg-transparent text-fg-subtle hover:text-fg-muted",
      "Button.plate.outline": "nine-plate scheme-light text-part-fg",
      "Button.plate.menu": "relative w-full justify-start py-1 pl-10 pr-4 nine-none bg-transparent text-part-fg data-current:text-part-fg disabled:text-fg-subtle",
      "Button.plate.menu-danger": "relative w-full justify-start py-1 pl-10 pr-4 nine-none bg-transparent text-fg-muted data-current:text-part-fg disabled:text-fg-subtle",
      "MenuList.indicator": "absolute inset-0 nine-entry-on pointer-events-none glide-menu",
      "MenuList.indicator.danger": "",
      "MenuList.cursor": "absolute left-1 top-1/2 w-6 h-4.5 -translate-y-1/2 nine-cursor",
      "MenuList.label": "text-start text-2xl leading-tight font-display",
      "MenuList.label.md": "text-xl",
      "MenuList.line": "text-start text-sm text-fg-muted whitespace-nowrap overflow-hidden",
      "MenuList.chevron": "hidden",
      "Screen.heading": "font-display text-display-xl leading-none text-part-gold text-stroke-4 text-stroke-part-fg",
      "Screen.note": "text-sm text-fg-muted text-stroke-2 text-stroke-part-fg",
      "Screen.prompt": "flex-row items-center gap-x-3 px-6 py-2 nine-plate scheme-light text-part-fg font-display text-xl",
      "Screen.veil": "bg-scrim",
      "Title.wordmark": "font-display text-display-3xl leading-none text-part-gold text-stroke-4 text-stroke-part-fg",
      "Checkbox.indicator": "flex items-center justify-center size-5 shrink-0 nine-check checked:nine-check-on disabled:grayscale",
      "Checkbox.mark": "hidden",
      "Panel.body": "gap-4 px-7 pt-5 pb-7",
      "Panel.title": "self-center px-6 py-1.5 nine-header font-display text-xl leading-tight text-part-on",
      "Tab.list": "flex-row items-end gap-1 px-2",
      "Tab.item": "flex-row items-center justify-center gap-2 min-h-11 px-4 pt-1 nine-tab text-part-on selected:nine-tab-on selected:text-part-fg disabled:opacity-60",
      "Tab.mark": "hidden",
      "Progress.track": "w-full h-4 p-1 nine-track overflow-hidden",
      "Progress.fill": "nine-range",
      "Slider.track": "relative grow h-4 nine-track disabled:grayscale",
      "Slider.range": "absolute left-1 top-1 bottom-1 nine-range",
      "Slider.thumb": "absolute top-1/2 w-5 h-6 -translate-x-1/2 -translate-y-1/2 nine-thumb",
      "Switch.track": "relative h-7 w-14 shrink-0 nine-track checked:nine-range disabled:grayscale",
      "Switch.thumb": "absolute top-0.5 left-0.5 w-5 h-6 nine-thumb checked:translate-x-7",
      "Bar.frame": "h-4 nine-bar overflow-hidden px-1 py-1",
      "Tooltip.frame": "max-w-64 px-3 py-2 nine-plate scheme-light text-part-fg text-xs",
      "Toast.frame": "flex-row items-center gap-3 w-full max-w-96 min-h-12 px-5 py-3 nine-plate scheme-light text-part-fg pointer-events-auto",
      "Toast.frame.danger": "",
      "Toast.frame.success": "",
      "Row.frame": "nine-slot scheme-light text-part-fg",
      "Row.frame.selected": "nine-entry-on text-part-fg",
      "Chip.frame": "nine-plate scheme-light text-part-fg"
    }
  }
});
