import { file } from '@origingame/engine/author';
import { images } from './parts.ts';

// Neon: luminous lines on black. Chamfered frames drawn only as glowing
// cyan lines over dark glass (Octahedron), wide technical capitals, the
// current choice lit magenta with a yellow chevron (Thumper, RUINER).
export default file.theme({
  "about": "Neon: glowing cyan line frames over dark glass, wide technical capitals, a magenta underline and a yellow chevron for the current choice, snappy motion (Thumper, Octahedron, RUINER).",
  "color": { "source": "#36f1ff", "accent": "#ff2bd6", "neutral": "#1a1830", "variant": "vibrant", "scheme": "dark" },
  "material": "glass",
  "shape": "cut4",
  "type": "techno",
  "motion": "snappy",
  "ornament": "none",
  "effects": "glow",
  "sound": "arcade",
  "icons": "glow",
  "pack": {
    "images": images,
    "theme": {
      "fontSize": { "2xs": [13, 1.25], "xs": [14, 1.3], "meta": [15, 1.3], "caption": [15, 1.3], "sm": [16, 1.35], "body": [17, 1.4], "reading": [18, 1.45], "base": [18, 1.35], "lg": [21, 1.3], "xl": [24, 1.25], "2xl": [28, 1.2] },
      "colors": {
        "fg": { "light": "#0b0a18", "dark": "#e8fbff" }, "fg-muted": { "light": "#2f3c52", "dark": "#9fc7d0" }, "fg-subtle": { "light": "#56627a", "dark": "#7fa9b3" },
        "part-fg": "#e8fbff", "part-on": "#04040a", "part-gold": "#ffe94a", "part-cyan": "#36f1ff", "part-magenta": "#ff2bd6",
        "accent": "#36f1ff", "accent-fg": "#04040a", "track": "#0c6f7a"
      },
      "textShadow": { "wordmark": "0 0 6px #36f1ff, 0 0 18px #36f1ff, 0 0 36px #ff2bd6", "neon": "0 0 8px #36f1ffcc", "neon-hot": "0 0 10px #ff2bd6" },
      "duration": {"ready-pulse": 500, "flash": 120, "damage": 700, "damage-stagger": 40, "trail-hold": 240, "trail": 380, "hit": 260, "arc": 480},
      "scale": {"ready-pulse": 1.06, "damage-pop": 1.45, "combo-pop": 1.3, "press": 0.95},
      "opacity": {"sweep": 0.6, "flash": 0.85},
      "stroke": {"arc": 10}
    },
    "variants": {
      "Ability.box": "size-15 compact:size-10", "Ability.box.ultimate": "size-22 compact:size-13",
      "Screen.heading": "font-display font-black text-display-xl leading-none uppercase tracking-label text-fg text-shadow-wordmark",
      "Screen.note": "text-xs uppercase tracking-label text-fg",
      "Screen.prompt": "flex-row items-center gap-x-3 py-2 px-6 nine-button scheme-dark font-display font-bold text-lg uppercase tracking-label text-part-cyan text-shadow-neon",
      "Screen.veil": "bg-scrim",
      "Title.wordmark": "font-display font-black text-display-3xl leading-none uppercase tracking-display-3xl text-part-fg text-shadow-wordmark",
      "Panel.frame": "nine-panel scheme-dark text-part-fg",
      "Panel.body": "gap-4 px-8 pt-7 pb-8",
      "Panel.title": "font-display font-bold text-xl uppercase tracking-label text-part-cyan text-shadow-neon",
      "Plate.frame": "nine-plate scheme-dark text-part-fg",
      "Slot.frame": "nine-slot scheme-dark text-part-fg",
      "Chip.frame": "nine-plate scheme-dark text-part-fg",
      "Dialog.frame": "flex-col gap-4 w-115 max-w-full max-h-[85vh] overflow-y-auto m-auto px-8 pt-7 pb-8 nine-panel scheme-dark text-part-fg",
      "Dialog.title": "font-display font-bold text-xl leading-tight uppercase tracking-label text-part-magenta text-shadow-neon-hot",
      "Dialog.body": "text-base leading-normal text-fg-muted",
      "Button.plate": "relative nine-button text-part-fg font-bold uppercase tracking-label text-base px-6 hover:text-part-cyan active:nine-button-down aria-pressed:nine-button-down selected:nine-button-down disabled:opacity-40",
      "Button.plate.primary": "nine-button-primary text-part-on active:nine-button-primary-down",
      "Button.plate.danger": "nine-button-danger text-part-fg active:nine-button-danger-down",
      "Button.plate.ghost": "nine-none bg-transparent text-part-fg hover:text-part-cyan",
      "Button.plate.ghost-muted": "nine-none bg-transparent text-fg-muted hover:text-part-fg",
      "Button.plate.ghost-faint": "nine-none bg-transparent text-fg-subtle hover:text-fg-muted",
      "Button.plate.outline": "nine-button text-part-fg",
      "Button.plate.menu": "relative w-full justify-start pt-1.5 pb-3 pl-10 pr-4 nine-none bg-transparent text-fg-muted data-current:text-part-fg disabled:text-fg-subtle",
      "Button.plate.menu-danger": "relative w-full justify-start pt-1.5 pb-3 pl-10 pr-4 nine-none bg-transparent text-fg-muted data-current:text-part-fg disabled:text-fg-subtle",
      "MenuList.indicator": "absolute inset-0 nine-entry-on pointer-events-none transition-layout duration-120 ease-out glide-menu",
      "MenuList.indicator.danger": "",
      "MenuList.cursor": "absolute left-2 top-1/2 w-4 h-5 -translate-y-1/2 nine-cursor animate-pulse",
      "MenuList.label": "text-start font-display font-bold text-2xl leading-tight uppercase tracking-label group-data-current/entry:text-shadow-neon-hot",
      "MenuList.label.md": "text-xl",
      "MenuList.line": "text-start text-sm uppercase tracking-label text-fg-muted whitespace-nowrap overflow-hidden",
      "MenuList.chevron": "hidden",
      "Tab.list": "flex-row items-end gap-2",
      "Tab.item": "flex-row items-center justify-center gap-2 min-h-11 px-4 font-bold uppercase tracking-label text-fg-muted hover:text-part-fg selected:nine-tab-on selected:text-part-cyan selected:text-shadow-neon disabled:opacity-40",
      "Tab.mark": "hidden",
      "Progress.track": "w-full h-4 nine-track overflow-hidden",
      "Progress.fill": "nine-range",
      "Slider.track": "relative grow h-4 nine-track disabled:opacity-40",
      "Slider.range": "absolute left-0 top-0 bottom-0 nine-range",
      "Slider.thumb": "absolute top-1/2 size-6 -translate-x-1/2 -translate-y-1/2 nine-thumb",
      "Switch.track": "relative h-7 w-14 shrink-0 nine-slot checked:nine-plate disabled:opacity-40",
      "Switch.thumb": "absolute top-0.5 left-0.5 size-6 nine-thumb transition-transform duration-120 checked:translate-x-7",
      "Checkbox.indicator": "flex items-center justify-center size-6 shrink-0 nine-check checked:nine-check-on disabled:opacity-40",
      "Checkbox.mark": "hidden",
      "Bar.frame": "h-4.5 nine-bar overflow-hidden px-1 py-1",
      "Tooltip.frame": "max-w-64 px-3 py-2 nine-plate scheme-dark text-part-fg text-xs uppercase tracking-label",
      "Toast.frame": "flex-row items-center gap-3 w-full max-w-96 min-h-12 px-5 py-3 nine-plate scheme-dark text-part-fg pointer-events-auto",
      "Toast.frame.danger": "",
      "Toast.frame.success": "",
      "Row.frame": "nine-slot scheme-dark text-part-fg",
      "Row.frame.selected": "nine-entry-on text-part-fg"
    }
  }
});
