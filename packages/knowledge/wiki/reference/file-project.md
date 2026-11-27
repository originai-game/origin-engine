<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-project"></a>
### `file:project`

Checked `project` input. Record roles and project-specific declarations are bound by `og check`.

```ts
/** One root manifest for small and large projects. Omitted simulation
settings remain the game's responsibility, not inferred cook choices. */
export type Input = ({
readonly "budgets"?: (((InputDefinitions["Budgets"])) | ((null)));
readonly "conform"?: (((InputDefinitions["Conform"])) | ((null)));
readonly "cook"?: (((InputDefinitions["ProjectCook"])) | ((null)));
readonly "create"?: (((InputDefinitions["Creation"])) | ((null)));
/** Direct dependencies of the implicit base source module. */
readonly "dependsOn"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Project-relative TypeScript game entry; bootstrap never executes it. */
readonly "entry": (string);
readonly "flow"?: (((InputDefinitions["ProjectFlow"])) | ((null)));
readonly "id": (string);
/** Explicit input globs for the base module, relative to its root. */
readonly "inputs"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "modules"?: (ReadonlyArray<(InputDefinitions["ModuleInput"])> | (number extends ((InputDefinitions["ModuleInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "packs"?: (ReadonlyArray<(InputDefinitions["DeliveryPack"])> | (number extends ((InputDefinitions["DeliveryPack"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Named median-time limits in milliseconds, not cook quality choices. */
readonly "perf"?: (Readonly<Record<string, (number)>>);
readonly "publish"?: (((InputDefinitions["Publish"])) | ((null)));
readonly "roots"?: (ReadonlyArray<(InputDefinitions["ContentRef"])> | (number extends ((InputDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "saveVersion"?: ((number) | (null));
readonly "scene"?: ((string) | (null));
readonly "scenes"?: (Readonly<Record<string, (InputDefinitions["ContentRef"])>>);
/** Pure record declaration files, never a running game entry. */
readonly "schemas"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Named command scripts used by UI and style capture tools. */
readonly "screens"?: (Readonly<Record<string, (string)>>);
readonly "seed"?: ((number) | (null));
/** Pure structured expression functions, checked before story compilation. */
readonly "storyFunctions"?: (Readonly<Record<string, (InputDefinitions["StoryFunction"])>>);
readonly "tick"?: ((number) | (null));
});
export interface InputDefinitions {
/** Explicit overrides of the existing content-report limits. Absence leaves
that limit's interpretation with the report, not the author evaluator. */
readonly "Budgets": ({
readonly "assetBytes"?: ((number) | (null));
readonly "bones"?: ((number) | (null));
readonly "bvhShare"?: ((number) | (null));
readonly "clips"?: ((number) | (null));
readonly "emitterParticles"?: ((number) | (null));
readonly "gpuBytes"?: ((number) | (null));
readonly "navPolygons"?: ((number) | (null));
readonly "sceneEntities"?: ((number) | (null));
readonly "soundSeconds"?: ((number) | (null));
readonly "textureSize"?: ((number) | (null));
readonly "totalBytes"?: ((number) | (null));
readonly "triangles"?: ((number) | (null));
readonly "vertices"?: ((number) | (null));
readonly "videoSeconds"?: ((number) | (null));
});
readonly "ColorEncoding": ("etc1s" | "uastc") & (string);
readonly "Conform": ({
readonly "off"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Controls": ("touch" | "keyboard-mouse" | "gamepad") & (string);
/** Creation metadata describes the chosen shell. No genre composition or
balancing is selected by bootstrap; open catalogue IDs remain strings. */
readonly "Creation": ({
readonly "form"?: (((InputDefinitions["Form"])) | ((null)));
readonly "genre"?: ((string) | (null));
readonly "input"?: (((InputDefinitions["Controls"])) | ((null)));
readonly "lang"?: ((ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "look"?: ((string) | (null));
readonly "music"?: ((string) | (null));
readonly "platform"?: (((InputDefinitions["Platform"])) | ((null)));
readonly "players"?: (((InputDefinitions["Players"])) | ((null)));
readonly "ui"?: ((string) | (null));
readonly "variant"?: ((string) | (null));
readonly "view"?: ((string) | (null));
});
/** An output delivery unit, not a module, stream group or world cell. */
readonly "DeliveryPack": ({
readonly "dependsOn"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "id": (string);
/** Namespaced content patterns. Overlap is an error, never first-wins. */
readonly "include"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "optional"?: (boolean);
readonly "roots"?: (ReadonlyArray<(InputDefinitions["ContentRef"])> | (number extends ((InputDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Expr": (((({
readonly "kind": ("literal") & (string);
readonly "value": (InputDefinitions["Value"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }) | ((({
readonly "kind": ("variable") & (string);
readonly "name": (string);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "id": (string);
readonly "kind": ("once") & (string);
})) & { readonly "args"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("not") & (string);
readonly "value": (InputDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }) | ((({
readonly "kind": ("negate") & (string);
readonly "value": (InputDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }) | ((({
readonly "kind": ("binary") & (string);
readonly "left": (InputDefinitions["Expr"]);
readonly "op": (InputDefinitions["Op"]);
readonly "right": (InputDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "quest"?: never; readonly "value"?: never; }) | ((({
readonly "args": (ReadonlyArray<(InputDefinitions["Expr"])> | (number extends ((InputDefinitions["Expr"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("call") & (string);
readonly "name": (string);
})) & { readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("seen") & (string);
readonly "node": (string);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("status") & (string);
readonly "quest": (string);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("stage") & (string);
readonly "quest": (string);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("count") & (string);
readonly "objective": (string);
readonly "quest": (string);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "op"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("member") & (string);
readonly "member": (string);
readonly "name": (string);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("enum-value") & (string);
readonly "name": (string);
readonly "value": (InputDefinitions["Value"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }));
readonly "Fit": ("fill" | "letterbox") & (string);
readonly "FlowScreen": ("engine" | "game") & (string);
readonly "Form": ("2d" | "2.5d" | "3d") & (string);
readonly "Fullscreen": ("off" | "gesture") & (string);
readonly "Gate": ("tap" | "none") & (string);
readonly "Isolation": ("required" | "preferred" | "off") & (string);
readonly "Loading": ("engine" | "game" | "page") & (string);
readonly "LoadingLabels": ({
readonly "fetch"?: ((string) | (null));
readonly "present"?: ((string) | (null));
readonly "prime"?: ((string) | (null));
readonly "transcode"?: ((string) | (null));
});
/** A namespace and source root, independent of delivery and streaming. */
readonly "ModuleInput": ({
readonly "dependsOn"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "id": (string);
readonly "inputs"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Canonical project-relative directory. Moving it preserves IDs. */
readonly "root": (string);
});
readonly "Op": ("add" | "sub" | "mul" | "div" | "rem" | "eq" | "ne" | "lt" | "le" | "gt" | "ge" | "and" | "or" | "xor") & (string);
readonly "Orientation": ("any" | "landscape" | "portrait") & (string);
readonly "Pipelines": ("auto" | "sync" | "async" | "prewarm") & (string);
readonly "Platform": ("phone-portrait" | "phone-landscape" | "desktop" | "multi") & (string);
readonly "Players": ("single" | "local-multiplayer" | "online-lockstep") & (string);
readonly "Probe": ("player" | "all") & (string);
readonly "ProjectCook": ({
readonly "clusters"?: ((boolean) | (null));
readonly "colorTextures"?: (((InputDefinitions["ColorEncoding"])) | ((null)));
/** Explicit prebuilt bundle inputs. Names must remain globally unique. */
readonly "uses"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "ProjectFlow": ({
readonly "screen"?: (((InputDefinitions["FlowScreen"])) | ((null)));
readonly "source": (InputDefinitions["ContentRef"]);
});
readonly "Publish": ({
readonly "aspect"?: ((string) | (null));
readonly "aspectMax"?: ((string) | (null));
readonly "aspectMin"?: ((string) | (null));
readonly "assets"?: ((string) | (null));
readonly "colorTextures"?: (((InputDefinitions["ColorEncoding"])) | ((null)));
readonly "content"?: ((string) | (null));
readonly "dprCap"?: ((number) | (null));
readonly "fit"?: (((InputDefinitions["Fit"])) | ((null)));
readonly "fullscreen"?: (((InputDefinitions["Fullscreen"])) | ((null)));
readonly "gate"?: (((InputDefinitions["Gate"])) | ((null)));
readonly "icon"?: ((string) | (null));
readonly "isolation"?: (((InputDefinitions["Isolation"])) | ((null)));
readonly "keep"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "loading"?: (((InputDefinitions["Loading"])) | ((null)));
readonly "loadingLabels"?: (((InputDefinitions["LoadingLabels"])) | ((null)));
readonly "madeWith"?: ((boolean) | (null));
readonly "orientation"?: (((InputDefinitions["Orientation"])) | ((null)));
readonly "out"?: ((string) | (null));
readonly "pipelines"?: (((InputDefinitions["Pipelines"])) | ((null)));
readonly "probe"?: (((InputDefinitions["Probe"])) | ((null)));
readonly "shortName"?: ((string) | (null));
readonly "telemetry"?: (((InputDefinitions["Telemetry"])) | ((null)));
});
readonly "StoryFunction": ({
readonly "body": (InputDefinitions["Expr"]);
/** Declaration order is the function's argument order. */
readonly "parameters"?: (ReadonlyArray<(InputDefinitions["StoryParameter"])> | (number extends ((InputDefinitions["StoryParameter"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "returns": (InputDefinitions["StoryType"]);
});
readonly "StoryParameter": ({
readonly "kind": (InputDefinitions["StoryType"]);
readonly "name": (string);
});
readonly "StoryType": ("number" | "string" | "bool") & (string);
readonly "Telemetry": ("on" | "off") & (string);
readonly "Value": (((number)) | ((boolean)) | ((string)));
}

```
