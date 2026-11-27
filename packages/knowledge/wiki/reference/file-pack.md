<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-pack"></a>
### `file:pack`

Checked `file["pack"](input)` declaration from `@origingame/engine/author`.

```ts
/** One style layer. Class resolution, CSS parsing, scheme-pair semantics,
slot names and referenced content are checked by the pack domain validator. */
export type Input = ({
/** Ignored author annotation, deliberately arbitrary JSON data. */
readonly "about"?: unknown;
readonly "contrast"?: (((InputDefinitions["Theme"])) | ((null)));
/** Ordered font faces; family, weight and style belong to each face. */
readonly "fonts"?: (ReadonlyArray<(InputDefinitions["ContentRef"])> | (number extends ((InputDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "haptics"?: (Readonly<Record<string, (InputDefinitions["HapticBinding"])>>);
readonly "iconSet"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "icons"?: (Readonly<Record<string, (InputDefinitions["ContentRef"])>>);
/** Named nine-slice art; names are kebab-case. */
readonly "images"?: (Readonly<Record<string, (InputDefinitions["Adaptive11"])>>);
readonly "scheme"?: (((InputDefinitions["Scheme"])) | ((null)));
readonly "seeds"?: (((InputDefinitions["Seeds"])) | ((null)));
/** An event maps to a sound; a Component.slot[.variant] maps to events. */
readonly "sounds"?: (Readonly<Record<string, (InputDefinitions["SoundBinding"])>>);
readonly "theme"?: (((InputDefinitions["Theme"])) | ((null)));
/** Component.slot[.variant] -> class string; empty declares the slot. */
readonly "variants"?: (Readonly<Record<string, (string)>>);
});
export interface InputDefinitions {
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive": (((InputDefinitions["Schemes"])) | ((string)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive10": (((InputDefinitions["Schemes10"])) | ((ReadonlyArray<(number)> & { readonly length: 4 })));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive11": ((((InputDefinitions["Schemes11"])) & { readonly "fill"?: never; readonly "repeat"?: never; readonly "slice"?: never; readonly "src"?: never; readonly "tint"?: never; readonly "width"?: never; }) | (((InputDefinitions["NineSlice"])) & { readonly "dark"?: never; readonly "light"?: never; }));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive12": ((((InputDefinitions["Schemes12"])) & { readonly "$ogRef"?: never; }) | (((InputDefinitions["ContentRef"])) & { readonly "dark"?: never; readonly "light"?: never; }));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive13": (((InputDefinitions["Schemes13"])) | ((InputDefinitions["Insets"])));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive14": (((InputDefinitions["Schemes14"])) | ((boolean)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive15": (((InputDefinitions["Schemes15"])) | ((InputDefinitions["Repeat"])));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive2": (((InputDefinitions["Schemes2"])) | ((Readonly<Record<string, (InputDefinitions["Adaptive3"])>>)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive3": (((InputDefinitions["Schemes3"])) | ((InputDefinitions["Palette"])));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive4": (((InputDefinitions["Schemes4"])) | ((number)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive5": (((InputDefinitions["Schemes5"])) | ((Readonly<Record<string, (InputDefinitions["Adaptive4"])>>)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive6": (((InputDefinitions["Schemes6"])) | ((Readonly<Record<string, (InputDefinitions["Adaptive7"])>>)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive7": (((InputDefinitions["Schemes7"])) | ((InputDefinitions["FontSize"])));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive8": (((InputDefinitions["Schemes8"])) | ((Readonly<Record<string, (InputDefinitions["Adaptive"])>>)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive9": (((InputDefinitions["Schemes9"])) | ((Readonly<Record<string, (InputDefinitions["Adaptive10"])>>)));
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
/** The same interaction event vocabulary for global and slot-local values. */
readonly "Events": ({
readonly "back"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "cancel"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "close"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "confirm"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "countdown"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "error"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "hover"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "levelup"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "notify"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "open"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "press"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "purchase"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "reward"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "select"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "step"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "swipe"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "tab"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "tick"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "toggle"?: (((InputDefinitions["Sound"])) | ((null)));
readonly "unlock"?: (((InputDefinitions["Sound"])) | ((null)));
});
/** The same interaction event vocabulary for global and slot-local values. */
readonly "Events2": ({
readonly "back"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "cancel"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "close"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "confirm"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "countdown"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "error"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "hover"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "levelup"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "notify"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "open"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "press"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "purchase"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "reward"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "select"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "step"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "swipe"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "tab"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "tick"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "toggle"?: (((InputDefinitions["Haptic"])) | ((null)));
readonly "unlock"?: (((InputDefinitions["Haptic"])) | ((null)));
});
readonly "FontSize": (((number)) | ((ReadonlyArray<(number)> & { readonly length: 2 })));
readonly "Haptic": ("tick" | "tap" | "thud" | "double" | "buzz" | "success" | "warning" | "error" | "none") & (string);
readonly "HapticBinding": (((InputDefinitions["Haptic"])) | ((InputDefinitions["Events2"])));
readonly "Insets": (((InputDefinitions["Nonnegative"])) | ((ReadonlyArray<(InputDefinitions["Nonnegative"])> & { readonly length: 4 })));
readonly "NineSlice": ({
/** Unset paints the centre. */
readonly "fill"?: (((InputDefinitions["Adaptive14"])) | ((null)));
readonly "repeat"?: (((InputDefinitions["Adaptive15"])) | ((null)));
readonly "slice": (InputDefinitions["Adaptive13"]);
readonly "src": (InputDefinitions["Adaptive12"]);
readonly "tint"?: (((InputDefinitions["Adaptive14"])) | ((null)));
/** Unset draws at the slice's own widths, in points. */
readonly "width"?: (((InputDefinitions["Adaptive13"])) | ((null)));
});
readonly "Nonnegative": (number);
readonly "Palette": (((string)) | ((Readonly<Record<string, (InputDefinitions["Adaptive"])>>)));
readonly "Range": (((number)) | ((ReadonlyArray<(number)> & { readonly length: 2 })));
readonly "Repeat": ("stretch" | "tile") & (string);
readonly "Scheme": ("light" | "dark") & (string);
readonly "Schemes": ({
readonly "dark": (string);
readonly "light": (string);
});
readonly "Schemes10": ({
readonly "dark": (ReadonlyArray<(number)> & { readonly length: 4 });
readonly "light": (ReadonlyArray<(number)> & { readonly length: 4 });
});
readonly "Schemes11": ({
readonly "dark": (InputDefinitions["NineSlice"]);
readonly "light": (InputDefinitions["NineSlice"]);
});
readonly "Schemes12": ({
readonly "dark": (InputDefinitions["ContentRef"]);
readonly "light": (InputDefinitions["ContentRef"]);
});
readonly "Schemes13": ({
readonly "dark": (InputDefinitions["Insets"]);
readonly "light": (InputDefinitions["Insets"]);
});
readonly "Schemes14": ({
readonly "dark": (boolean);
readonly "light": (boolean);
});
readonly "Schemes15": ({
readonly "dark": (InputDefinitions["Repeat"]);
readonly "light": (InputDefinitions["Repeat"]);
});
readonly "Schemes2": ({
readonly "dark": (Readonly<Record<string, (InputDefinitions["Adaptive3"])>>);
readonly "light": (Readonly<Record<string, (InputDefinitions["Adaptive3"])>>);
});
readonly "Schemes3": ({
readonly "dark": (InputDefinitions["Palette"]);
readonly "light": (InputDefinitions["Palette"]);
});
readonly "Schemes4": ({
readonly "dark": (number);
readonly "light": (number);
});
readonly "Schemes5": ({
readonly "dark": (Readonly<Record<string, (InputDefinitions["Adaptive4"])>>);
readonly "light": (Readonly<Record<string, (InputDefinitions["Adaptive4"])>>);
});
readonly "Schemes6": ({
readonly "dark": (Readonly<Record<string, (InputDefinitions["Adaptive7"])>>);
readonly "light": (Readonly<Record<string, (InputDefinitions["Adaptive7"])>>);
});
readonly "Schemes7": ({
readonly "dark": (InputDefinitions["FontSize"]);
readonly "light": (InputDefinitions["FontSize"]);
});
readonly "Schemes8": ({
readonly "dark": (Readonly<Record<string, (InputDefinitions["Adaptive"])>>);
readonly "light": (Readonly<Record<string, (InputDefinitions["Adaptive"])>>);
});
readonly "Schemes9": ({
readonly "dark": (Readonly<Record<string, (InputDefinitions["Adaptive10"])>>);
readonly "light": (Readonly<Record<string, (InputDefinitions["Adaptive10"])>>);
});
readonly "Seeds": ({
readonly "accent"?: (((InputDefinitions["Adaptive"])) | ((null)));
readonly "danger"?: (((InputDefinitions["Adaptive"])) | ((null)));
readonly "hud"?: (((InputDefinitions["Adaptive"])) | ((null)));
readonly "info"?: (((InputDefinitions["Adaptive"])) | ((null)));
readonly "neutral"?: (((InputDefinitions["Adaptive"])) | ((null)));
readonly "primary"?: (((InputDefinitions["Adaptive"])) | ((null)));
readonly "success"?: (((InputDefinitions["Adaptive"])) | ((null)));
readonly "team"?: (((InputDefinitions["Adaptive"])) | ((null)));
readonly "warn"?: (((InputDefinitions["Adaptive"])) | ((null)));
});
readonly "Sound": ({
/** Empty clips silence an underlying layer only when volume is zero. */
readonly "clips"?: (ReadonlyArray<(InputDefinitions["ContentRef"])> | (number extends ((InputDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Playback rate or [low, high]; unset is 1. */
readonly "pitch"?: (((InputDefinitions["Range"])) | ((null)));
/** Gain or [low, high]; unset is 1. */
readonly "volume"?: (((InputDefinitions["Range"])) | ((null)));
});
readonly "SoundBinding": ((((InputDefinitions["Sound"])) & { readonly "back"?: never; readonly "cancel"?: never; readonly "close"?: never; readonly "confirm"?: never; readonly "countdown"?: never; readonly "error"?: never; readonly "hover"?: never; readonly "levelup"?: never; readonly "notify"?: never; readonly "open"?: never; readonly "press"?: never; readonly "purchase"?: never; readonly "reward"?: never; readonly "select"?: never; readonly "step"?: never; readonly "swipe"?: never; readonly "tab"?: never; readonly "tick"?: never; readonly "toggle"?: never; readonly "unlock"?: never; }) | (((InputDefinitions["Events"])) & { readonly "clips"?: never; readonly "pitch"?: never; readonly "volume"?: never; }));
/** NSH Tailwind theme sections, as consumed by Theme::from_json after scheme
splitting. Numerical scales are numbers, not arbitrary CSS/JSON values. */
readonly "Theme": ({
readonly "aspect"?: (((InputDefinitions["Adaptive5"])) | ((null)));
readonly "blur"?: (((InputDefinitions["Adaptive5"])) | ((null)));
readonly "breakpoints"?: (((InputDefinitions["Adaptive5"])) | ((null)));
readonly "colors"?: (((InputDefinitions["Adaptive2"])) | ((null)));
readonly "container"?: (((InputDefinitions["Adaptive5"])) | ((null)));
readonly "dropShadow"?: (((InputDefinitions["Adaptive8"])) | ((null)));
readonly "duration"?: (((InputDefinitions["Adaptive5"])) | ((null)));
/** Cubic Bezier control points [x1, y1, x2, y2]. */
readonly "ease"?: (((InputDefinitions["Adaptive9"])) | ((null)));
readonly "fontFamily"?: (((InputDefinitions["Adaptive8"])) | ((null)));
readonly "fontSize"?: (((InputDefinitions["Adaptive6"])) | ((null)));
readonly "fontWeight"?: (((InputDefinitions["Adaptive5"])) | ((null)));
readonly "insetShadow"?: (((InputDefinitions["Adaptive8"])) | ((null)));
readonly "leading"?: (((InputDefinitions["Adaptive5"])) | ((null)));
readonly "opacity"?: (((InputDefinitions["Adaptive5"])) | ((null)));
readonly "perspective"?: (((InputDefinitions["Adaptive5"])) | ((null)));
readonly "radius"?: (((InputDefinitions["Adaptive5"])) | ((null)));
/** Root em in points. */
readonly "rem"?: (((InputDefinitions["Adaptive4"])) | ((null)));
readonly "scale"?: (((InputDefinitions["Adaptive5"])) | ((null)));
/** CSS box-shadow values. */
readonly "shadow"?: (((InputDefinitions["Adaptive8"])) | ((null)));
/** Base spacing unit in points. */
readonly "spacing"?: (((InputDefinitions["Adaptive4"])) | ((null)));
readonly "stroke"?: (((InputDefinitions["Adaptive5"])) | ((null)));
/** CSS text-shadow values. */
readonly "textShadow"?: (((InputDefinitions["Adaptive8"])) | ((null)));
readonly "tracking"?: (((InputDefinitions["Adaptive5"])) | ((null)));
});
}
```
