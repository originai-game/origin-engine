import { file } from '@origingame/engine/author';
import { images } from './parts.ts';

// Minimal: quiet type floating over the scene (Monument Valley, ABZU),
// matte cream plates with 45-degree facets where a container is needed
// (Bad North), a small gold diamond and a gold rim for the current choice.
// Light scheme: the cream is the ground text reads on, in slate.
export default file.theme({
  "about": "Minimal: wide light capitals, matte faceted cream plates with a hairline and a soft shadow, a gold diamond for the current choice, calm motion (Monument Valley, ABZU, Bad North).",
  "color": { "source": "#2d3440", "accent": "#e3a63b", "neutral": "#5d6470", "variant": "muted", "scheme": "light" },
  "material": "matte",
  "shape": "soft",
  "type": "geometric",
  "motion": "calm",
  "ornament": "none",
  "effects": "sheen",
  "sound": "soft",
  "icons": "ink",
  "density": "comfortable",
  "pack": {
    "images": images,
    "theme": {
      "colors": {
        "fg": "#2d3440", "fg-muted": "#525a68", "fg-subtle": "#7d8493",
        "part-fg": "#2d3440", "part-on": "#f3ede2", "part-gold": "#b97a12",
        "accent": "#2d3440", "accent-fg": "#f3ede2", "track": "#c8bba3"
      },
      "duration": {"ready-pulse": 1900, "flash": 220, "damage": 850, "damage-stagger": 50, "trail-hold": 280, "trail": 500, "hit": 420, "arc": 680},
      "scale": {"ready-pulse": 1.02, "damage-pop": 1.28, "combo-pop": 1.15, "press": 0.97},
      "opacity": {"sweep": 0.52, "flash": 0.62},
      "stroke": {"arc": 8}
    },
    "variants": {
      "Ability.box": "size-15 compact:size-10", "Ability.box.ultimate": "size-22 compact:size-13",
      "Title.wordmark": "font-display font-light text-display-3xl leading-none uppercase tracking-display-3xl text-part-fg",
      "Panel.frame": "nine-panel text-part-fg",
      "Panel.body": "gap-4 px-8 pt-6 pb-8",
      "Panel.title": "font-display font-light text-2xl uppercase tracking-kicker text-part-fg",
      "Plate.frame": "nine-plate text-part-fg",
      "Slot.frame": "nine-slot text-part-fg",
      "Chip.frame": "nine-plate text-part-fg",
      "Dialog.frame": "flex-col gap-4 w-115 max-w-full max-h-[85vh] overflow-y-auto m-auto px-8 pt-6 pb-8 nine-panel text-part-fg",
      "Dialog.title": "font-display font-light text-2xl leading-tight uppercase tracking-kicker text-part-fg",
      "Dialog.body": "text-base leading-normal text-fg-muted",
      "Button.plate": "relative nine-button text-part-fg font-medium uppercase tracking-label px-6 pb-0.5 active:nine-button-down aria-pressed:nine-button-down selected:nine-button-down disabled:opacity-50",
      "Button.plate.primary": "nine-button-primary text-part-on active:nine-button-primary-down",
      "Button.plate.danger": "nine-button-danger text-part-on active:nine-button-danger-down",
      "Button.plate.ghost": "nine-none bg-transparent text-part-fg hover:text-part-gold",
      "Button.plate.ghost-muted": "nine-none bg-transparent text-fg-muted hover:text-part-fg",
      "Button.plate.ghost-faint": "nine-none bg-transparent text-fg-subtle hover:text-fg-muted",
      "Button.plate.outline": "nine-entry text-part-fg",
      "Button.plate.menu": "relative w-full justify-start py-1.5 pl-9 pr-4 nine-none bg-transparent text-fg-muted data-current:text-part-fg disabled:text-fg-subtle",
      "Button.plate.menu-danger": "relative w-full justify-start py-1.5 pl-9 pr-4 nine-none bg-transparent text-fg-muted data-current:text-part-fg disabled:text-fg-subtle",
      "MenuList.indicator": "absolute inset-0 nine-entry-on pointer-events-none transition-layout duration-300 ease-out glide-menu",
      "MenuList.indicator.danger": "",
      "MenuList.cursor": "absolute left-3 top-1/2 size-3 -translate-y-1/2 nine-cursor",
      "MenuList.label": "text-start font-display font-light text-2xl leading-tight uppercase tracking-label",
      "MenuList.label.md": "text-xl",
      "MenuList.line": "text-start text-xs uppercase tracking-label text-fg-muted whitespace-nowrap overflow-hidden",
      "MenuList.chevron": "hidden",
      "Tab.list": "flex-row items-end gap-6 border-b border-track",
      "Tab.item": "flex-row items-center justify-center gap-2 min-h-11 px-1 text-sm font-medium uppercase tracking-label text-fg-muted hover:text-part-fg selected:text-part-fg disabled:opacity-50",
      "Tab.mark": "inset-x-0 -bottom-px h-0.5 bg-part-gold",
      "Slider.track": "relative grow h-1.5 nine-track disabled:opacity-50",
      "Slider.range": "absolute left-0 top-0 bottom-0 nine-range",
      "Slider.thumb": "absolute top-1/2 size-5 -translate-x-1/2 -translate-y-1/2 nine-thumb",
      "Switch.track": "relative h-6 w-12 shrink-0 nine-slot checked:nine-button-primary disabled:opacity-50",
      "Switch.thumb": "absolute top-0.5 left-0.5 size-5 nine-thumb transition-transform duration-300 ease-out checked:translate-x-6",
      "Checkbox.indicator": "flex items-center justify-center size-5 shrink-0 nine-check checked:nine-check-on disabled:opacity-50",
      "Checkbox.mark": "hidden",
      "Bar.frame": "h-3 nine-bar overflow-hidden p-0.5",
      "Tooltip.frame": "max-w-64 px-3 py-1.5 nine-plate text-part-fg text-xs",
      "Toast.frame": "flex-row items-center gap-3 w-full max-w-96 min-h-12 px-5 py-3 nine-plate text-part-fg pointer-events-auto",
      "Toast.frame.danger": "",
      "Toast.frame.success": "",
      "Row.frame": "nine-slot text-part-fg",
      "Row.frame.selected": "nine-entry-on text-part-fg"
    }
  }
});
