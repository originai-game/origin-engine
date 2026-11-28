import { file } from '@origingame/engine/author';
import { images } from './parts.ts';

// Candy: toy plastic. Fat white pills with a deep lavender lip and a soft
// shadow, a pink primary, a yellow pill under a white rim for the current
// choice, fat rounded lettering (Fall Guys, Astro's Playroom). Light scheme:
// the white plastic is the ground, in deep navy.
export default file.theme({
  "about": "Candy: glossy toy plastic pills with a deep lip and a soft shadow, a pink primary, a yellow pill under a white rim for the current choice, fat rounded lettering, bouncy motion (Fall Guys, Astro's Playroom).",
  "color": { "source": "#e0185f", "accent": "#ffd23a", "neutral": "#5b3fb0", "variant": "vibrant", "scheme": "light" },
  "material": "plastic",
  "shape": "round",
  "type": "bubble",
  "motion": "bouncy",
  "ornament": "none",
  "effects": "sheen",
  "sound": "cute",
  "icons": "sticker",
  "density": "touch",
  "pack": {
    "images": images,
    "theme": {
      "colors": {
        "fg": "#2a1f5c", "fg-muted": "#4e4486", "fg-subtle": "#7d76ab",
        "part-fg": "#2a1f5c", "part-on": "#ffffff", "part-gold": "#ffd23a", "part-pink": "#e0185f",
        "accent": "#e0185f", "accent-fg": "#ffffff", "track": "#cdd2f0"
      },
      "textShadow": { "wordmark": "0 4px 0 #98093c, 0 7px 0 #2a1f5c", "pop": "0 3px 0 #2a1f5c" },
      "duration": {"ready-pulse": 900, "flash": 320, "damage": 1000, "damage-stagger": 80, "trail-hold": 400, "trail": 700, "hit": 500, "arc": 800},
      "scale": {"ready-pulse": 1.1, "damage-pop": 1.7, "combo-pop": 1.4, "press": 0.86},
      "opacity": {"sweep": 0.55, "flash": 0.6},
      "stroke": {"arc": 16}
    },
    "variants": {
      "Ability.box": "size-18 compact:size-12", "Ability.box.ultimate": "size-27 compact:size-16",
      "Title.wordmark": "font-display text-display-4xl leading-none text-part-on text-stroke-4 text-stroke-part-pink text-shadow-wordmark -rotate-2",
      "Panel.frame": "nine-panel text-part-fg",
      "Panel.body": "gap-4 px-8 pt-7 pb-10",
      "Panel.title": "self-center font-display text-3xl leading-none text-part-pink",
      "Plate.frame": "nine-plate text-part-fg",
      "Slot.frame": "nine-slot text-part-fg",
      "Chip.frame": "nine-plate text-part-fg",
      "Dialog.frame": "flex-col gap-4 w-115 max-w-full max-h-[85vh] overflow-y-auto m-auto px-8 pt-7 pb-10 nine-panel text-part-fg",
      "Dialog.title": "self-center font-display text-3xl leading-tight text-part-pink",
      "Dialog.body": "text-base font-medium leading-normal text-fg-muted text-center",
      "Button.plate": "relative nine-button text-part-fg font-display text-lg px-7 pb-2 hover:scale-105 active:nine-button-down active:pb-1 active:pt-1 aria-pressed:nine-button-down selected:nine-button-down disabled:grayscale disabled:opacity-60",
      "Button.plate.primary": "nine-button-primary text-part-on text-stroke-2 text-stroke-part-fg active:nine-button-primary-down",
      "Button.plate.danger": "nine-button-danger text-part-on text-stroke-2 text-stroke-part-fg active:nine-button-danger-down",
      "Button.plate.ghost": "nine-none bg-transparent text-part-fg hover:text-part-pink",
      "Button.plate.ghost-muted": "nine-none bg-transparent text-fg-muted hover:text-part-fg",
      "Button.plate.ghost-faint": "nine-none bg-transparent text-fg-subtle hover:text-fg-muted",
      "Button.plate.outline": "nine-plate text-part-fg",
      "Button.plate.menu": "relative w-full justify-start py-2 pl-7 pr-5 pb-3 nine-entry text-part-fg data-current:scale-105 disabled:text-fg-subtle",
      "Button.plate.menu-danger": "relative w-full justify-start py-2 pl-7 pr-5 pb-3 nine-entry text-fg-muted data-current:text-part-fg data-current:scale-105 disabled:text-fg-subtle",
      "MenuList.indicator": "absolute -inset-1 nine-entry-on pointer-events-none glide-menu",
      "MenuList.indicator.danger": "",
      "MenuList.cursor": "hidden",
      "MenuList.label": "text-start font-display text-2xl leading-tight",
      "MenuList.label.md": "text-xl",
      "MenuList.line": "text-start text-sm font-semibold text-fg-muted whitespace-nowrap overflow-hidden",
      "MenuList.chevron": "hidden",
      "Tab.list": "flex-row items-end gap-2",
      "Tab.item": "flex-row items-center justify-center gap-2 min-h-11 px-5 pb-2 nine-tab font-display text-base text-part-fg selected:nine-tab-on selected:text-part-on selected:text-stroke-2 selected:text-stroke-part-fg disabled:opacity-60",
      "Tab.mark": "hidden",
      "Slider.track": "relative grow h-5 nine-track disabled:grayscale",
      "Slider.range": "absolute left-0 top-0 bottom-0 nine-range",
      "Slider.thumb": "absolute top-1/2 size-8 -translate-x-1/2 -translate-y-1/2 nine-thumb",
      "Switch.track": "relative h-8 w-15 shrink-0 nine-track checked:nine-range disabled:grayscale",
      "Switch.thumb": "absolute top-0 left-0 size-8 nine-thumb transition-transform duration-300 ease-spring-[400,14] checked:translate-x-7",
      "Checkbox.indicator": "flex items-center justify-center size-7 shrink-0 nine-check checked:nine-check-on disabled:grayscale",
      "Checkbox.mark": "hidden",
      "Bar.frame": "h-5 nine-bar overflow-hidden px-1 pt-1 pb-1.5",
      "Tooltip.frame": "max-w-64 px-4 py-2 pb-3 nine-plate text-part-fg text-sm font-semibold",
      "Toast.frame": "flex-row items-center gap-3 w-full max-w-96 min-h-12 px-6 pt-3 pb-5 nine-plate text-part-fg pointer-events-auto",
      "Toast.frame.danger": "",
      "Toast.frame.success": "",
      "Row.frame": "nine-entry text-part-fg",
      "Row.frame.selected": "nine-entry-on text-part-fg"
    }
  }
});
