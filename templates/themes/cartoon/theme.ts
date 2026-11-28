import { file } from '@origingame/engine/author';
import { images } from './parts.ts';

// Cartoon: a thick warm-dark outline round everything, glossy coloured
// buttons with a lip (blue, yellow primary, red danger), a cream panel under
// an orange header plank, an orange button under a white halo for the
// current choice, chunky capitals (Brawl Stars, Castle Crashers, Rayman
// Legends). Light scheme: the cream is the ground, in the outline's ink.
export default file.theme({
  "about": "Cartoon: a thick warm-dark outline, glossy coloured buttons with a lip, a cream panel under an orange header, an orange button under a white halo for the current choice, chunky capitals, bouncy motion (Brawl Stars, Castle Crashers, Rayman Legends).",
  "color": { "source": "#ff7a1a", "accent": "#2a6fe0", "neutral": "#8a7560", "variant": "vibrant", "scheme": "light" },
  "material": "flat",
  "shape": "round",
  "type": "cartoon",
  "motion": "bouncy",
  "ornament": "none",
  "effects": "sheen",
  "sound": "cute",
  "icons": "sticker",
  "density": "touch",
  "pack": {
    "images": images,
    "theme": {
      "fontSize": { "2xs": [13, 1.25], "xs": [14, 1.3], "meta": [15, 1.3], "caption": [15, 1.3], "sm": [16, 1.35], "body": [17, 1.4], "reading": [18, 1.45], "base": [18, 1.35], "lg": [21, 1.3], "xl": [24, 1.25], "2xl": [28, 1.2] },
      "colors": {
        "fg": { "light": "#22142e", "dark": "#fff6e3" }, "fg-muted": { "light": "#4f3d5c", "dark": "#e6d6bd" }, "fg-subtle": { "light": "#62506d", "dark": "#b8a88f" },
        "part-fg": "#22142e", "part-on": "#ffffff", "part-gold": "#ffc61a", "part-orange": "#ff7a1a",
        "accent": "#2fae45", "accent-fg": "#ffffff", "track": "#3a2f52"
      },
      "textShadow": { "wordmark": "0 5px 0 #22142e", "pop": "0 2px 0 #22142e" },
      "duration": {"ready-pulse": 950, "flash": 280, "damage": 950, "damage-stagger": 70, "trail-hold": 350, "trail": 650, "hit": 480, "arc": 750},
      "scale": {"ready-pulse": 1.09, "damage-pop": 1.65, "combo-pop": 1.38, "press": 0.88},
      "opacity": {"sweep": 0.58, "flash": 0.72},
      "stroke": {"arc": 15}
    },
    "variants": {
      "Ability.box": "size-18 compact:size-12", "Ability.box.ultimate": "size-26 compact:size-15",
      "Screen.heading": "font-display text-display-2xl leading-none text-part-gold text-stroke-4 text-stroke-part-fg text-shadow-wordmark -rotate-1",
      "Screen.note": "text-sm font-bold text-fg",
      "Screen.prompt": "flex-row items-center gap-x-3 px-7 pt-2 pb-4 nine-button-primary scheme-light text-part-on text-stroke-3 text-stroke-part-fg font-display text-xl",
      "Screen.veil.full": "bg-scrim",
      "Title.wordmark": "font-display text-display-4xl leading-none text-part-gold text-stroke-6 text-stroke-part-fg text-shadow-wordmark -rotate-2",
      "Panel.frame": "nine-panel scheme-light text-part-fg",
      "Panel.body": "gap-4 px-7 pt-5 pb-9",
      "Panel.title": "self-center -mt-1 px-8 pt-2 pb-3 nine-header font-display text-2xl leading-none text-part-on text-stroke-3 text-stroke-part-fg text-shadow-pop",
      "Plate.frame": "nine-plate scheme-light text-part-fg",
      "Slot.frame": "nine-slot scheme-light text-part-fg",
      "Chip.frame": "nine-plate scheme-light text-part-fg",
      "Dialog.frame": "flex-col gap-4 w-115 max-w-full max-h-[85vh] overflow-y-auto m-auto px-7 pt-5 pb-9 nine-panel scheme-light text-part-fg",
      "Dialog.title": "self-center px-8 pt-2 pb-3 nine-header font-display text-2xl leading-none text-part-on text-stroke-3 text-stroke-part-fg text-shadow-pop",
      "Dialog.body": "text-base font-semibold leading-normal text-fg-muted text-center",
      "Button.plate": "relative nine-button text-part-on font-display text-lg px-7 pb-2 text-stroke-3 text-stroke-part-fg active:nine-button-down active:pb-1 active:pt-1 aria-pressed:nine-button-down selected:nine-button-down disabled:grayscale disabled:opacity-60",
      "Button.plate.primary": "nine-button-primary active:nine-button-primary-down",
      "Button.plate.danger": "nine-button-danger active:nine-button-danger-down",
      "Button.plate.ghost": "nine-none bg-transparent text-part-fg text-stroke-0 hover:text-part-orange",
      "Button.plate.ghost-muted": "nine-none bg-transparent text-fg-muted text-stroke-0 hover:text-part-fg",
      "Button.plate.ghost-faint": "nine-none bg-transparent text-fg-subtle text-stroke-0 hover:text-fg-muted",
      "Button.plate.outline": "nine-plate scheme-light text-part-fg text-stroke-0",
      "Button.plate.menu": "relative w-full justify-start py-2 pl-7 pr-5 pb-3 nine-entry text-part-on text-stroke-3 text-stroke-part-fg data-current:-rotate-1 data-current:scale-105 disabled:grayscale",
      "Button.plate.menu-danger": "relative w-full justify-start py-2 pl-7 pr-5 pb-3 nine-button-danger text-part-on text-stroke-3 text-stroke-part-fg data-current:-rotate-1 data-current:scale-105 disabled:grayscale",
      "MenuList.indicator": "absolute -inset-1 nine-entry-on pointer-events-none glide-menu",
      "MenuList.indicator.danger": "",
      "MenuList.cursor": "hidden",
      "MenuList.label": "text-start font-display text-2xl leading-tight",
      "MenuList.label.md": "text-xl",
      "MenuList.line": "text-start text-sm font-bold text-part-on text-stroke-2 text-stroke-part-fg whitespace-nowrap overflow-hidden",
      "MenuList.chevron": "hidden",
      "Tab.list": "flex-row items-end gap-2",
      "Tab.item": "flex-row items-center justify-center gap-2 min-h-11 px-5 pb-2 nine-tab font-display text-base text-part-fg selected:nine-tab-on disabled:opacity-60",
      "Tab.mark": "hidden",
      "Progress.track": "w-full h-5 nine-track overflow-hidden",
      "Progress.fill": "nine-range",
      "Slider.track": "relative grow h-5 nine-track disabled:grayscale",
      "Slider.range": "absolute left-0 top-0 bottom-0 nine-range",
      "Slider.thumb": "absolute top-1/2 size-8 -translate-x-1/2 -translate-y-1/2 nine-thumb",
      "Switch.track": "relative h-8 w-15 shrink-0 nine-track checked:nine-range disabled:grayscale",
      "Switch.thumb": "absolute top-0 left-0 size-8 nine-thumb transition-transform duration-300 ease-spring-[400,14] checked:translate-x-7",
      "Checkbox.indicator": "flex items-center justify-center size-7 shrink-0 nine-check checked:nine-check-on disabled:grayscale",
      "Checkbox.mark": "hidden",
      "Bar.frame": "h-5 nine-bar overflow-hidden px-1.5 py-1.5",
      "Tooltip.frame": "max-w-64 px-4 pt-2 pb-3 nine-plate scheme-light text-part-fg text-sm font-bold",
      "Toast.frame": "flex-row items-center gap-3 w-full max-w-96 min-h-12 px-6 pt-3 pb-5 nine-plate scheme-light text-part-fg pointer-events-auto",
      "Toast.frame.danger": "",
      "Toast.frame.success": "",
      "Row.frame": "nine-slot scheme-light text-part-fg",
      "Row.frame.selected": "nine-entry-on text-part-on"
    }
  }
});
