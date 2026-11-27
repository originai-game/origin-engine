<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-environment"></a>
### `file:environment`

Checked `file["environment"](input)` declaration from `@origingame/engine/author`.

```ts
/** Sky source and explicit overrides. Omission is retained, not replaced by
normalized defaults: e.g. LDR extracts the sun while HDR keeps it. Source
compatibility, prefilter/sample conflicts, hemisphere/ground relationships,
image dimensions and power-of-two size remain environment domain checks. */
export type Input = ({
/** Inverse tone-mapping strength; omitted: 0.25, zero disables it. */
readonly "expand"?: ((number) | (null));
/** Linear RGB ground tint, requires an upper-hemisphere equirectangular image. */
readonly "ground"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "hemisphere"?: (((InputDefinitions["Hemisphere"])) | ((null)));
/** Radiance multiplier; omitted: 1. */
readonly "intensity"?: ((number) | (null));
/** Use the KTX2 cube's existing mip chain; omitted: false. */
readonly "prefiltered"?: ((boolean) | (null));
/** Rotation about the vertical axis; omitted: 0 degrees. */
readonly "rotation"?: ((number) | (null));
/** GGX samples per texel; omitted: 256. Forbidden for a prefiltered cube. */
readonly "samples"?: ((number) | (null));
/** Face side in texels. Omitted: 128, or the prefiltered cube's own size. */
readonly "size"?: ((number) | (null));
/** Explicit canonical paths under the owning module's assets/ or sources/.
No sibling lookup or project asset-root fallback. */
readonly "source": (InputDefinitions["EnvironmentSource"]);
/** Omitted: extract for LDR images, keep for HDR and KTX2. */
readonly "sun"?: (((InputDefinitions["Sun"])) | ((null)));
/** Estimated LDR sun multiplier; omitted: 1. */
readonly "sunBoost"?: ((number) | (null));
});
export interface InputDefinitions {
/** What an environment bakes from. */
readonly "EnvironmentSource": (((({
readonly "equirect": (string);
})) & { readonly "faces"?: never; readonly "ktx2"?: never; }) | ((({
readonly "ktx2": (string);
})) & { readonly "equirect"?: never; readonly "faces"?: never; }) | ((({
readonly "faces": (ReadonlyArray<(string)> & { readonly length: 6 });
})) & { readonly "equirect"?: never; readonly "ktx2"?: never; }));
readonly "Hemisphere": ("upper") & (string);
readonly "Sun": ("extract" | "keep") & (string);
}
```
