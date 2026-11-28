import { file } from '@origingame/engine/author';
import { images } from './parts.ts';

// Comic: a comic page's lettering and panels. Charcoal panels under a 3 pt
// ink rim and a hard ink drop, slanted slabs for buttons, tabs and the
// current entry (yellow, dropped on red), a hand-drawn arrow at the current
// entry and Bangers capitals (Hi-Fi Rush, Borderlands, Marvel Rivals).
// Dark scheme: the charcoal is the ground most text reads on; yellow, cream
// and red slabs carry their own inks.
export default file.theme({
  "about": "Comic: charcoal panels under a thick ink rim and a hard drop, slanted yellow and red slabs, a hand-drawn arrow, loud comic capitals, punchy motion (Hi-Fi Rush, Borderlands, Marvel Rivals).",
  "color": { "source": "#ffd21f", "accent": "#d92631", "neutral": "#25222c", "variant": "vibrant", "scheme": "dark" },
  "material": "flat",
  "shape": "cut",
  "type": "comic",
  "motion": "snappy",
  "effects": "spark",
  "sound": "arcade",
  "icons": "sticker",
  "pack": {
    "images": images,
    "theme": {
      "fontSize": { "2xs": [13, 1.25], "xs": [14, 1.3], "meta": [15, 1.3], "caption": [15, 1.3], "sm": [16, 1.35], "body": [17, 1.4], "reading": [18, 1.45], "base": [18, 1.35], "lg": [21, 1.3], "xl": [24, 1.25], "2xl": [28, 1.2], "3xl": [32, 1.2], "4xl": [38, 1.15] },
      "colors": {
        "stage-hi": "#2b3a6b", "stage-lo": "#0e1020", "stage-glow": "#ffd23d",
        "fg": { "light": "#17151c", "dark": "#f7efdc" }, "fg-muted": { "light": "#3b3745", "dark": "#cfc5ad" }, "fg-subtle": { "light": "#55505e", "dark": "#9b927f" },
        "part-fg": "#0d0c10", "part-on": "#ffffff", "part-gold": "#ffd21f", "part-red": "#d92631",
        "accent": "#ffd21f", "accent-fg": "#0d0c10", "track": "#17151c"
      },
      "textShadow": { "wordmark": "5px 5px 0 #d92631, 8px 8px 0 #0d0c10", "punch": "3px 3px 0 #0d0c10" },
      "duration": {"ready-pulse": 800, "flash": 120, "damage": 700, "damage-stagger": 40, "trail-hold": 260, "trail": 420, "hit": 300, "arc": 500},
      "scale": {"ready-pulse": 1.08, "damage-pop": 1.6, "combo-pop": 1.35, "press": 0.94},
      "opacity": {"sweep": 0.7, "flash": 0.9},
      "stroke": {"arc": 14}
    },
    "variants": {
      "MenuList.frame": "flex-col nine-panel scheme-dark text-fg p-4",
      "Screen.glow": "bg-radial from-stage-glow to-transparent to-60% opacity-dim",
      "Screen.backdrop": "bg-linear-to-b from-stage-hi to-stage-lo",
      "Ability.box": "size-16 compact:size-11", "Ability.box.ultimate": "size-24 compact:size-14",
      "Screen.heading": "font-display text-display-2xl leading-none uppercase text-part-gold text-stroke-4 text-stroke-part-fg text-shadow-punch -rotate-2",
      "Screen.note": "text-sm font-semibold text-fg text-shadow-punch",
      "Screen.prompt": "flex-row items-center gap-x-3 pl-6 pr-8 pt-1.5 pb-3 nine-caption scheme-light text-part-fg font-display text-2xl uppercase",
      "Screen.veil.full": "bg-scrim",
      "Title.wordmark": "font-display text-display-4xl leading-none uppercase text-part-gold text-stroke-4 text-stroke-part-fg text-shadow-wordmark -rotate-3",
      "Panel.frame": "nine-panel scheme-dark text-fg",
      "Panel.body": "gap-4 pl-7 pr-9 pt-6 pb-9",
      "Panel.title": "self-start -ml-3 px-7 pt-1 pb-2 nine-header font-display text-4xl leading-none uppercase text-part-on",
      "Plate.frame": "nine-plate scheme-dark text-fg",
      "Slot.frame": "nine-slot scheme-dark text-fg",
      "Chip.frame": "nine-caption scheme-light text-part-fg",
      "Dialog.frame": "flex-col gap-4 w-115 max-w-full max-h-[85vh] overflow-y-auto m-auto pl-7 pr-9 pt-6 pb-9 nine-panel scheme-dark text-fg",
      "Dialog.title": "font-display text-display-sm leading-none uppercase text-part-gold text-shadow-punch",
      "Dialog.body": "text-base leading-normal text-fg-muted",
      "Button.plate": "relative nine-button text-part-fg font-display text-xl uppercase tracking-wide pl-6 pr-8 pb-1.5 active:nine-button-down active:translate-x-0.5 active:translate-y-0.5 aria-pressed:nine-button-down selected:nine-button-down disabled:grayscale disabled:opacity-60",
      "Button.plate.primary": "nine-button-primary text-part-fg active:nine-button-primary-down",
      "Button.plate.danger": "nine-button-danger text-part-on active:nine-button-danger-down",
      "Button.plate.ghost": "nine-none bg-transparent text-fg hover:text-part-gold",
      "Button.plate.ghost-muted": "nine-none bg-transparent text-fg-muted hover:text-fg",
      "Button.plate.ghost-faint": "nine-none bg-transparent text-fg-subtle hover:text-fg-muted",
      "Button.plate.outline": "nine-plate scheme-dark text-fg",
      "Button.plate.menu": "relative w-full justify-start py-1.5 pl-12 pr-8 nine-none bg-transparent text-fg data-current:text-part-fg disabled:text-fg-subtle",
      "Button.plate.menu-danger": "relative w-full justify-start py-1.5 pl-12 pr-8 nine-none bg-transparent text-fg-muted data-current:text-part-fg disabled:text-fg-subtle",
      "MenuList.indicator": "absolute inset-0 nine-entry-on pointer-events-none transition-layout duration-120 ease-out glide-menu",
      "MenuList.indicator.danger": "",
      "MenuList.cursor": "absolute -left-4 top-1/2 w-10 h-7 -translate-y-1/2 nine-cursor",
      "MenuList.label": "text-start font-display text-4xl leading-none uppercase tracking-wide",
      "MenuList.label.md": "text-3xl",
      "MenuList.line": "text-start text-sm font-semibold text-fg-muted group-data-current/entry:text-part-fg whitespace-nowrap overflow-hidden",
      "MenuList.chevron": "hidden",
      "Tab.list": "flex-row items-end gap-1",
      "Tab.item": "flex-row items-center justify-center gap-2 min-h-11 pl-5 pr-7 pb-1 nine-tab font-display text-xl uppercase text-fg selected:nine-tab-on selected:text-part-fg disabled:opacity-60",
      "Tab.mark": "hidden",
      "Progress.track": "w-full h-4 p-0.5 nine-track overflow-hidden",
      "Progress.fill": "nine-range",
      "Slider.track": "relative grow h-4 nine-track disabled:grayscale",
      "Slider.range": "absolute left-0.5 top-0.5 bottom-0.5 nine-range",
      "Slider.thumb": "absolute top-1/2 size-7 -translate-x-1/2 -translate-y-1/2 nine-thumb",
      "Switch.track": "relative h-7 w-14 shrink-0 nine-track checked:nine-range disabled:grayscale",
      "Switch.thumb": "absolute -top-0.5 left-0 size-7 nine-thumb checked:translate-x-7",
      "Checkbox.indicator": "flex items-center justify-center size-6 shrink-0 nine-check checked:nine-check-on disabled:grayscale",
      "Checkbox.mark": "hidden",
      "Bar.frame": "h-5 nine-bar overflow-hidden pl-1.5 pr-2.5 pt-1 pb-1.5",
      "Tooltip.frame": "max-w-64 pl-3 pr-4 pt-1.5 pb-2.5 nine-caption scheme-light text-part-fg text-sm font-semibold",
      "Toast.frame": "flex-row items-center gap-3 w-full max-w-96 min-h-12 pl-5 pr-6 pt-3 pb-4 nine-plate scheme-dark text-fg pointer-events-auto",
      "Toast.frame.danger": "",
      "Toast.frame.success": "",
      "Row.frame": "nine-slot scheme-dark text-fg",
      "Row.frame.selected": "nine-entry-on text-part-fg"
    }
  }
});
