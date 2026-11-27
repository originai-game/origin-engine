// Generated from the cook's Rust input schemas by `og api gen`. Do not edit.
/** One root manifest for small and large projects. Omitted simulation
settings remain the game's responsibility, not inferred cook choices. */
export type ProjectInput = ({
readonly "budgets"?: (((ProjectInputDefinitions["Budgets"])) | ((null)));
readonly "conform"?: (((ProjectInputDefinitions["Conform"])) | ((null)));
readonly "cook"?: (((ProjectInputDefinitions["ProjectCook"])) | ((null)));
readonly "create"?: (((ProjectInputDefinitions["Creation"])) | ((null)));
/** Direct dependencies of the implicit base source module. */
readonly "dependsOn"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Project-relative TypeScript game entry; bootstrap never executes it. */
readonly "entry": (string);
readonly "flow"?: (((ProjectInputDefinitions["ProjectFlow"])) | ((null)));
readonly "id": (string);
/** Explicit input globs for the base module, relative to its root. */
readonly "inputs"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "modules"?: (ReadonlyArray<(ProjectInputDefinitions["ModuleInput"])> | (number extends ((ProjectInputDefinitions["ModuleInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "packs"?: (ReadonlyArray<(ProjectInputDefinitions["DeliveryPack"])> | (number extends ((ProjectInputDefinitions["DeliveryPack"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Named median-time limits in milliseconds, not cook quality choices. */
readonly "perf"?: (Readonly<Record<string, (number)>>);
readonly "publish"?: (((ProjectInputDefinitions["Publish"])) | ((null)));
readonly "roots"?: (ReadonlyArray<(ProjectInputDefinitions["ContentRef"])> | (number extends ((ProjectInputDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "saveVersion"?: ((number) | (null));
readonly "scene"?: ((string) | (null));
readonly "scenes"?: (Readonly<Record<string, (ProjectInputDefinitions["ContentRef"])>>);
/** Pure record declaration files, never a running game entry. */
readonly "schemas"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Named command scripts used by UI and style capture tools. */
readonly "screens"?: (Readonly<Record<string, (string)>>);
readonly "seed"?: ((number) | (null));
/** Pure structured expression functions, checked before story compilation. */
readonly "storyFunctions"?: (Readonly<Record<string, (ProjectInputDefinitions["StoryFunction"])>>);
readonly "tick"?: ((number) | (null));
});
export interface ProjectInputDefinitions {
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
readonly "form"?: (((ProjectInputDefinitions["Form"])) | ((null)));
readonly "genre"?: ((string) | (null));
readonly "input"?: (((ProjectInputDefinitions["Controls"])) | ((null)));
readonly "lang"?: ((ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "look"?: ((string) | (null));
readonly "music"?: ((string) | (null));
readonly "platform"?: (((ProjectInputDefinitions["Platform"])) | ((null)));
readonly "players"?: (((ProjectInputDefinitions["Players"])) | ((null)));
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
readonly "roots"?: (ReadonlyArray<(ProjectInputDefinitions["ContentRef"])> | (number extends ((ProjectInputDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Expr": (((({
readonly "kind": ("literal") & (string);
readonly "value": (ProjectInputDefinitions["Value"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }) | ((({
readonly "kind": ("variable") & (string);
readonly "name": (string);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "id": (string);
readonly "kind": ("once") & (string);
})) & { readonly "args"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("not") & (string);
readonly "value": (ProjectInputDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }) | ((({
readonly "kind": ("negate") & (string);
readonly "value": (ProjectInputDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }) | ((({
readonly "kind": ("binary") & (string);
readonly "left": (ProjectInputDefinitions["Expr"]);
readonly "op": (ProjectInputDefinitions["Op"]);
readonly "right": (ProjectInputDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "quest"?: never; readonly "value"?: never; }) | ((({
readonly "args": (ReadonlyArray<(ProjectInputDefinitions["Expr"])> | (number extends ((ProjectInputDefinitions["Expr"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
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
readonly "value": (ProjectInputDefinitions["Value"]);
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
readonly "colorTextures"?: (((ProjectInputDefinitions["ColorEncoding"])) | ((null)));
/** Explicit prebuilt bundle inputs. Names must remain globally unique. */
readonly "uses"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "ProjectFlow": ({
readonly "screen"?: (((ProjectInputDefinitions["FlowScreen"])) | ((null)));
readonly "source": (ProjectInputDefinitions["ContentRef"]);
});
readonly "Publish": ({
readonly "aspect"?: ((string) | (null));
readonly "aspectMax"?: ((string) | (null));
readonly "aspectMin"?: ((string) | (null));
readonly "assets"?: ((string) | (null));
readonly "colorTextures"?: (((ProjectInputDefinitions["ColorEncoding"])) | ((null)));
readonly "content"?: ((string) | (null));
readonly "dprCap"?: ((number) | (null));
readonly "fit"?: (((ProjectInputDefinitions["Fit"])) | ((null)));
readonly "fullscreen"?: (((ProjectInputDefinitions["Fullscreen"])) | ((null)));
readonly "gate"?: (((ProjectInputDefinitions["Gate"])) | ((null)));
readonly "icon"?: ((string) | (null));
readonly "isolation"?: (((ProjectInputDefinitions["Isolation"])) | ((null)));
readonly "keep"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "loading"?: (((ProjectInputDefinitions["Loading"])) | ((null)));
readonly "loadingLabels"?: (((ProjectInputDefinitions["LoadingLabels"])) | ((null)));
readonly "madeWith"?: ((boolean) | (null));
readonly "orientation"?: (((ProjectInputDefinitions["Orientation"])) | ((null)));
readonly "out"?: ((string) | (null));
readonly "pipelines"?: (((ProjectInputDefinitions["Pipelines"])) | ((null)));
readonly "probe"?: (((ProjectInputDefinitions["Probe"])) | ((null)));
readonly "shortName"?: ((string) | (null));
readonly "telemetry"?: (((ProjectInputDefinitions["Telemetry"])) | ((null)));
});
readonly "StoryFunction": ({
readonly "body": (ProjectInputDefinitions["Expr"]);
/** Declaration order is the function's argument order. */
readonly "parameters"?: (ReadonlyArray<(ProjectInputDefinitions["StoryParameter"])> | (number extends ((ProjectInputDefinitions["StoryParameter"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "returns": (ProjectInputDefinitions["StoryType"]);
});
readonly "StoryParameter": ({
readonly "kind": (ProjectInputDefinitions["StoryType"]);
readonly "name": (string);
});
readonly "StoryType": ("number" | "string" | "bool") & (string);
readonly "Telemetry": ("on" | "off") & (string);
readonly "Value": (((number)) | ((boolean)) | ((string)));
}
/** A namespace and source root, independent of delivery and streaming. */
export type ModuleInput = ({
readonly "dependsOn"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "id": (string);
readonly "inputs"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Canonical project-relative directory. Moving it preserves IDs. */
readonly "root": (string);
});
export type Input_animgraph = ({
readonly "bindings"?: (ReadonlyArray<(Input_animgraphDefinitions["Binding"])> | (number extends ((Input_animgraphDefinitions["Binding"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "layers": (ReadonlyArray<(Input_animgraphDefinitions["LayerInput"])> | (number extends ((Input_animgraphDefinitions["LayerInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "motions": (Readonly<Record<string, (Input_animgraphDefinitions["MotionInput"])>>);
readonly "params"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "sprite"?: (boolean);
});
export interface Input_animgraphDefinitions {
readonly "Binding": ({
readonly "clip": (string);
readonly "name": (string);
readonly "source": (Input_animgraphDefinitions["ContentRef"]);
});
readonly "ConditionInput": ({
readonly "op": (Input_animgraphDefinitions["Op"]);
readonly "param": (string);
readonly "value": (number);
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
readonly "Interrupt": ("any" | "none" | "source" | "next") & (string);
readonly "LayerInput": ({
readonly "entry": (string);
readonly "mask"?: (Readonly<Record<string, (number)>>);
readonly "mode"?: (((Input_animgraphDefinitions["LayerMode"])) | ((null)));
readonly "name": (string);
readonly "states": (ReadonlyArray<(Input_animgraphDefinitions["StateInput"])> | (number extends ((Input_animgraphDefinitions["StateInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "transitions"?: (ReadonlyArray<(Input_animgraphDefinitions["TransitionInput"])> | (number extends ((Input_animgraphDefinitions["TransitionInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "weight"?: ((number) | (null));
readonly "weightParam"?: ((string) | (null));
});
readonly "LayerMode": ("override" | "additive") & (string);
readonly "MotionInput": (((({
readonly "clip": (string);
readonly "kind": ("clip") & (string);
readonly "looping"?: ((boolean) | (null));
readonly "mirror"?: ((boolean) | (null));
readonly "speed"?: ((number) | (null));
})) & { readonly "children"?: never; readonly "param"?: never; readonly "x"?: never; readonly "y"?: never; }) | ((({
readonly "children": (ReadonlyArray<(Input_animgraphDefinitions["ThresholdInput"])> | (number extends ((Input_animgraphDefinitions["ThresholdInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("blend1d") & (string);
readonly "param": (string);
})) & { readonly "clip"?: never; readonly "looping"?: never; readonly "mirror"?: never; readonly "speed"?: never; readonly "x"?: never; readonly "y"?: never; }) | ((({
readonly "children": (ReadonlyArray<(Input_animgraphDefinitions["PointInput"])> | (number extends ((Input_animgraphDefinitions["PointInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("blend2d") & (string);
readonly "x": (string);
readonly "y": (string);
})) & { readonly "clip"?: never; readonly "looping"?: never; readonly "mirror"?: never; readonly "param"?: never; readonly "speed"?: never; }));
readonly "Op": ("gt" | "ge" | "lt" | "le" | "eq" | "ne") & (string);
readonly "PointInput": ({
readonly "motion": (string);
readonly "point": (ReadonlyArray<(number)> & { readonly length: 2 });
});
readonly "StateInput": ({
readonly "motion": (string);
readonly "name": (string);
readonly "speed"?: ((number) | (null));
readonly "sync"?: ((string) | (null));
});
readonly "ThresholdInput": ({
readonly "motion": (string);
readonly "threshold": (number);
});
readonly "TransitionInput": ({
readonly "conditions"?: (ReadonlyArray<(Input_animgraphDefinitions["ConditionInput"])> | (number extends ((Input_animgraphDefinitions["ConditionInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "exit"?: ((number) | (null));
readonly "fade"?: ((number) | (null));
readonly "from"?: ((string) | (null));
readonly "interrupt"?: (((Input_animgraphDefinitions["Interrupt"])) | ((null)));
readonly "to": (string);
});
}
export type Input_behaviour = ({
readonly "keys"?: (Readonly<Record<string, (Input_behaviourDefinitions["Value"])>>);
readonly "name": (string);
readonly "root": (Input_behaviourDefinitions["Root"]);
readonly "routes"?: (Readonly<Record<string, (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))>>);
});
export interface Input_behaviourDefinitions {
readonly "Action": (((({
readonly "key"?: ((string) | (null));
readonly "kind": ("move-to") & (string);
readonly "point"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "speed"?: (((Input_behaviourDefinitions["Speed"])) | ((null)));
})) & { readonly "args"?: never; readonly "cooldown"?: never; readonly "distance"?: never; readonly "loops"?: never; readonly "name"?: never; readonly "radius"?: never; readonly "reach"?: never; readonly "route"?: never; readonly "seconds"?: never; readonly "value"?: never; readonly "wait"?: never; }) | ((({
readonly "key": (string);
readonly "kind": ("chase") & (string);
readonly "reach": (number);
readonly "speed"?: (((Input_behaviourDefinitions["Speed"])) | ((null)));
})) & { readonly "args"?: never; readonly "cooldown"?: never; readonly "distance"?: never; readonly "loops"?: never; readonly "name"?: never; readonly "point"?: never; readonly "radius"?: never; readonly "route"?: never; readonly "seconds"?: never; readonly "value"?: never; readonly "wait"?: never; }) | ((({
readonly "distance": (number);
readonly "key": (string);
readonly "kind": ("flee") & (string);
readonly "speed"?: (((Input_behaviourDefinitions["Speed"])) | ((null)));
})) & { readonly "args"?: never; readonly "cooldown"?: never; readonly "loops"?: never; readonly "name"?: never; readonly "point"?: never; readonly "radius"?: never; readonly "reach"?: never; readonly "route"?: never; readonly "seconds"?: never; readonly "value"?: never; readonly "wait"?: never; }) | ((({
readonly "kind": ("patrol") & (string);
readonly "loops": (boolean);
readonly "route": (string);
readonly "speed"?: (((Input_behaviourDefinitions["Speed"])) | ((null)));
readonly "wait": (number);
})) & { readonly "args"?: never; readonly "cooldown"?: never; readonly "distance"?: never; readonly "key"?: never; readonly "name"?: never; readonly "point"?: never; readonly "radius"?: never; readonly "reach"?: never; readonly "seconds"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("wander") & (string);
readonly "radius": (number);
readonly "speed"?: (((Input_behaviourDefinitions["Speed"])) | ((null)));
readonly "wait": (number);
})) & { readonly "args"?: never; readonly "cooldown"?: never; readonly "distance"?: never; readonly "key"?: never; readonly "loops"?: never; readonly "name"?: never; readonly "point"?: never; readonly "reach"?: never; readonly "route"?: never; readonly "seconds"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("wait") & (string);
readonly "seconds": (number);
})) & { readonly "args"?: never; readonly "cooldown"?: never; readonly "distance"?: never; readonly "key"?: never; readonly "loops"?: never; readonly "name"?: never; readonly "point"?: never; readonly "radius"?: never; readonly "reach"?: never; readonly "route"?: never; readonly "speed"?: never; readonly "value"?: never; readonly "wait"?: never; }) | ((({
readonly "kind": ("stop") & (string);
})) & { readonly "args"?: never; readonly "cooldown"?: never; readonly "distance"?: never; readonly "key"?: never; readonly "loops"?: never; readonly "name"?: never; readonly "point"?: never; readonly "radius"?: never; readonly "reach"?: never; readonly "route"?: never; readonly "seconds"?: never; readonly "speed"?: never; readonly "value"?: never; readonly "wait"?: never; }) | ((({
readonly "key": (string);
readonly "kind": ("set") & (string);
readonly "value": (Input_behaviourDefinitions["Value"]);
})) & { readonly "args"?: never; readonly "cooldown"?: never; readonly "distance"?: never; readonly "loops"?: never; readonly "name"?: never; readonly "point"?: never; readonly "radius"?: never; readonly "reach"?: never; readonly "route"?: never; readonly "seconds"?: never; readonly "speed"?: never; readonly "wait"?: never; }) | ((({
readonly "args": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "cooldown": (number);
readonly "kind": ("command") & (string);
readonly "name": (string);
})) & { readonly "distance"?: never; readonly "key"?: never; readonly "loops"?: never; readonly "point"?: never; readonly "radius"?: never; readonly "reach"?: never; readonly "route"?: never; readonly "seconds"?: never; readonly "speed"?: never; readonly "value"?: never; readonly "wait"?: never; }) | ((({
readonly "args": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("game") & (string);
readonly "name": (string);
})) & { readonly "cooldown"?: never; readonly "distance"?: never; readonly "key"?: never; readonly "loops"?: never; readonly "point"?: never; readonly "radius"?: never; readonly "reach"?: never; readonly "route"?: never; readonly "seconds"?: never; readonly "speed"?: never; readonly "value"?: never; readonly "wait"?: never; }));
readonly "Check": ({
/** Dynamic comparison operand, exclusive with value. */
readonly "against"?: ((string) | (null));
readonly "op": (Input_behaviourDefinitions["Op"]);
readonly "operand": (string);
readonly "value"?: (((Input_behaviourDefinitions["Value"])) | ((null)));
});
readonly "Consideration": (((({
readonly "kind": ("constant") & (string);
readonly "value": (number);
})) & { readonly "curve"?: never; readonly "hi"?: never; readonly "lo"?: never; readonly "operand"?: never; }) | ((({
readonly "curve": (Input_behaviourDefinitions["Curve"]);
readonly "hi": (number);
readonly "kind": ("read") & (string);
readonly "lo": (number);
readonly "operand": (string);
})) & { readonly "value"?: never; }));
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Curve": ("linear" | "inverse" | "quadratic" | "sqrt" | "smooth" | "step") & (string);
readonly "Node": (((({
readonly "children": (ReadonlyArray<(Input_behaviourDefinitions["Node"])> | (number extends ((Input_behaviourDefinitions["Node"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("selector") & (string);
readonly "name"?: ((string) | (null));
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "child"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "children": (ReadonlyArray<(Input_behaviourDefinitions["Node"])> | (number extends ((Input_behaviourDefinitions["Node"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("sequence") & (string);
readonly "name"?: ((string) | (null));
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "child"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "children": (ReadonlyArray<(Input_behaviourDefinitions["Node"])> | (number extends ((Input_behaviourDefinitions["Node"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("steps") & (string);
readonly "name"?: ((string) | (null));
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "child"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "all": (boolean);
readonly "children": (ReadonlyArray<(Input_behaviourDefinitions["Node"])> | (number extends ((Input_behaviourDefinitions["Node"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("parallel") & (string);
readonly "name"?: ((string) | (null));
})) & { readonly "action"?: never; readonly "check"?: never; readonly "child"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "child": (Input_behaviourDefinitions["Node"]);
readonly "kind": ("invert") & (string);
readonly "name"?: ((string) | (null));
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "children"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "child": (Input_behaviourDefinitions["Node"]);
readonly "kind": ("succeed") & (string);
readonly "name"?: ((string) | (null));
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "children"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "child": (Input_behaviourDefinitions["Node"]);
readonly "kind": ("repeat") & (string);
readonly "name"?: ((string) | (null));
readonly "times": (number);
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "children"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "source"?: never; }) | ((({
readonly "child": (Input_behaviourDefinitions["Node"]);
readonly "kind": ("cooldown") & (string);
readonly "name"?: ((string) | (null));
readonly "seconds": (number);
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "children"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "child": (Input_behaviourDefinitions["Node"]);
readonly "kind": ("timeout") & (string);
readonly "name"?: ((string) | (null));
readonly "seconds": (number);
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "children"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "check": (Input_behaviourDefinitions["Check"]);
readonly "kind": ("check") & (string);
readonly "name"?: ((string) | (null));
})) & { readonly "action"?: never; readonly "all"?: never; readonly "child"?: never; readonly "children"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "action": (Input_behaviourDefinitions["Action"]);
readonly "kind": ("action") & (string);
readonly "name"?: ((string) | (null));
})) & { readonly "all"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "kind": ("subtree") & (string);
readonly "name"?: ((string) | (null));
readonly "source": (Input_behaviourDefinitions["ContentRef"]);
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "times"?: never; }) | ((({
readonly "inertia": (number);
readonly "kind": ("utility") & (string);
readonly "name"?: ((string) | (null));
readonly "options": (ReadonlyArray<(Input_behaviourDefinitions["UtilityOption"])> | (number extends ((Input_behaviourDefinitions["UtilityOption"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "seconds"?: never; readonly "source"?: never; readonly "times"?: never; }));
readonly "Op": ("is" | "not" | "below" | "above" | "atMost" | "atLeast" | "truthy") & (string);
readonly "Root": (((({
readonly "kind": ("tree") & (string);
readonly "node": (Input_behaviourDefinitions["Node"]);
})) & { readonly "initial"?: never; readonly "states"?: never; }) | ((({
readonly "initial": (string);
readonly "kind": ("machine") & (string);
readonly "states": (ReadonlyArray<(Input_behaviourDefinitions["State"])> | (number extends ((Input_behaviourDefinitions["State"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "node"?: never; }));
readonly "Speed": (((number)) | ((string)));
readonly "State": ({
readonly "body"?: (((Input_behaviourDefinitions["Node"])) | ((null)));
readonly "enter"?: (ReadonlyArray<(Input_behaviourDefinitions["Action"])> | (number extends ((Input_behaviourDefinitions["Action"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "exit"?: (ReadonlyArray<(Input_behaviourDefinitions["Action"])> | (number extends ((Input_behaviourDefinitions["Action"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "name": (string);
readonly "transitions"?: (ReadonlyArray<(Input_behaviourDefinitions["Transition"])> | (number extends ((Input_behaviourDefinitions["Transition"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Transition": ({
readonly "after"?: ((number) | (null));
readonly "checks"?: (ReadonlyArray<(Input_behaviourDefinitions["Check"])> | (number extends ((Input_behaviourDefinitions["Check"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "to": (string);
});
readonly "UtilityOption": ({
readonly "children": (ReadonlyArray<(Input_behaviourDefinitions["Node"])> | (number extends ((Input_behaviourDefinitions["Node"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "name": (string);
readonly "scores": (ReadonlyArray<(Input_behaviourDefinitions["Consideration"])> | (number extends ((Input_behaviourDefinitions["Consideration"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Value": (((number)) | ((boolean)) | ((ReadonlyArray<(number)> & { readonly length: 3 })));
}
export type Input_dialogue = ((((Input_dialogueDefinitions["TypedDialogue"])) & { readonly "assets"?: never; readonly "yarn"?: never; }) | (((Input_dialogueDefinitions["YarnImport"])) & { readonly "barks"?: never; readonly "locale"?: never; readonly "name"?: never; readonly "nodes"?: never; readonly "smart"?: never; readonly "speakers"?: never; readonly "start"?: never; readonly "strings"?: never; readonly "vars"?: never; }));
export interface Input_dialogueDefinitions {
readonly "Bark": ({
readonly "cooldown"?: ((number) | (null));
readonly "duration"?: ((number) | (null));
readonly "id": (string);
readonly "lines": (ReadonlyArray<(Input_dialogueDefinitions["Line"])> | (number extends ((Input_dialogueDefinitions["Line"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "once"?: ((boolean) | (null));
readonly "priority"?: ((number) | (null));
readonly "random"?: ((boolean) | (null));
readonly "shared"?: ((number) | (null));
readonly "speaker"?: ((string) | (null));
readonly "steps"?: (ReadonlyArray<(Input_dialogueDefinitions["Step"])> | (number extends ((Input_dialogueDefinitions["Step"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "topic": (string);
readonly "weight"?: ((number) | (null));
readonly "when"?: (((Input_dialogueDefinitions["Expr"])) | ((null)));
});
readonly "Candidate": ({
readonly "condition"?: (((Input_dialogueDefinitions["Expr"])) | ((null)));
readonly "node": (string);
readonly "weight": (number);
});
readonly "Check": ({
readonly "active": (boolean);
readonly "stat": (string);
readonly "target": (Input_dialogueDefinitions["Expr"]);
});
readonly "Choice": ({
readonly "check"?: (((Input_dialogueDefinitions["Check"])) | ((null)));
readonly "condition"?: (((Input_dialogueDefinitions["Expr"])) | ((null)));
readonly "fail"?: ((string) | (null));
readonly "goto"?: ((string) | (null));
readonly "hide"?: (boolean);
readonly "id": (string);
readonly "kind"?: ((string) | (null));
readonly "reason"?: (((Input_dialogueDefinitions["Text"])) | ((null)));
readonly "steps"?: (ReadonlyArray<(Input_dialogueDefinitions["Step"])> | (number extends ((Input_dialogueDefinitions["Step"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "tags"?: (Readonly<Record<string, (string)>>);
readonly "text": (Input_dialogueDefinitions["Text"]);
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
readonly "Expr": (((({
readonly "kind": ("literal") & (string);
readonly "value": (Input_dialogueDefinitions["Value"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }) | ((({
readonly "kind": ("variable") & (string);
readonly "name": (string);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "id": (string);
readonly "kind": ("once") & (string);
})) & { readonly "args"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("not") & (string);
readonly "value": (Input_dialogueDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }) | ((({
readonly "kind": ("negate") & (string);
readonly "value": (Input_dialogueDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }) | ((({
readonly "kind": ("binary") & (string);
readonly "left": (Input_dialogueDefinitions["Expr"]);
readonly "op": (Input_dialogueDefinitions["Op"]);
readonly "right": (Input_dialogueDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "quest"?: never; readonly "value"?: never; }) | ((({
readonly "args": (ReadonlyArray<(Input_dialogueDefinitions["Expr"])> | (number extends ((Input_dialogueDefinitions["Expr"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
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
readonly "value": (Input_dialogueDefinitions["Value"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }));
readonly "Flow": ({
readonly "chapter"?: ((string) | (null));
readonly "dissolve"?: ((number) | (null));
readonly "ending"?: ((string) | (null));
readonly "music"?: (((Input_dialogueDefinitions["Music"])) | ((null)));
readonly "title"?: ((string) | (null));
});
readonly "Line": ({
readonly "expression"?: ((string) | (null));
/** Stable author identity, independent of source position. */
readonly "id": (string);
readonly "shot"?: ((string) | (null));
readonly "speaker"?: ((string) | (null));
readonly "text": (Input_dialogueDefinitions["Text"]);
readonly "voice"?: (((Input_dialogueDefinitions["ContentRef"])) | ((null)));
});
readonly "Music": ({
readonly "duck"?: ((number) | (null));
readonly "fade"?: ((number) | (null));
/** Omit to stop the current music. */
readonly "sound"?: (((Input_dialogueDefinitions["ContentRef"])) | ((null)));
readonly "volume"?: ((number) | (null));
});
readonly "Node": ({
readonly "choiceFallback"?: ((string) | (null));
readonly "choices"?: (ReadonlyArray<(Input_dialogueDefinitions["Choice"])> | (number extends ((Input_dialogueDefinitions["Choice"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "flow"?: (((Input_dialogueDefinitions["Flow"])) | ((null)));
readonly "id": (string);
readonly "segment"?: (((Input_dialogueDefinitions["ContentRef"])) | ((null)));
readonly "shot"?: ((string) | (null));
readonly "steps"?: (ReadonlyArray<(Input_dialogueDefinitions["Step"])> | (number extends ((Input_dialogueDefinitions["Step"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Tick count and fallback choice identity, not an array index. */
readonly "timer"?: ((readonly [(number), (string)]) | (null));
/** Owning source node; omit unless this is an explicit continuation block. */
readonly "title"?: ((string) | (null));
});
readonly "Op": ("add" | "sub" | "mul" | "div" | "rem" | "eq" | "ne" | "lt" | "le" | "gt" | "ge" | "and" | "or" | "xor") & (string);
readonly "QuestOp": (((({
readonly "kind": ("start") & (string);
readonly "quest": (string);
})) & { readonly "amount"?: never; readonly "objective"?: never; readonly "stage"?: never; }) | ((({
readonly "amount": (Input_dialogueDefinitions["Expr"]);
readonly "kind": ("progress") & (string);
readonly "objective": (string);
readonly "quest": (string);
})) & { readonly "stage"?: never; }) | ((({
readonly "kind": ("stage") & (string);
readonly "quest": (string);
readonly "stage": (string);
})) & { readonly "amount"?: never; readonly "objective"?: never; }) | ((({
readonly "kind": ("complete") & (string);
readonly "quest": (string);
})) & { readonly "amount"?: never; readonly "objective"?: never; readonly "stage"?: never; }) | ((({
readonly "kind": ("fail") & (string);
readonly "quest": (string);
})) & { readonly "amount"?: never; readonly "objective"?: never; readonly "stage"?: never; }));
readonly "Side": ("left" | "right" | "center") & (string);
readonly "Speaker": ({
readonly "expression"?: ((string) | (null));
readonly "id": (string);
readonly "name"?: ((string) | (null));
readonly "portrait"?: (((Input_dialogueDefinitions["ContentRef"])) | ((null)));
readonly "side"?: (((Input_dialogueDefinitions["Side"])) | ((null)));
});
readonly "Step": (((({
readonly "expression"?: ((string) | (null));
/** Stable author identity, independent of source position. */
readonly "id": (string);
readonly "kind": ("line") & (string);
readonly "shot"?: ((string) | (null));
readonly "speaker"?: ((string) | (null));
readonly "text": (Input_dialogueDefinitions["Text"]);
readonly "voice"?: (((Input_dialogueDefinitions["ContentRef"])) | ((null)));
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "name"?: never; readonly "node"?: never; readonly "operation"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("set") & (string);
readonly "name": (string);
readonly "value": (Input_dialogueDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "node"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "voice"?: never; }) | ((({
readonly "args": (ReadonlyArray<(Input_dialogueDefinitions["Expr"])> | (number extends ((Input_dialogueDefinitions["Expr"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("call") & (string);
readonly "name": (string);
})) & { readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "node"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "args": (ReadonlyArray<(Input_dialogueDefinitions["Expr"])> | (number extends ((Input_dialogueDefinitions["Expr"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("command") & (string);
readonly "name": (string);
})) & { readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "node"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "id": (string);
readonly "kind": ("mark-once") & (string);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "name"?: never; readonly "node"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "condition": (Input_dialogueDefinitions["Expr"]);
readonly "kind": ("if") & (string);
readonly "node": (string);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "name"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "kind": ("goto") & (string);
readonly "node": (string);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "name"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "kind": ("jump") & (string);
readonly "node": (Input_dialogueDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "name"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "kind": ("end") & (string);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "name"?: never; readonly "node"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "kind": ("emit") & (string);
readonly "name": (string);
readonly "value"?: (((Input_dialogueDefinitions["Expr"])) | ((null)));
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "node"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "voice"?: never; }) | ((({
readonly "kind": ("quest") & (string);
readonly "operation": (Input_dialogueDefinitions["QuestOp"]);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "name"?: never; readonly "node"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "continuation": (string);
readonly "kind": ("detour") & (string);
readonly "node": (string);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "name"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "continuation": (string);
readonly "kind": ("dynamic-detour") & (string);
readonly "node": (Input_dialogueDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "name"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "candidates": (ReadonlyArray<(Input_dialogueDefinitions["Candidate"])> | (number extends ((Input_dialogueDefinitions["Candidate"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("select") & (string);
})) & { readonly "args"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "name"?: never; readonly "node"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "kind": ("return") & (string);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "name"?: never; readonly "node"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }));
readonly "Text": ({
readonly "args"?: (Readonly<Record<string, (Input_dialogueDefinitions["Expr"])>>);
/** Localization key, not the rendered sentence. */
readonly "key": (string);
});
readonly "TypedDialogue": ({
readonly "barks"?: (ReadonlyArray<(Input_dialogueDefinitions["Bark"])> | (number extends ((Input_dialogueDefinitions["Bark"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "locale"?: ((string) | (null));
readonly "name": (string);
readonly "nodes": (ReadonlyArray<(Input_dialogueDefinitions["Node"])> | (number extends ((Input_dialogueDefinitions["Node"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Lazily evaluated variables expressed in the same typed expression tree. */
readonly "smart"?: (Readonly<Record<string, (Input_dialogueDefinitions["Expr"])>>);
readonly "speakers"?: (ReadonlyArray<(Input_dialogueDefinitions["Speaker"])> | (number extends ((Input_dialogueDefinitions["Speaker"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "start": (string);
/** Stable line key to ICU base text. ICU remains a string language. */
readonly "strings"?: (Readonly<Record<string, (string)>>);
readonly "vars"?: (ReadonlyArray<(Input_dialogueDefinitions["Variable"])> | (number extends ((Input_dialogueDefinitions["Variable"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Value": (((number)) | ((boolean)) | ((string)));
readonly "Variable": ({
readonly "global"?: (boolean);
readonly "name": (string);
readonly "value": (Input_dialogueDefinitions["Value"]);
});
readonly "YarnImport": ({
/** Yarn resource names mapped explicitly to checked content references. */
readonly "assets"?: (Readonly<Record<string, (Input_dialogueDefinitions["ContentRef"])>>);
/** Explicit Yarn source text from input.text; never automatically scanned. */
readonly "yarn": (string);
});
}
export type Input_district = ({
readonly "buildings"?: (((Input_districtDefinitions["Buildings"])) | ((null)));
readonly "cells"?: (((Input_districtDefinitions["Cells"])) | ((null)));
readonly "hlod"?: (((Input_districtDefinitions["HlodInput"])) | ((null)));
readonly "lots": (Input_districtDefinitions["Lots"]);
readonly "materials": (Input_districtDefinitions["ContentRef"]);
/** Omitted: merge generated static geometry. */
readonly "merge"?: ((boolean) | (null));
readonly "props"?: (ReadonlyArray<(Input_districtDefinitions["Prop"])> | (number extends ((Input_districtDefinitions["Prop"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "region": (Input_districtDefinitions["Region"]);
readonly "seed"?: ((number) | (null));
readonly "streets"?: (((Input_districtDefinitions["Streets"])) | ((null)));
readonly "terrain"?: (((Input_districtDefinitions["Terrain"])) | ((null)));
});
export interface Input_districtDefinitions {
readonly "Buildings": ({
readonly "density"?: ((number) | (null));
readonly "height"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "heightNoise"?: (((Input_districtDefinitions["HeightNoise"])) | ((null)));
readonly "rules"?: (ReadonlyArray<(Input_districtDefinitions["RuleUse"])> | (number extends ((Input_districtDefinitions["RuleUse"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Cells": ({
readonly "load"?: ((number) | (null));
readonly "size": (number);
});
readonly "Collider": (((({
readonly "kind": ("box") & (string);
readonly "size": (ReadonlyArray<(number)> & { readonly length: 3 });
})) & { readonly "height"?: never; readonly "radius"?: never; }) | ((({
readonly "height": (number);
readonly "kind": ("capsule") & (string);
readonly "radius": (number);
})) & { readonly "size"?: never; }));
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Grid": ({
readonly "jitter"?: ((number) | (null));
readonly "sidewalk"?: ((number) | (null));
readonly "size": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "turn"?: ((number) | (null));
readonly "width"?: ((number) | (null));
});
readonly "HeightNoise": ({
readonly "frequency": (number);
readonly "share": (number);
});
readonly "HlodInput": ({
/** Maximum proxy displacement in metres. Omitted/zero: size / 256;
a supplied positive value must be at most size / 4 (domain check). */
readonly "error"?: ((number) | (null));
/** Leaf cell side in metres; omitted: 64. */
readonly "size"?: ((number) | (null));
/** Longest proxy texture side in pixels; omitted: 1024. */
readonly "texels"?: ((number) | (null));
});
readonly "Lots": ({
readonly "depth": (ReadonlyArray<(number)> & { readonly length: 2 });
/** Least/greatest dimensions in metres; ordering checked by layout domain. */
readonly "width": (ReadonlyArray<(number)> & { readonly length: 2 });
});
readonly "Prop": ({
readonly "collider"?: (((Input_districtDefinitions["Collider"])) | ((null)));
readonly "corners"?: ((boolean) | (null));
readonly "every"?: ((number) | (null));
readonly "jitter"?: ((number) | (null));
readonly "kerb"?: ((number) | (null));
readonly "model": (Input_districtDefinitions["ContentRef"]);
readonly "name": (string);
readonly "street"?: ((string) | (null));
});
readonly "Region": (((({
readonly "kind": ("area") & (string);
readonly "points": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "depth"?: never; readonly "street"?: never; }) | ((({
readonly "depth"?: ((number) | (null));
readonly "kind": ("along") & (string);
readonly "street": (Input_districtDefinitions["Street"]);
})) & { readonly "points"?: never; }));
readonly "RuleUse": ({
readonly "grammar": (Input_districtDefinitions["ContentRef"]);
readonly "street"?: ((string) | (null));
readonly "weight"?: ((number) | (null));
});
readonly "Street": ({
readonly "name"?: ((string) | (null));
readonly "points": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "sidewalk"?: ((number) | (null));
readonly "width"?: ((number) | (null));
});
readonly "Streets": ({
/** Grid street settings are explicit, never inferred from authored streets. */
readonly "grid"?: (((Input_districtDefinitions["Grid"])) | ((null)));
readonly "kerb"?: ((number) | (null));
readonly "streets"?: (ReadonlyArray<(Input_districtDefinitions["Street"])> | (number extends ((Input_districtDefinitions["Street"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Terrain": ({
readonly "depth"?: ((number) | (null));
readonly "terrain": (Input_districtDefinitions["ContentRef"]);
});
}
/** Effect material input, not text containing another author language. */
export type Input_effect = ({
readonly "alpha"?: (((Input_effectDefinitions["EffectValue"])) | ((null)));
readonly "blend"?: (((Input_effectDefinitions["Blend"])) | ((null)));
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "custom"?: ((ReadonlyArray<(Input_effectDefinitions["EffectValue"])> | (number extends ((Input_effectDefinitions["EffectValue"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "doubleSided"?: ((boolean) | (null));
readonly "erosion"?: (((Input_effectDefinitions["EffectValue"])) | ((null)));
readonly "erosionEdge"?: (((Input_effectDefinitions["ErosionEdge"])) | ((null)));
readonly "erosionSoftness"?: ((number) | (null));
readonly "flow"?: (((Input_effectDefinitions["FlowInput"])) | ((null)));
readonly "flowMap"?: (((Input_effectDefinitions["EffectTextureSlot"])) | ((null)));
readonly "fresnel"?: (((Input_effectDefinitions["FresnelInput"])) | ((null)));
readonly "intensity"?: (((Input_effectDefinitions["EffectValue"])) | ((null)));
readonly "main"?: (((Input_effectDefinitions["EffectTextureSlot"])) | ((null)));
readonly "mask"?: (((Input_effectDefinitions["EffectTextureSlot"])) | ((null)));
readonly "noise"?: (((Input_effectDefinitions["EffectTextureSlot"])) | ((null)));
readonly "offsetU"?: (((Input_effectDefinitions["EffectValue"])) | ((null)));
readonly "offsetV"?: (((Input_effectDefinitions["EffectValue"])) | ((null)));
readonly "params"?: ((Readonly<Record<string, (Input_effectDefinitions["ShaderParam"])>>) | (null));
readonly "ramp"?: ((ReadonlyArray<(readonly [(number), (ReadonlyArray<(number)> & { readonly length: 4 })])> | (number extends ((readonly [(number), (ReadonlyArray<(number)> & { readonly length: 4 })])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "shader"?: (((Input_effectDefinitions["ContentRef"])) | ((null)));
readonly "softDepth"?: ((number) | (null));
});
export interface Input_effectDefinitions {
readonly "Blend": ("alpha" | "additive" | "multiply" | "subtractive") & (string);
readonly "Channel": ("rgba" | "red" | "green" | "blue" | "alpha") & (string);
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "EffectTextureSlot": ({
readonly "channel"?: (((Input_effectDefinitions["Channel"])) | ((null)));
readonly "texture"?: (((Input_effectDefinitions["ContentRef"])) | ((null)));
readonly "uv"?: (((Input_effectDefinitions["UvInput"])) | ((null)));
});
/** A scalar constant, per-instance range or ordered lifetime curve. */
readonly "EffectValue": (((number)) | ((Input_effectDefinitions["ValueInput"])));
readonly "ErosionEdge": ({
readonly "color": (ReadonlyArray<(number)> & { readonly length: 4 });
readonly "width": (number);
});
readonly "FlowInput": ({
readonly "frequency"?: ((number) | (null));
readonly "strength"?: ((number) | (null));
});
readonly "FresnelInput": ({
readonly "invert"?: ((boolean) | (null));
readonly "power": (number);
readonly "strength"?: ((number) | (null));
});
readonly "ShaderParam": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** UV transform evaluated over the surface's life. */
readonly "UvInput": ({
readonly "offset"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "polar"?: ((boolean) | (null));
readonly "rotate"?: ((number) | (null));
readonly "scroll"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "tile"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "ValueInput": (((({
readonly "kind": ("random") & (string);
readonly "max": (number);
readonly "min": (number);
})) & { readonly "keys"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("curve") & (string);
})) & { readonly "max"?: never; readonly "min"?: never; }));
}
/** Offline texture transport input; all appearance values are explicit.
Atlas dimensions, total work and nonzero light direction are domain checks. */
export type Input_effect_texture = ({
/** Nonnegative Beer-Lambert extinction/density/cube width. */
readonly "absorption": (number);
/** Nonnegative isotropic scattering light; does not change alpha or emission. */
readonly "ambient": (number);
/** Signed Y acceleration per unit heat. */
readonly "buoyancy": (number);
/** Linear scattering RGB, each 0..1, illuminated by ambient/direct/light. */
readonly "color": (ReadonlyArray<(number)> & { readonly length: 3 });
/** Sheet columns. Domain validation also checks columns <= frames. */
readonly "columns": (number);
/** Nonnegative exponential heat loss/second. */
readonly "cooling": (number);
/** Nonnegative exponential density loss/second. */
readonly "decay": (number);
/** Density/second; explosion uses initial density. Nonnegative. */
readonly "density": (number);
/** Source boundary/density fBm modulation, 0..1; zero is a smooth sphere. */
readonly "detail": (number);
/** Nonnegative directional light with volume self-shadow; RGB clamps to sRGB8. */
readonly "direct": (number);
/** Seconds per substep, >0..0.1; noise time is (frame+1)*steps*dt. */
readonly "dt": (number);
/** Linear emission RGB/unit heat, each 0..1, written separately to #emission. */
readonly "emission": (ReadonlyArray<(number)> & { readonly length: 3 });
/** Nonnegative radial source speed in cube widths/s. */
readonly "expansion": (number);
/** 1..256 chronological frames, in row-major sheet order. */
readonly "frames": (number);
/** Positive cycles per unit cube; three simplex octaves, lacunarity 2, gain 0.5. */
readonly "frequency": (number);
/** 8..96 volume cells per axis of the unit cube; no per-frame rescaling. */
readonly "grid": (number);
/** Heat/second; explosion uses initial heat. Nonnegative. */
readonly "heat": (number);
/** Nonnegative source duration in seconds; explosion injects only initially. */
readonly "inject": (number);
/** Nonzero direction toward the light, normalized at cook. View rays travel +Z. */
readonly "light": (ReadonlyArray<(number)> & { readonly length: 3 });
/** Density/heat transport, or the shared noise/Worley/curl kernels; no preset. */
readonly "mode": ("smoke" | "fire" | "explosion" | "noise" | "worley" | "curl") & (string);
/** 1..128 Jacobi pressure iterations per volume step. */
readonly "pressure": (number);
/** Injection radius in cube widths, >0..0.5. */
readonly "radius": (number);
/** Positive encoding range: signed noise/curl -range..range maps to 0..1;
Worley 0..range maps to 0..1. Values clamp. Unused for volume transport. */
readonly "range": (number);
/** Seed for the same fBm/curl/Worley kernels as og.procgen. */
readonly "seed": (number);
/** Square frame resolution, 8..1024. Sheet extents <=8192; <=16777216 frame pixels. */
readonly "size": (number);
/** Injection centre in the unit cube, Y up. */
readonly "source": (ReadonlyArray<(number)> & { readonly length: 3 });
/** 1..64 fixed simulation substeps between frames; frame zero follows one interval. */
readonly "steps": (number);
/** Nonnegative curl acceleration; on noise textures this is Z travel speed. */
readonly "turbulence": (number);
/** Nonnegative vorticity confinement in 1/s, scaled by cell width; zero disables it. */
readonly "vorticity": (number);
});
/** Sky source and explicit overrides. Omission is retained, not replaced by
normalized defaults: e.g. LDR extracts the sun while HDR keeps it. Source
compatibility, prefilter/sample conflicts, hemisphere/ground relationships,
image dimensions and power-of-two size remain environment domain checks. */
export type Input_environment = ({
/** Inverse tone-mapping strength; omitted: 0.25, zero disables it. */
readonly "expand"?: ((number) | (null));
/** Linear RGB ground tint, requires an upper-hemisphere equirectangular image. */
readonly "ground"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "hemisphere"?: (((Input_environmentDefinitions["Hemisphere"])) | ((null)));
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
readonly "source": (Input_environmentDefinitions["EnvironmentSource"]);
/** Omitted: extract for LDR images, keep for HDR and KTX2. */
readonly "sun"?: (((Input_environmentDefinitions["Sun"])) | ((null)));
/** Estimated LDR sun multiplier; omitted: 1. */
readonly "sunBoost"?: ((number) | (null));
});
export interface Input_environmentDefinitions {
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
export type Input_field = ({
/** Interleaved scalar channels per cell; their meaning belongs to the author. */
readonly "channels": (number);
/** Positive sample counts; x varies fastest, then y, then z. */
readonly "dimensions": (ReadonlyArray<(number)> & { readonly length: 3 });
/** First sample position in authored coordinate units. */
readonly "origin"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** Positive sample spacing in authored coordinate units. */
readonly "spacing"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** Finite f32 samples; dimensions product times channels, without normalization. */
readonly "values": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
export type Input_flipbook = ((((Input_flipbookDefinitions["TextureSheet"])) & { readonly "align"?: never; readonly "crop"?: never; readonly "dedupe"?: never; readonly "key"?: never; readonly "padding"?: never; readonly "scale"?: never; readonly "sources"?: never; }) | (((Input_flipbookDefinitions["AnimatedSheet"])) & { readonly "atlas"?: never; readonly "frames"?: never; readonly "grid"?: never; readonly "margin"?: never; readonly "spacing"?: never; readonly "texture"?: never; }));
export interface Input_flipbookDefinitions {
readonly "Alignment": ("feet" | "bbox" | "centroid" | "none") & (string);
readonly "AnimatedSheet": ({
/** Omitted: no alignment. */
readonly "align"?: (((Input_flipbookDefinitions["Alignment"])) | ((null)));
readonly "array"?: (boolean);
/** Explicit clips override an imported clip of the same name. */
readonly "clips"?: (ReadonlyArray<(Input_flipbookDefinitions["Clip"])> | (number extends ((Input_flipbookDefinitions["Clip"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "conform"?: (((Input_flipbookDefinitions["TilesetConform"])) | ((null)));
/** Omitted all: common frame size; each packs sources in separate sizes. */
readonly "crop"?: (((Input_flipbookDefinitions["Crop"])) | ((null)));
/** Omitted true: identical frames share a cell. */
readonly "dedupe"?: ((boolean) | (null));
readonly "filter"?: (((Input_flipbookDefinitions["Filter"])) | ((null)));
/** Omitted: no background keying. */
readonly "key"?: (((Input_flipbookDefinitions["ChromaKey"])) | ((null)));
/** Transparent texels around the common crop; omitted 1. */
readonly "padding"?: ((number) | (null));
readonly "pivot"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** Omitted 1; resample before measuring. */
readonly "scale"?: ((number) | (null));
/** Explicit animated GIF, WebP, APNG or still paths under this module's
assets/ or sources/. No declaration-relative or sibling lookup. */
readonly "sources": (ReadonlyArray<(Input_flipbookDefinitions["AnimatedSource"])> | (number extends ((Input_flipbookDefinitions["AnimatedSource"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "AnimatedSource": ({
readonly "align"?: (((Input_flipbookDefinitions["Alignment"])) | ((null)));
/** Omitted: the source's file stem. */
readonly "clip"?: ((string) | (null));
readonly "events"?: (ReadonlyArray<(readonly [(number), (string)])> | (number extends ((readonly [(number), (string)])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "fps"?: ((number) | (null));
readonly "mode"?: (((Input_flipbookDefinitions["FlipMode"])) | ((null)));
readonly "scale"?: ((number) | (null));
readonly "source": (string);
});
readonly "ChromaKey": ({
/** Omitted: auto-detect the flat border colour; explicit RGB8 otherwise. */
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
/** Also key enclosed pockets of background; omitted false. */
readonly "holes"?: (boolean);
/** Omitted 0.1. */
readonly "tolerance"?: ((number) | (null));
});
readonly "Clip": ({
/** One strictly positive duration per frame; otherwise use fps. */
readonly "durations"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "events"?: (ReadonlyArray<(readonly [(number), (string)])> | (number extends ((readonly [(number), (string)])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "fps"?: ((number) | (null));
/** Frame names, resolved only by the domain validator. */
readonly "frames": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "mode"?: (((Input_flipbookDefinitions["FlipMode"])) | ((null)));
readonly "name": (string);
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
readonly "Crop": ("all" | "each") & (string);
readonly "Filter": ("nearest" | "linear") & (string);
readonly "FlipMode": ("loop" | "once" | "pingPong") & (string);
readonly "Frame": ({
readonly "name": (string);
/** Bottom-left fractions; a pivot may lie outside the frame. */
readonly "pivot"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** Top-left x, y, width, height. Extents checked against the texture. */
readonly "rect": (ReadonlyArray<(number)> & { readonly length: 4 });
});
readonly "Grid": ({
readonly "columns"?: ((number) | (null));
readonly "rows"?: ((number) | (null));
readonly "size": (ReadonlyArray<(number)> & { readonly length: 2 });
});
readonly "TextureSheet": ({
readonly "array"?: (boolean);
/** Explicit Aseprite or TexturePacker exchange text from input.text. */
readonly "atlas"?: ((string) | (null));
readonly "clips"?: (ReadonlyArray<(Input_flipbookDefinitions["Clip"])> | (number extends ((Input_flipbookDefinitions["Clip"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "conform"?: (((Input_flipbookDefinitions["TilesetConform"])) | ((null)));
readonly "filter"?: (((Input_flipbookDefinitions["Filter"])) | ((null)));
readonly "frames"?: (ReadonlyArray<(Input_flipbookDefinitions["Frame"])> | (number extends ((Input_flipbookDefinitions["Frame"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Grid and explicit/imported frames are composed by the domain validator. */
readonly "grid"?: (((Input_flipbookDefinitions["Grid"])) | ((null)));
readonly "margin"?: (number);
/** Bottom-left fractions; a pivot may lie outside the frame. */
readonly "pivot"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "spacing"?: (number);
readonly "texture": (Input_flipbookDefinitions["ContentRef"]);
});
readonly "TilesetConform": (((({
readonly "kind": ("off") & (string);
})) & { readonly "size"?: never; }) | (({
readonly "kind": ("size") & (string);
readonly "size": (number);
})));
}
/** Pack membership does not change another face's own import settings or
charset. Coverage and script/language validity are checked from actual fonts. */
export type Input_font_extension = ({
readonly "faces": (ReadonlyArray<(Input_font_extensionDefinitions["ContentRef"])> | (number extends ((Input_font_extensionDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** External <language>.hyph dictionaries, consumed only when reached by
hyphens:auto and shipped locales; not independent runtime font assets. */
readonly "hyphenation"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": (Input_font_extensionDefinitions["FontKind"]);
readonly "languages"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "name": (string);
/** ISO 15924 script codes (e.g. Zsye for emoji-only packs). */
readonly "scripts"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
export interface Input_font_extensionDefinitions {
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "FontKind": ("font") & (string);
}
export type Input_grammar = ({
readonly "params"?: (Readonly<Record<string, (number)>>);
readonly "rules": (Readonly<Record<string, (ReadonlyArray<(Input_grammarDefinitions["Operation"])> | (number extends ((Input_grammarDefinitions["Operation"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))>>);
readonly "start": (string);
});
export interface Input_grammarDefinitions {
readonly "Axis": ("x" | "y") & (string);
readonly "CaseArm": ({
readonly "condition": (Input_grammarDefinitions["Condition"]);
readonly "rule": (string);
});
readonly "Condition": (((({
readonly "kind": ("wider") & (string);
readonly "value": (Input_grammarDefinitions["Number"]);
})) & { readonly "name"?: never; }) | ((({
readonly "kind": ("taller") & (string);
readonly "value": (Input_grammarDefinitions["Number"]);
})) & { readonly "name"?: never; }) | (({
readonly "kind": ("param") & (string);
readonly "name": (string);
readonly "value": (Input_grammarDefinitions["Number"]);
})));
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Faces": ({
readonly "back"?: ((string) | (null));
readonly "bottom"?: ((string) | (null));
readonly "front"?: ((string) | (null));
readonly "side"?: ((string) | (null));
readonly "sides"?: ((string) | (null));
readonly "top"?: ((string) | (null));
});
/** A literal or a named grammar parameter. This is not an expression evaluator. */
readonly "Number": (((number)) | ((string)));
readonly "Operation": (((({
readonly "depth": (Input_grammarDefinitions["Number"]);
readonly "jamb"?: ((string) | (null));
readonly "op": ("recess") & (string);
})) & { readonly "arms"?: never; readonly "axis"?: never; readonly "border"?: never; readonly "eave"?: never; readonly "faces"?: never; readonly "gable"?: never; readonly "inner"?: never; readonly "label"?: never; readonly "level"?: never; readonly "model"?: never; readonly "options"?: never; readonly "otherwise"?: never; readonly "overhang"?: never; readonly "parts"?: never; readonly "roof"?: never; readonly "slope"?: never; readonly "width"?: never; }) | ((({
readonly "level": (number);
readonly "op": ("detail") & (string);
})) & { readonly "arms"?: never; readonly "axis"?: never; readonly "border"?: never; readonly "depth"?: never; readonly "eave"?: never; readonly "faces"?: never; readonly "gable"?: never; readonly "inner"?: never; readonly "jamb"?: never; readonly "label"?: never; readonly "model"?: never; readonly "options"?: never; readonly "otherwise"?: never; readonly "overhang"?: never; readonly "parts"?: never; readonly "roof"?: never; readonly "slope"?: never; readonly "width"?: never; }) | ((({
readonly "label": (string);
readonly "op": ("label") & (string);
})) & { readonly "arms"?: never; readonly "axis"?: never; readonly "border"?: never; readonly "depth"?: never; readonly "eave"?: never; readonly "faces"?: never; readonly "gable"?: never; readonly "inner"?: never; readonly "jamb"?: never; readonly "level"?: never; readonly "model"?: never; readonly "options"?: never; readonly "otherwise"?: never; readonly "overhang"?: never; readonly "parts"?: never; readonly "roof"?: never; readonly "slope"?: never; readonly "width"?: never; }) | ((({
readonly "depth": (Input_grammarDefinitions["Number"]);
readonly "faces": (Input_grammarDefinitions["Faces"]);
readonly "op": ("extrude") & (string);
})) & { readonly "arms"?: never; readonly "axis"?: never; readonly "border"?: never; readonly "eave"?: never; readonly "gable"?: never; readonly "inner"?: never; readonly "jamb"?: never; readonly "label"?: never; readonly "level"?: never; readonly "model"?: never; readonly "options"?: never; readonly "otherwise"?: never; readonly "overhang"?: never; readonly "parts"?: never; readonly "roof"?: never; readonly "slope"?: never; readonly "width"?: never; }) | ((({
readonly "axis": (Input_grammarDefinitions["Axis"]);
readonly "op": ("split") & (string);
readonly "parts": (ReadonlyArray<(Input_grammarDefinitions["SplitPart"])> | (number extends ((Input_grammarDefinitions["SplitPart"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "arms"?: never; readonly "border"?: never; readonly "depth"?: never; readonly "eave"?: never; readonly "faces"?: never; readonly "gable"?: never; readonly "inner"?: never; readonly "jamb"?: never; readonly "label"?: never; readonly "level"?: never; readonly "model"?: never; readonly "options"?: never; readonly "otherwise"?: never; readonly "overhang"?: never; readonly "roof"?: never; readonly "slope"?: never; readonly "width"?: never; }) | ((({
readonly "border": (string);
readonly "inner": (string);
readonly "op": ("inset") & (string);
readonly "width": (Input_grammarDefinitions["Number"]);
})) & { readonly "arms"?: never; readonly "axis"?: never; readonly "depth"?: never; readonly "eave"?: never; readonly "faces"?: never; readonly "gable"?: never; readonly "jamb"?: never; readonly "label"?: never; readonly "level"?: never; readonly "model"?: never; readonly "options"?: never; readonly "otherwise"?: never; readonly "overhang"?: never; readonly "parts"?: never; readonly "roof"?: never; readonly "slope"?: never; }) | ((({
readonly "eave"?: ((string) | (null));
readonly "gable"?: ((string) | (null));
readonly "op": ("roof") & (string);
readonly "overhang"?: (((Input_grammarDefinitions["Number"])) | ((null)));
readonly "roof": (Input_grammarDefinitions["Roof"]);
readonly "slope": (string);
})) & { readonly "arms"?: never; readonly "axis"?: never; readonly "border"?: never; readonly "depth"?: never; readonly "faces"?: never; readonly "inner"?: never; readonly "jamb"?: never; readonly "label"?: never; readonly "level"?: never; readonly "model"?: never; readonly "options"?: never; readonly "otherwise"?: never; readonly "parts"?: never; readonly "width"?: never; }) | ((({
readonly "model": (Input_grammarDefinitions["ContentRef"]);
readonly "op": ("insert") & (string);
})) & { readonly "arms"?: never; readonly "axis"?: never; readonly "border"?: never; readonly "depth"?: never; readonly "eave"?: never; readonly "faces"?: never; readonly "gable"?: never; readonly "inner"?: never; readonly "jamb"?: never; readonly "label"?: never; readonly "level"?: never; readonly "options"?: never; readonly "otherwise"?: never; readonly "overhang"?: never; readonly "parts"?: never; readonly "roof"?: never; readonly "slope"?: never; readonly "width"?: never; }) | ((({
readonly "op": ("pick") & (string);
readonly "options": (ReadonlyArray<(Input_grammarDefinitions["WeightedRule"])> | (number extends ((Input_grammarDefinitions["WeightedRule"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "arms"?: never; readonly "axis"?: never; readonly "border"?: never; readonly "depth"?: never; readonly "eave"?: never; readonly "faces"?: never; readonly "gable"?: never; readonly "inner"?: never; readonly "jamb"?: never; readonly "label"?: never; readonly "level"?: never; readonly "model"?: never; readonly "otherwise"?: never; readonly "overhang"?: never; readonly "parts"?: never; readonly "roof"?: never; readonly "slope"?: never; readonly "width"?: never; }) | ((({
readonly "arms": (ReadonlyArray<(Input_grammarDefinitions["CaseArm"])> | (number extends ((Input_grammarDefinitions["CaseArm"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "op": ("case") & (string);
readonly "otherwise"?: ((string) | (null));
})) & { readonly "axis"?: never; readonly "border"?: never; readonly "depth"?: never; readonly "eave"?: never; readonly "faces"?: never; readonly "gable"?: never; readonly "inner"?: never; readonly "jamb"?: never; readonly "label"?: never; readonly "level"?: never; readonly "model"?: never; readonly "options"?: never; readonly "overhang"?: never; readonly "parts"?: never; readonly "roof"?: never; readonly "slope"?: never; readonly "width"?: never; }) | ((({
readonly "op": ("drop") & (string);
})) & { readonly "arms"?: never; readonly "axis"?: never; readonly "border"?: never; readonly "depth"?: never; readonly "eave"?: never; readonly "faces"?: never; readonly "gable"?: never; readonly "inner"?: never; readonly "jamb"?: never; readonly "label"?: never; readonly "level"?: never; readonly "model"?: never; readonly "options"?: never; readonly "otherwise"?: never; readonly "overhang"?: never; readonly "parts"?: never; readonly "roof"?: never; readonly "slope"?: never; readonly "width"?: never; }));
readonly "Roof": (((({
readonly "kind": ("flat") & (string);
})) & { readonly "angle"?: never; }) | (({
readonly "angle": (Input_grammarDefinitions["Number"]);
readonly "kind": ("gable") & (string);
})) | (({
readonly "angle": (Input_grammarDefinitions["Number"]);
readonly "kind": ("hip") & (string);
})));
readonly "Size": ("abs" | "fill" | "repeat") & (string);
readonly "SplitPart": ({
readonly "length": (Input_grammarDefinitions["Number"]);
readonly "rule": (string);
readonly "size": (Input_grammarDefinitions["Size"]);
});
readonly "WeightedRule": ({
readonly "rule": (string);
readonly "weight": (number);
});
}
/** An icon declaration; SVG validation and distance-field baking stay native. */
export type Input_icons = ({
/** (name, SVG file stem), in index order.
Unique names and paths beneath source are checked by the icon validator. */
readonly "icons": (ReadonlyArray<(readonly [(string), (string)])> | (number extends ((readonly [(string), (string)])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** The join of a stroked path that names none (`join`; SVG's miter). */
readonly "join"?: (Input_iconsDefinitions["Join"]);
/** MSDF distance range in pixels. */
readonly "range"?: (number);
/** Square MSDF glyph size in pixels. */
readonly "size"?: (number);
/** Canonical directory of SVGs relative to the owning module, for example
assets/icons. No source-file-relative or asset-root fallback. */
readonly "source"?: (string);
/** A toned icon's outline width and shadow offset, in 24ths of the em. */
readonly "sticker"?: (readonly [(number), (number), (number)]);
});
export interface Input_iconsDefinitions {
/** How a stroke's segments meet at a corner (SVG's `stroke-linejoin`). */
readonly "Join": ((("miter") & (string)) | (("round") & (string)) | (("bevel") & (string)));
}
export type Input_import = (((({
readonly "animation"?: (((Input_importDefinitions["Animation"])) | ((null)));
readonly "collider"?: (((Input_importDefinitions["Collider"])) | ((null)));
readonly "conform"?: (((Input_importDefinitions["Conform"])) | ((null)));
readonly "fracture"?: (((Input_importDefinitions["Fracture"])) | ((null)));
/** Source forward axis rotated to +Z; must be perpendicular to up. */
readonly "front"?: (((Input_importDefinitions["Axis2"])) | ((null)));
readonly "impostor"?: ((boolean) | (null));
readonly "impostorBelow"?: ((boolean) | (null));
readonly "impostorGrid"?: ((number) | (null));
/** Atlas width (grid × frame size) must not exceed 4096 pixels. */
readonly "impostorSize"?: ((number) | (null));
readonly "kind": ("model") & (string);
readonly "lodError"?: ((number) | (null));
/** At most eight strictly decreasing fractions beginning at one. */
readonly "lods"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "process"?: (((Input_importDefinitions["Process"])) | ((null)));
readonly "raycast"?: ((boolean) | (null));
/** Opt-in probe-region skin relaxation (12 passes); absent means off. */
readonly "relaxSkin"?: ((boolean) | (null));
/** GLB/glTF/OBJ/FBX/VOX input. External buffers and images are discovered by
the importer and must enter the closed input/dependency graph as well. */
readonly "source": (string);
readonly "streamTextures"?: ((boolean) | (null));
readonly "up"?: (((Input_importDefinitions["Axis2"])) | ((null)));
readonly "voxClips"?: (ReadonlyArray<(Input_importDefinitions["VoxClip"])> | (number extends ((Input_importDefinitions["VoxClip"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "voxFps"?: ((number) | (null));
readonly "voxOrigin"?: (((Input_importDefinitions["VoxOrigin"])) | ((null)));
readonly "voxScale"?: ((number) | (null));
})) & { readonly "artboards"?: never; readonly "bitrate"?: never; readonly "chunk"?: never; readonly "codec"?: never; readonly "fonts"?: never; readonly "level"?: never; readonly "normal"?: never; readonly "pixel"?: never; readonly "settings"?: never; readonly "solid"?: never; readonly "stream"?: never; readonly "sway"?: never; readonly "textures"?: never; readonly "tileset"?: never; readonly "tilesets"?: never; readonly "unit"?: never; readonly "usage"?: never; }) | ((({
readonly "conform"?: (((Input_importDefinitions["TextureConform"])) | ((null)));
readonly "kind": ("texture") & (string);
readonly "normal"?: (((Input_importDefinitions["Normal"])) | ((null)));
readonly "source": (string);
/** Pixel textures have no mip chain and cannot stream (domain validation). */
readonly "stream"?: ((boolean) | (null));
readonly "sway"?: (((Input_importDefinitions["Sway"])) | ((null)));
/** Distinct from the root import kind discriminant. */
readonly "usage"?: (((Input_importDefinitions["TextureUsage"])) | ((null)));
})) & { readonly "animation"?: never; readonly "artboards"?: never; readonly "bitrate"?: never; readonly "chunk"?: never; readonly "codec"?: never; readonly "collider"?: never; readonly "fonts"?: never; readonly "fracture"?: never; readonly "front"?: never; readonly "impostor"?: never; readonly "impostorBelow"?: never; readonly "impostorGrid"?: never; readonly "impostorSize"?: never; readonly "level"?: never; readonly "lodError"?: never; readonly "lods"?: never; readonly "pixel"?: never; readonly "process"?: never; readonly "raycast"?: never; readonly "relaxSkin"?: never; readonly "settings"?: never; readonly "solid"?: never; readonly "streamTextures"?: never; readonly "textures"?: never; readonly "tileset"?: never; readonly "tilesets"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "voxClips"?: never; readonly "voxFps"?: never; readonly "voxOrigin"?: never; readonly "voxScale"?: never; }) | ((({
/** Rounded to whole kb/s; absent selects the mono/stereo codec default. */
readonly "bitrate"?: ((number) | (null));
readonly "chunk"?: ((number) | (null));
readonly "codec"?: (((Input_importDefinitions["Codec"])) | ((null)));
readonly "kind": ("sound") & (string);
readonly "source": (string);
/** QOA only. Absent selects automatic streaming above 1 MiB; Opus streams
its own codec blocks rather than QOA chunks. */
readonly "stream"?: ((boolean) | (null));
})) & { readonly "animation"?: never; readonly "artboards"?: never; readonly "collider"?: never; readonly "conform"?: never; readonly "fonts"?: never; readonly "fracture"?: never; readonly "front"?: never; readonly "impostor"?: never; readonly "impostorBelow"?: never; readonly "impostorGrid"?: never; readonly "impostorSize"?: never; readonly "level"?: never; readonly "lodError"?: never; readonly "lods"?: never; readonly "normal"?: never; readonly "pixel"?: never; readonly "process"?: never; readonly "raycast"?: never; readonly "relaxSkin"?: never; readonly "settings"?: never; readonly "solid"?: never; readonly "streamTextures"?: never; readonly "sway"?: never; readonly "textures"?: never; readonly "tileset"?: never; readonly "tilesets"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "usage"?: never; readonly "voxClips"?: never; readonly "voxFps"?: never; readonly "voxOrigin"?: never; readonly "voxScale"?: never; }) | ((({
readonly "kind": ("font") & (string);
/** Omitted: outline font. Pixel grid changes only this font's raster mode. */
readonly "pixel"?: ((number) | (null));
readonly "source": (string);
})) & { readonly "animation"?: never; readonly "artboards"?: never; readonly "bitrate"?: never; readonly "chunk"?: never; readonly "codec"?: never; readonly "collider"?: never; readonly "conform"?: never; readonly "fonts"?: never; readonly "fracture"?: never; readonly "front"?: never; readonly "impostor"?: never; readonly "impostorBelow"?: never; readonly "impostorGrid"?: never; readonly "impostorSize"?: never; readonly "level"?: never; readonly "lodError"?: never; readonly "lods"?: never; readonly "normal"?: never; readonly "process"?: never; readonly "raycast"?: never; readonly "relaxSkin"?: never; readonly "settings"?: never; readonly "solid"?: never; readonly "stream"?: never; readonly "streamTextures"?: never; readonly "sway"?: never; readonly "textures"?: never; readonly "tileset"?: never; readonly "tilesets"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "usage"?: never; readonly "voxClips"?: never; readonly "voxFps"?: never; readonly "voxOrigin"?: never; readonly "voxScale"?: never; }) | ((({
readonly "kind": ("tilemap") & (string);
readonly "level"?: ((string) | (null));
readonly "solid"?: (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Explicit Tiled/LDtk exchange text; never an engine author DSL. */
readonly "source": (string);
/** Embedded Tiled/LDtk image paths as written in the map to texture refs. */
readonly "textures"?: (Readonly<Record<string, (Input_importDefinitions["ContentRef"])>>);
readonly "tileset"?: (((Input_importDefinitions["ContentRef"])) | ((null)));
/** External TSX/TSJ or LDtk image paths as written in the map to tileset refs. */
readonly "tilesets"?: (Readonly<Record<string, (Input_importDefinitions["ContentRef"])>>);
/** World units per tile width. */
readonly "unit"?: ((number) | (null));
})) & { readonly "animation"?: never; readonly "artboards"?: never; readonly "bitrate"?: never; readonly "chunk"?: never; readonly "codec"?: never; readonly "collider"?: never; readonly "conform"?: never; readonly "fonts"?: never; readonly "fracture"?: never; readonly "front"?: never; readonly "impostor"?: never; readonly "impostorBelow"?: never; readonly "impostorGrid"?: never; readonly "impostorSize"?: never; readonly "lodError"?: never; readonly "lods"?: never; readonly "normal"?: never; readonly "pixel"?: never; readonly "process"?: never; readonly "raycast"?: never; readonly "relaxSkin"?: never; readonly "settings"?: never; readonly "stream"?: never; readonly "streamTextures"?: never; readonly "sway"?: never; readonly "up"?: never; readonly "usage"?: never; readonly "voxClips"?: never; readonly "voxFps"?: never; readonly "voxOrigin"?: never; readonly "voxScale"?: never; }) | ((({
/** Omitted: all artboards. Explicit selection retains authored order. */
readonly "artboards"?: ((ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "fonts"?: (Readonly<Record<string, (Input_importDefinitions["FontSource"])>>);
readonly "kind": ("rive") & (string);
readonly "settings"?: (Readonly<Record<string, (Input_importDefinitions["Artboard"])>>);
readonly "source": (string);
})) & { readonly "animation"?: never; readonly "bitrate"?: never; readonly "chunk"?: never; readonly "codec"?: never; readonly "collider"?: never; readonly "conform"?: never; readonly "fracture"?: never; readonly "front"?: never; readonly "impostor"?: never; readonly "impostorBelow"?: never; readonly "impostorGrid"?: never; readonly "impostorSize"?: never; readonly "level"?: never; readonly "lodError"?: never; readonly "lods"?: never; readonly "normal"?: never; readonly "pixel"?: never; readonly "process"?: never; readonly "raycast"?: never; readonly "relaxSkin"?: never; readonly "solid"?: never; readonly "stream"?: never; readonly "streamTextures"?: never; readonly "sway"?: never; readonly "textures"?: never; readonly "tileset"?: never; readonly "tilesets"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "usage"?: never; readonly "voxClips"?: never; readonly "voxFps"?: never; readonly "voxOrigin"?: never; readonly "voxScale"?: never; }));
export interface Input_importDefinitions {
readonly "Align": ("center" | "top" | "bottom" | "left" | "right" | "top-left" | "top-right" | "bottom-left" | "bottom-right") & (string);
readonly "Anchor": (((({
readonly "kind": ("none") & (string);
})) & { readonly "bottom"?: never; readonly "distance"?: never; readonly "ends"?: never; readonly "normal"?: never; }) | ((({
readonly "bottom": (boolean);
readonly "ends": (boolean);
readonly "kind": ("supports") & (string);
})) & { readonly "distance"?: never; readonly "normal"?: never; }) | ((({
readonly "distance": (number);
readonly "kind": ("plane") & (string);
readonly "normal": (ReadonlyArray<(number)> & { readonly length: 3 });
})) & { readonly "bottom"?: never; readonly "ends"?: never; }));
readonly "Animation": ({
readonly "clips"?: (ReadonlyArray<(Input_importDefinitions["Clip"])> | (number extends ((Input_importDefinitions["Clip"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "compress"?: ((boolean) | (null));
readonly "events"?: (ReadonlyArray<(Input_importDefinitions["Event"])> | (number extends ((Input_importDefinitions["Event"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "footIk"?: ((boolean) | (null));
readonly "libraries"?: (ReadonlyArray<(Input_importDefinitions["ContentRef"])> | (number extends ((Input_importDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "mirrors"?: (ReadonlyArray<(Input_importDefinitions["Mirror"])> | (number extends ((Input_importDefinitions["Mirror"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "rate"?: ((number) | (null));
readonly "rig"?: (((Input_importDefinitions["ContentRef"])) | ((null)));
readonly "tolerance"?: ((number) | (null));
});
readonly "Artboard": ({
readonly "align"?: (((Input_importDefinitions["Align"])) | ((null)));
readonly "fit"?: (((Input_importDefinitions["Fit"])) | ((null)));
readonly "name"?: ((string) | (null));
readonly "role"?: (((Input_importDefinitions["Role"])) | ((null)));
readonly "stateMachine"?: ((string) | (null));
});
readonly "Axis": ("x" | "y" | "z") & (string);
readonly "Axis2": ("+x" | "-x" | "+y" | "-y" | "+z" | "-z") & (string);
readonly "Bake": ({
/** Zero fills all uncovered texels. */
readonly "dilate"?: ((number) | (null));
readonly "distance"?: ((number) | (null));
readonly "normals"?: ((boolean) | (null));
readonly "padding"?: ((number) | (null));
readonly "size"?: ((number) | (null));
});
readonly "Channel": ({
readonly "interpolation"?: (((Input_importDefinitions["Interpolation"])) | ((null)));
readonly "keys": (ReadonlyArray<(Input_importDefinitions["Key"])> | (number extends ((Input_importDefinitions["Key"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "node": (string);
readonly "path": (Input_importDefinitions["ChannelPath"]);
});
readonly "ChannelPath": ("translation" | "rotation" | "scale" | "weights" | "visibility") & (string);
readonly "Clip": ({
readonly "channels": (ReadonlyArray<(Input_importDefinitions["Channel"])> | (number extends ((Input_importDefinitions["Channel"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "name": (string);
});
readonly "Codec": ("qoa" | "opus") & (string);
readonly "Collider": (((boolean)) | ((Input_importDefinitions["ColliderMethod"])));
readonly "ColliderMethod": ("convex" | "vhacd") & (string);
readonly "Conform": ("off" | "character" | "effect") & (string);
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Decimate": ({
readonly "crossColorBorders"?: ((boolean) | (null));
readonly "lockBorder"?: ((boolean) | (null));
readonly "maxError"?: ((number) | (null));
readonly "mergeCharts"?: ((number) | (null));
readonly "ratio"?: ((number) | (null));
/** Zero leaves the triangle-count budget unset. */
readonly "target"?: ((number) | (null));
});
readonly "Event": ({
readonly "clip": (string);
readonly "marker": (string);
readonly "time": (number);
});
readonly "Fit": ("contain" | "cover" | "fill" | "none" | "scale-down" | "fit-width" | "fit-height") & (string);
readonly "FontSource": ({
/** Omitted: use the embedded font with this source name. */
readonly "source"?: ((string) | (null));
});
readonly "Fracture": ({
readonly "anchor"?: (((Input_importDefinitions["Anchor"])) | ((null)));
readonly "brick"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "children"?: ((number) | (null));
readonly "chunks"?: ((number) | (null));
readonly "grain"?: (((Input_importDefinitions["Axis"])) | ((null)));
readonly "impact"?: (((Input_importDefinitions["Impact"])) | ((null)));
readonly "interior"?: (((Input_importDefinitions["Material"])) | ((null)));
readonly "interiorScale"?: ((number) | (null));
readonly "mortar"?: (((Input_importDefinitions["Material"])) | ((null)));
readonly "pattern"?: (((Input_importDefinitions["Pattern"])) | ((null)));
readonly "rings"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
/** Explicit centres replace sampling; count 2..128, distinct, volume
Voronoi only, exclusive with chunks/impact (domain validation). */
readonly "sites"?: ((ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "style"?: (((Input_importDefinitions["Style"])) | ((null)));
});
readonly "Impact": ({
readonly "position": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "radius": (number);
});
readonly "Interpolation": ("step" | "linear" | "cubicSpline") & (string);
readonly "Joint": ({
readonly "name": (string);
/** Empty string denotes root; all other parents must precede the child. */
readonly "parent": (string);
readonly "position": (ReadonlyArray<(number)> & { readonly length: 3 });
});
readonly "Key": ({
readonly "time": (number);
/** Metres, xyzw unit quaternion, scale factors, morph weights or 0/1.
Cubic spline keys contain in-tangent, value, out-tangent (per second). */
readonly "values": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
/** Omitted fields remain absent until the material domain normalizes them. */
readonly "Material": ({
/** Alpha-mask cutoff takes precedence over opacity blending. */
readonly "alphaTest"?: ((number) | (null));
/** Ambient occlusion in red. */
readonly "aoMap"?: ((string) | (null));
readonly "autumnColor"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "autumnColor2"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "clearcoat"?: ((number) | (null));
readonly "clearcoatRoughness"?: ((number) | (null));
/** Linear base RGB (glTF factor); not a runtime colour handle. */
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "domeNormals"?: ((number) | (null));
readonly "doubleSided"?: ((boolean) | (null));
/** Linear emitted RGB, multiplied by emissiveIntensity. */
readonly "emissive"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "emissiveIntensity"?: ((number) | (null));
readonly "grime"?: ((number) | (null));
readonly "grimeColor"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "grimeHeight"?: ((number) | (null));
readonly "ior"?: ((number) | (null));
/** Source image inputs; not cooked texture references. Resolution and path
containment are checked at the input boundary. */
readonly "map"?: ((string) | (null));
readonly "metalness"?: ((number) | (null));
/** Roughness in green, metalness in blue. */
readonly "metalnessMap"?: ((string) | (null));
readonly "normalMap"?: ((string) | (null));
readonly "normalScale"?: ((number) | (null));
readonly "opacity"?: ((number) | (null));
readonly "roughness"?: ((number) | (null));
/** Zero evergreen; one deciduous. */
readonly "season"?: ((number) | (null));
readonly "seasonShift"?: ((number) | (null));
readonly "stains"?: ((number) | (null));
readonly "stochastic"?: ((number) | (null));
readonly "thickness"?: ((number) | (null));
/** Diffuse transmission through thin leaves; enables both faces. */
readonly "translucency"?: ((number) | (null));
readonly "transmission"?: ((number) | (null));
});
readonly "Mirror": ({
readonly "clip": (string);
readonly "name"?: ((string) | (null));
});
readonly "Normal": (((({
readonly "kind": ("auto") & (string);
})) & { readonly "source"?: never; }) | (({
readonly "kind": ("image") & (string);
readonly "source": (string);
})));
readonly "Pattern": ("auto" | "volume" | "slab") & (string);
readonly "Process": ({
readonly "bake"?: (((Input_importDefinitions["Bake"])) | ((null)));
readonly "decimate"?: (((Input_importDefinitions["Decimate"])) | ((null)));
readonly "project"?: (((Input_importDefinitions["Project"])) | ((null)));
readonly "repair"?: (((Input_importDefinitions["Repair"])) | ((null)));
readonly "rig"?: (((Input_importDefinitions["Rig"])) | ((null)));
readonly "transfer"?: (((Input_importDefinitions["Transfer"])) | ((null)));
readonly "unwrap"?: (((Input_importDefinitions["Unwrap"])) | ((null)));
});
readonly "Project": ({
readonly "padding"?: ((number) | (null));
readonly "size"?: ((number) | (null));
readonly "views": (ReadonlyArray<(Input_importDefinitions["View"])> | (number extends ((Input_importDefinitions["View"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Projection": (((({
readonly "fov": (number);
readonly "kind": ("perspective") & (string);
})) & { readonly "ortho"?: never; }) | ((({
readonly "kind": ("orthographic") & (string);
readonly "ortho": (number);
})) & { readonly "fov"?: never; }));
readonly "Repair": ({
readonly "floaterFraction"?: ((number) | (null));
readonly "floaters"?: ((number) | (null));
readonly "fuse"?: ((boolean) | (null));
readonly "holeEdges"?: ((number) | (null));
readonly "holesThrough"?: ((boolean) | (null));
readonly "minEdge"?: ((number) | (null));
readonly "nonManifold"?: ((boolean) | (null));
readonly "thicken"?: ((number) | (null));
readonly "weld"?: ((number) | (null));
readonly "weldAll"?: ((boolean) | (null));
});
readonly "Rig": (((({
readonly "kind": ("auto") & (string);
})) & { readonly "joints"?: never; }) | (({
readonly "joints": (ReadonlyArray<(Input_importDefinitions["Joint"])> | (number extends ((Input_importDefinitions["Joint"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("joints") & (string);
})));
readonly "Role": ("button" | "link" | "text" | "heading" | "image" | "checkbox" | "radio" | "switch" | "slider" | "spinbutton" | "textbox" | "combobox" | "listbox" | "option" | "tab" | "tablist" | "tabpanel" | "radiogroup" | "progressbar" | "meter" | "timer" | "status" | "alert" | "log" | "dialog" | "alertdialog" | "tooltip" | "separator" | "group" | "region" | "navigation" | "toolbar" | "menu" | "menubar" | "menuitem" | "menuitemcheckbox" | "menuitemradio" | "list" | "listitem" | "tree" | "treeitem" | "table" | "grid" | "row" | "rowheader" | "columnheader" | "cell" | "gridcell" | "figure" | "scrollbar") & (string);
readonly "Style": ("voronoi" | "concrete" | "brick" | "wood" | "glass") & (string);
readonly "Sway": ({
readonly "kind": (Input_importDefinitions["SwayKind"]);
readonly "parts"?: ((string) | (null));
});
readonly "SwayKind": ("clumps" | "tiers") & (string);
readonly "TextureConform": (((({
readonly "kind": ("off") & (string);
})) & { readonly "size"?: never; }) | (({
readonly "kind": ("sprite") & (string);
readonly "size"?: ((number) | (null));
})) | (({
readonly "kind": ("ui") & (string);
readonly "size"?: ((number) | (null));
})) | (({
readonly "kind": ("auto") & (string);
readonly "size"?: ((number) | (null));
})));
readonly "TextureUsage": ("color" | "normal" | "linear" | "single" | "pixel") & (string);
readonly "Transfer": ({
readonly "padding"?: ((number) | (null));
readonly "size"?: ((number) | (null));
readonly "source": (Input_importDefinitions["ContentRef"]);
});
readonly "Unwrap": ({
readonly "angle"?: ((number) | (null));
readonly "conformal"?: ((boolean) | (null));
readonly "padding"?: ((number) | (null));
readonly "resolution"?: ((number) | (null));
readonly "seams"?: ((boolean) | (null));
readonly "shared"?: ((boolean) | (null));
});
readonly "View": ({
/** Square RGBA source; actual dimensions/decoding are input-domain checks. */
readonly "image": (string);
readonly "position": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "projection": (Input_importDefinitions["Projection"]);
readonly "target": (ReadonlyArray<(number)> & { readonly length: 3 });
});
readonly "VoxClip": ({
/** Inclusive frame indices; first <= last and source bounds are domain checks. */
readonly "first": (number);
readonly "last": (number);
readonly "name": (string);
});
readonly "VoxOrigin": ("center" | "bottom") & (string);
}
export type Input_lightmap = ({
readonly "models"?: (ReadonlyArray<(Input_lightmapDefinitions["BakedModel"])> | (number extends ((Input_lightmapDefinitions["BakedModel"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** External RGBA16F KTX2 array, consumed by this lightmap only. */
readonly "pages": (string);
readonly "samples"?: ((number) | (null));
readonly "scene": (Input_lightmapDefinitions["ContentRef"]);
readonly "sceneHash": (Input_lightmapDefinitions["SourceHash"]);
readonly "seconds"?: ((number) | (null));
readonly "volumes"?: (ReadonlyArray<(Input_lightmapDefinitions["BakedVolume"])> | (number extends ((Input_lightmapDefinitions["BakedVolume"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
export interface Input_lightmapDefinitions {
/** HLOD nodes are generated by the baked scene, not independently authored
model declarations. Their node number binds through that scene's producer. */
readonly "BakedAsset": ((((Input_lightmapDefinitions["ContentRef"])) & { readonly "hlodProxy"?: never; }) | (((Input_lightmapDefinitions["HlodProxy"])) & { readonly "$ogRef"?: never; }));
readonly "BakedModel": ({
readonly "asset": (Input_lightmapDefinitions["BakedAsset"]);
readonly "entity": (string);
readonly "hash": (Input_lightmapDefinitions["SourceHash"]);
readonly "parts"?: (ReadonlyArray<(Input_lightmapDefinitions["BakedPart"])> | (number extends ((Input_lightmapDefinitions["BakedPart"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "BakedPart": ({
readonly "indices"?: (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Flat u, v, page triples per remapped vertex; accepts Float32Array. */
readonly "uv"?: (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "vertices"?: ((number) | (null));
readonly "xref"?: (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "BakedVolume": ({
readonly "counts": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "entity": (string);
readonly "fade"?: ((number) | (null));
readonly "max": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "min": (ReadonlyArray<(number)> & { readonly length: 3 });
/** Flat RGB coefficients [c1.x,c1.y,c1.z,c0] per probe (12 floats),
x fastest; accepts Float32Array. */
readonly "probes"?: (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
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
readonly "HlodProxy": ({
readonly "hlodProxy": (number);
});
/** Full-width u64 source hashes are hex text, not lossy JS numbers. */
readonly "SourceHash": (string);
}
/** A look overlays reflected resource patches on explicitly named bases.
Inheritance order/cycles, preset variants and asset identities are domain checks. */
export type Input_look = ({
readonly "ArtCamera"?: (((Input_lookDefinitions["Reflected_og_cook::cook::source_schemas::procedural::ArtCamera"])) | ((null)));
readonly "ArtLine"?: (((Input_lookDefinitions["Reflected_og_cook::cook::source_schemas::procedural::ArtLine"])) | ((null)));
readonly "ArtMotion"?: (((Input_lookDefinitions["Reflected_og_cook::cook::source_schemas::procedural::ArtMotion"])) | ((null)));
readonly "ArtPalette"?: (((Input_lookDefinitions["Reflected_og_cook::cook::source_schemas::procedural::ArtPalette"])) | ((null)));
readonly "ArtPixels"?: (((Input_lookDefinitions["Reflected_og_cook::cook::source_schemas::procedural::ArtPixels"])) | ((null)));
readonly "ArtShape"?: (((Input_lookDefinitions["Reflected_og_cook::cook::source_schemas::procedural::ArtShape"])) | ((null)));
readonly "ArtTexels"?: (((Input_lookDefinitions["Reflected_og_cook::cook::source_schemas::procedural::ArtTexels"])) | ((null)));
readonly "Environment"?: (((Input_lookDefinitions["Reflected_og_cook::cook::source_schemas::procedural::Environment"])) | ((null)));
readonly "Fog"?: (((Input_lookDefinitions["Reflected_og_cook::cook::source_schemas::procedural::Fog"])) | ((null)));
readonly "MaterialStyle"?: (((Input_lookDefinitions["Reflected_og_cook::cook::source_schemas::procedural::MaterialStyle"])) | ((null)));
readonly "PostEffects"?: (((Input_lookDefinitions["Reflected_og_cook::cook::source_schemas::procedural::PostEffects"])) | ((null)));
readonly "RenderSettings"?: (((Input_lookDefinitions["Reflected_og_cook::cook::source_schemas::procedural::RenderSettings"])) | ((null)));
readonly "Sky"?: (((Input_lookDefinitions["Reflected_og_cook::cook::source_schemas::procedural::Sky"])) | ((null)));
readonly "Stylise"?: (((Input_lookDefinitions["Reflected_og_cook::cook::source_schemas::procedural::Stylise"])) | ((null)));
readonly "antialias"?: (((Input_lookDefinitions["Antialias"])) | ((null)));
readonly "extends"?: ((ReadonlyArray<(Input_lookDefinitions["LookBase"])> | (number extends ((Input_lookDefinitions["LookBase"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "mood"?: (((Input_lookDefinitions["Mood"])) | ((null)));
readonly "name"?: ((string) | (null));
readonly "ui"?: (((Input_lookDefinitions["UiStyle"])) | ((null)));
readonly "uiSeeds"?: (((Input_lookDefinitions["UiSeeds"])) | ((null)));
readonly "variant"?: ((string) | (null));
});
export interface Input_lookDefinitions {
readonly "Antialias": ("tier" | "off" | "msaa4" | "taa") & (string);
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "HexColor": (string);
readonly "LookBase": (((string)) | ((Input_lookDefinitions["ContentRef"])));
readonly "Mood": ("bright" | "dark" | "warm" | "cold" | "soft" | "high-contrast") & (string);
readonly "Reflected_og_cook::cook::source_schemas::procedural::ArtCamera": ({
/** The camera keeps one angle, pitch and yaw: false is no constraint. */
readonly "fixedAngle"?: (boolean);
/** The widest vertical field of view a perspective camera of the style takes; 0 is no limit. */
readonly "fovMax"?: (number);
/** The narrowest vertical field of view a perspective camera of the style takes; 0 is no limit. */
readonly "fovMin"?: (number);
/** With fixedAngle: how far below the horizon the camera looks (0 side-on, 90 straight down, 30 the 2:1 pixel isometric, 35.264 true isometric). */
readonly "pitch"?: (number);
/** A 2D camera moves and zooms by whole logical pixels only (ArtPixels.unit), as Stylise.pixelSnap does the frame's: pixel art never shimmers as it scrolls. */
readonly "pixelPerfect"?: (boolean);
/** The projection: any (no constraint), perspective or orthographic. */
readonly "projection"?: ("any" | "perspective" | "orthographic") & (string);
/** With fixedAngle: its turn about the vertical axis (45 for isometric). */
readonly "yaw"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::ArtLine": ({
/** Round a sprite: the brush the cook draws the line into its cells with: diamond (the zero value), circle or square (docs/art.md §2.3). */
readonly "brush"?: ("diamond" | "circle" | "square") & (string);
/** The line's colour (linear; a look may write an sRGB "#rrggbb"). */
readonly "color"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Round a mesh: the distance at which the line is width wide, nearer wider and farther thinner (as MaterialStyle.outlineDistance); 0 keeps one width. */
readonly "distanceFalloff"?: (number);
/** Where the line lies against the shape: outside (the zero value), inside, or both. */
readonly "place"?: ("outside" | "inside" | "both") & (string);
/** The line's width: in logical pixels round a sprite (1 is one dark pixel round the shape), in pixels at distanceFalloff round a mesh (as MaterialStyle.outlineWidth); 0 is no line. */
readonly "width"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::ArtMotion": ({
/** The most frames one clip or flipbook may hold; 0 is no limit. */
readonly "maxFrames"?: (number);
/** The default squash and stretch of the style's movement: og.tween.squash's impact where a call leaves it out, 0.5 the preset's own (1.25, 0.8), 1 as rubber hose (1.5, 0.6); 0 is no constraint (the preset's own). */
readonly "squash"?: (number);
/** Clips and flipbooks advance this many times a second whatever the tick, holding each pose between (12 is on twos at 24 fps, 8 on threes); 0 is smooth, no constraint. */
readonly "stepFps"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::ArtPalette": ({
/** The colours of colors kept for signals (the player, danger, the goal, a pickup), by their index in the palette (0 its first colour): scenery and backgrounds stay off them. A look gives them on one line (`accents 3 7`); empty keeps none. */
readonly "accents"?: (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** The palette (og.load.palette of a `.hex` or GIMP `.gpl` file; a look names one under assets/ or a built-in `@palettes/<name>`); 0 is none, any colour. */
readonly "colors"?: ({
readonly "$ogRef": (string);
});
/** How an asset mixes the two nearest colours where a tone falls between them: none in flat bands (the zero value), or an ordered or noise pattern as Stylise.dither. */
readonly "dither"?: ("none" | "bayer2" | "bayer4" | "bayer8" | "blueNoise" | "interleavedGradient") & (string);
/** The most distinct colours one asset (a sprite sheet, a texture, a model's colours) may use; 0 is no limit. */
readonly "maxColors"?: (number);
/** The most shades one hue ramps through from its shadow to its highlight (3 or 4 for 8-bit art, 4 to 6 for 16-bit); 0 is no limit. */
readonly "rampSteps"?: (number);
/** Every opaque texel is exactly a colour of colors (indexed colour, as hand-placed pixel art); false lets texels between them (shading, filtering) and holds only the hues to the palette. */
readonly "strict"?: (boolean);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::ArtPixels": ({
/** The rows of the logical resolution the art is drawn for, as Stylise.pixelHeight counts them (180, 224, 240); 0 is no constraint. */
readonly "logicalHeight"?: (number);
/** How pixel art turns: free (no constraint), nearest, cleanEdge, or prerotated in rotationSteps. */
readonly "rotation"?: ("free" | "nearest" | "cleanEdge" | "prerotated") & (string);
/** With rotation prerotated: the rotations baked per full turn (8 is one every 45 degrees); 0 is none. */
readonly "rotationSteps"?: (number);
/** Sprites and tiles stand on whole logical pixels: their positions are multiples of 1/unit metres, so no pixel of the art lies between two of the grid. */
readonly "snap"?: (boolean);
/** The cell sizes, in logical pixels, a sprite, a flipbook frame or a tile may have: each side is one of them (`spriteSizes 16 32` takes 16 x 16, 16 x 32 and 32 x 32). A look gives them on one line; empty is any size. */
readonly "spriteSizes"?: (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Logical pixels per metre (per world unit in 2D): one pixel of every sprite, tile and backdrop covers 1/unit metres, so no two assets show pixels of different sizes (16 with 16-pixel tiles one metre wide); 0 is no constraint. */
readonly "unit"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::ArtShape": ({
/** Hard edges rounded by a chamfer of this radius (clay, toys, diorama pieces); 0 is none. */
readonly "bevel"?: (number);
/** The most triangles one character may have; 0 is no budget. */
readonly "characterFaces"?: (number);
/** With normals flat: neighbouring faces within this angle of their facet's normal shade as one facet, one normal and one baked colour, so a gently curved ground is a few broad planes, not a pattern of triangles. 0 is every triangle its own facet. */
readonly "facetAngle"?: (number);
/** With normals angle: the angle between two faces past which their edge is shaded hard. */
readonly "normalAngle"?: (number);
/** How models are shaded across their faces: any (no constraint), smooth, flat, or angle (split past normalAngle). */
readonly "normals"?: ("any" | "smooth" | "flat" | "angle") & (string);
/** The most triangles one prop may have (50 to 500 for faceted low poly); 0 is no budget. An effect's mesh (`conform effect` in its import settings, as og vfx import writes) has none. */
readonly "propFaces"?: (number);
/** The thinnest feature a model's silhouette keeps at the style's scale: thinner detail does not read and is left out; 0 is no constraint. */
readonly "silhouette"?: (number);
/** Models built of cubes of this edge (voxel worlds); 0 is no voxel grid. */
readonly "voxel"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::ArtTexels": ({
/** Texels per metre on the surfaces of 3D models (a 2 m character at 64 texels per metre takes a 128-texel-tall texture); 0 is no constraint. */
readonly "density"?: (number);
/** How the style's textures are sampled: any (no constraint), nearest or linear. */
readonly "filter"?: ("any" | "nearest" | "linear") & (string);
/** The longest side of any texture of the style, in texels (256 for low-resolution 3D); 0 is no limit. */
readonly "maxSize"?: (number);
/** Their mip maps: any (no constraint), full or none. */
readonly "mips"?: ("any" | "full" | "none") & (string);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::Environment": ({
/** Draw the map behind the scene, as three.js scene.background = the environment (Camera.background, og.Sky and RenderSettings.sky give way); false leaves the background as they set it. */
readonly "background"?: (boolean);
/** Blurs only the background, as three.js scene.backgroundBlurriness: 0 sharp to 1 the coarsest mip. HDR and physical skies use GGX levels; authored LDR skies use their own source-resolution box mips. The light is not changed. */
readonly "backgroundBlurriness"?: (number);
/** Multiplies the map where it is drawn behind the scene, as three.js scene.backgroundIntensity; the light is not changed. */
readonly "backgroundIntensity"?: (number);
/** A second map from og.load.environment that the light and the background blend towards (indoor to outdoor); 0 is none. Both maps turn by rotation and the background is scaled by backgroundIntensity. */
readonly "blendTo"?: ({
readonly "$ogRef": (string);
});
/** How far the blend has gone: 0 the map, 1 blendTo, in between a mix of both maps' light (every GGX level and the harmonics) and background. Drive it from the tick (a tween or a system), never from a clock, so a replay blends alike; it is drawn in 128 steps, each regenerating the light once. */
readonly "blendWeight"?: (number);
/** The ground point under where the map was photographed: the dome's centre is groundHeight above it. */
readonly "groundCenter"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** Draws the sky on a dome with a flat ground under it, as three.js GroundedSkybox: how far above the ground the map was photographed (metres). Objects stand on the map's ground, and the environment's reflections show it too. 0 (the default) draws the sky at infinity. See `render §4`. */
readonly "groundHeight"?: (number);
/** The grounded dome's radius (metres, larger than groundHeight), as GroundedSkybox's radius: the ground looks flat out to about this distance. */
readonly "groundRadius"?: (number);
/** How dark the grounded ground (groundHeight above 0) takes the shadows of what stands on it, the sun's (a DirectionalLight with castShadow) and the map's light they hide, so props sit on it: 1 as dark as a real floor, 0 none (the ground as photographed). */
readonly "groundShadows"?: (number);
/** The hemisphere light's ground colour (linear), as three.js HemisphereLight.groundColor. */
readonly "hemiGround"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** The hemisphere light's intensity, as three.js HemisphereLight.intensity; 0 (the default) is off. */
readonly "hemiIntensity"?: (number);
/** A hemisphere light's sky colour (linear), as three.js HemisphereLight.color: diffuse-only ambient light mix(hemiGround, hemiSky, 0.5 n.y + 0.5) * hemiIntensity, darkened by ambient occlusion like the map's light. Unlike light baked into the map it adds no reflection. */
readonly "hemiSky"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** The strength of the map's light on every surface, as three.js scene.environmentIntensity; while a map is set it replaces RenderSettings.environmentIntensity. */
readonly "intensity"?: (number);
/** The map from og.load.environment; 0 lights by the sky. */
readonly "map"?: ({
readonly "$ogRef": (string);
});
/** Turns the map about +Y, as three.js scene.environmentRotation.y: positive turns what was at +X towards -Z. The light and the background turn together. */
readonly "rotation"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::Fog": ({
/** Which way the fog scatters (Henyey–Greenstein g): 0 evenly, towards 1 forward (bright looking into the light: shafts against the sun), below 0 back towards the light. */
readonly "anisotropy"?: (number);
/** The colour the fog tends to (linear, before exposure). */
readonly "color"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Exponential distance fog per metre past start, with or without end's ramp (0: none). */
readonly "density"?: (number);
/** Draw the fog. */
readonly "enabled"?: (boolean);
/** The distance from the camera where fog is full: with end > start, a linear ramp from start to end applied together with density's exponential fog (0: no ramp). */
readonly "end"?: (number);
/** How fast height fog thins above height, per metre. */
readonly "falloff"?: (number);
/** The world height below which height fog is densest. */
readonly "height"?: (number);
/** Height fog at and below height (0: none). */
readonly "heightDensity"?: (number);
/** The most the global distance and height fog together hide, also with volumetric on. Does not limit independent FogVolume media. */
readonly "maxOpacity"?: (number);
/** How much of the sun's and the lamps' light the fog scatters (its albedo, tinted by color): 0 leaves the plain fog's haze without shafts or glow, 1 sends all of that light on, above 1 brightens the shafts for effect. */
readonly "scattering"?: (number);
/** How much fog covers the sky and clouds (0: clear). Below a cumulus base, or without cumulus, the lower sky stands for distant land and takes land fog. Above a cumulus base it is cloud sea and keeps this share. */
readonly "sky"?: (number);
/** The distance from the camera where distance fog starts, also with volumetric on. Height fog and FogVolume are independent of start. */
readonly "start"?: (number);
/** Plain fog's colour looking into the sun. Ignored by volumetric fog, which scatters the actual lights' colours; anisotropy controls its angular spread. */
readonly "sunColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** How tight plain fog's sun-colour glow is (higher: smaller). Ignored by volumetric fog: use anisotropy for its Henyey-Greenstein angular spread. */
readonly "sunPower"?: (number);
/** Light the fog as a medium: unlit, it tends to color as the plain fog does; over that it (density, thickened below height) scatters the sun through its cascades and the point, spot and rect lights (up to RenderCapabilities.volumetricLights a sample). Blended surfaces and sprites take the plain fog. See `render §7`. */
readonly "volumetric"?: (boolean);
/** How far from the camera the fog shows shafts and the local lights; past it the fog goes on in its colour, with the sun's light in it unshadowed. */
readonly "volumetricDistance"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::MaterialStyle": ({
/** Shade every lit material with face normals, as StandardMaterialDesc.flatShading: visible facets for low-poly, paper-craft and voxel looks. */
readonly "flatShading"?: (boolean);
/** The toon ramp for materials without their own (StandardMaterialDesc.gradientMap, og.load.texture): its first row maps the light's angle to the light received, read without filtering; 0 is three built-in bands. */
readonly "gradientMap"?: ({
readonly "$ogRef": (string);
});
/** The outline colour for materials without their own (linear). */
readonly "outlineColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Distance in metres at which outlines have their base width. Nearer they grow up to outlineNearCap times that width (default 4); farther they thin proportionally. 0 keeps a constant width. Foliage never grows beyond its base width and fades below a pixel. */
readonly "outlineDistance"?: (number);
/** Near-camera width limit as a multiple of the base width, with outlineDistance > 0. Default 4 preserves existing growth; 1 caps near lines at base width while retaining far falloff. Also applies to material and ArtLine outlines. Foliage and orthographic cameras keep their existing rules. */
readonly "outlineNearCap"?: (number);
/** Where outlines lie against an object's silhouette: inside (the zero value, its own edge pixels), outside (round it, at any width, anti-aliased) or both. Creases between objects stay inside. */
readonly "outlinePlace"?: ("inside" | "outside" | "both") & (string);
/** The outline width in pixels at outlineDistance for materials without their own; 0 is none. A hull outline (StandardMaterialDesc.outlineWidth): silhouettes and creases against other objects, never the faces of one. A surface seen at a grazing angle is lined only where it ends. See `looks §12`. */
readonly "outlineWidth"?: (number);
/** Rim (fresnel) light for materials without their own (linear). */
readonly "rimColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** The rim light's strength; 0 is none. */
readonly "rimIntensity"?: (number);
/** How tight the rim is (higher: thinner). */
readonly "rimPower"?: (number);
/** How far shadowed surfaces fall towards black: where the sun does not reach, a surface loses this share of the sun's ramp and of the ambient light (1: solid black shadows). Point, spot and rect lights and the emissive still show inside it; shadowTint's light is added after it. 0 is off. See `looks §12`. */
readonly "shadowDarkness"?: (number);
/** The colour the shadowed side of a toon or lit surface moves towards instead of black (linear): the cool or warm shadow of illustration. */
readonly "shadowTint"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** How far shadowed light moves towards shadowTint; 0 is off. */
readonly "shadowTintAmount"?: (number);
/** A made surface on every lit material (clay, paper, plastic): its normal detail, colour grain and light response, in each mesh's own space; none is no change. Unlit materials, materials with StandardMaterialDesc.styled false and those with a MeshPhysicalMaterial lobe keep their own shading. */
readonly "surface"?: ("none" | "clay" | "paper" | "plastic") & (string);
/** The size of the surface's pattern in metres of the mesh's own space (a clay lump, the paper's fibre); 0 takes the surface's own (clay 0.12, paper 0.15, plastic 0.3). */
readonly "surfaceScale"?: (number);
/** How strongly the surface shows: its bumps, grain and wrapped light; 0 is none. */
readonly "surfaceStrength"?: (number);
/** Cel shading for every lit material, as StandardMaterialDesc.toon: light quantised by the ramp, hard highlights. A material with a physical lobe (clearcoat, transmission and the rest) keeps its physical shading; set its own toon to make it toon anyway. */
readonly "toon"?: (boolean);
/** How smooth a toon material's ambient light (the Environment and its hemisphere light) and reflection stay: 0 one flat colour, the light from above and below averaged (MToon's equalised ambient), so the bands sit on flat colour; 1 the smooth PBR ambient and reflections. See `looks §12`. */
readonly "toonAmbient"?: (number);
/** The toon bands' and highlight's edges, anti-aliased over this many pixels (lilToon's screen-space smoothing); 0 hard steps that only TAA softens. */
readonly "toonSoftness"?: (number);
/** Draw every lit material's feature edges over its faces (StandardMaterialDesc.wireframe's; this style's angle and colour where it sets none), in the colour the surface shows along each edge or wireframeColor; the faces hide the edges behind them. A material's own wireframe draws its edges alone. See `looks §12`. */
readonly "wireframe"?: (boolean);
/** The crease angle wireframe draws past (StandardMaterialDesc.wireframeAngle); 0 every triangle edge. */
readonly "wireframeAngle"?: (number);
/** The look's line colour for wireframe (StandardMaterialDesc.wireframeColor), linear, above 1 blooms; black (the zero value) draws each edge in the colour its surface shows there. */
readonly "wireframeColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::PostEffects": ({
/** Screen-space ambient occlusion: darkens the ambient light in creases and contact areas. */
readonly "ao"?: (boolean);
/** How much the occlusion also darkens direct and local light (0 only the ambient light, 1 a fully occluded pixel keeps 35 % of it). Above 0 the AO sees and darkens water surfaces too; at 0 water is outside it. */
readonly "aoDirect"?: (number);
/** 0 none, 1 full; above 1 exaggerates. */
readonly "aoIntensity"?: (number);
/** The algorithm. */
readonly "aoMethod"?: ("auto" | "gtao" | "ssao" | "screen") & (string);
/** The world-space radius searched for occluders. */
readonly "aoRadius"?: (number);
/** Above 0 the radius grows with the distance, min(aoRadius, aoRadiusScale × distance), a fixed share of the screen up close as three.js SSAO passes size theirs; 0 keeps aoRadius everywhere. See `render §5`. */
readonly "aoRadiusScale"?: (number);
/** Half (the default) searches and blurs at half resolution with a normal- and depth-aware upsample; full searches every pixel. QualityPreset.high sets RenderSettings.aoResolution to full. */
readonly "aoResolution"?: ("full" | "half") & (string);
/** The colour balance of the highlights (luminance above 0.5, fading in from it), as balanceShadows. */
readonly "balanceHighlights"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The colour balance of the mid-tones (what the shadows and highlights leave), as balanceShadows. */
readonly "balanceMidtones"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The colour balance of the shadows (luminance below 0.5, fading out towards it): x shifts from cyan to red, y from magenta to green, z from yellow to blue, each -1 to 1 added to its channel; zero is off. */
readonly "balanceShadows"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** Added to every channel of the display colour; 0 is off. */
readonly "brightness"?: (number);
/** Red and blue split towards the edges, in pixels at the corners at 720p; 0 is off. */
readonly "chromaticAberration"?: (number);
/** Contrast around mid grey: positive spreads the tones by 1 / (1 - contrast), negative pulls them in by 1 + contrast (-1 is flat grey); 0 is off. */
readonly "contrast"?: (number);
/** Custom post passes in order (og.load.post), at most RenderCapabilities.maxCustomPosts (8), each over the one before, after depth of field and motion blur, before exposure and bloom, reading depth and normals (the prepass). A 0 is skipped, as is one failing to compile (og.render.postErrors). See `render §5`. */
readonly "custom"?: (ReadonlyArray<({
readonly "$ogRef": (string);
})> | (number extends (({
readonly "$ogRef": (string);
})) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Bokeh depth of field: pixels away from the focus distance blur into discs. */
readonly "depthOfField"?: (boolean);
/** How quickly blur grows outside the band: full blur focusRange times (1 + falloff) away. */
readonly "dofFalloff"?: (number);
/** The distance from the camera that is sharp. */
readonly "dofFocusDistance"?: (number);
/** The half-width of the sharp band around it. */
readonly "dofFocusRange"?: (number);
/** The largest blur radius, as a fraction of the image height (0.012 is about 9 px at 720p). */
readonly "dofMaxBlur"?: (number);
/** Taps per pixel. */
readonly "dofQuality"?: ("low" | "high") & (string);
/** Tilt-shift: above 0 the sharp band is a strip across the image's middle, this fraction of its height either side of the centre line, at any camera distance; blur grows above it as far and below it as near, full at dofTiltShift times (1 + dofFalloff) past it. 0 focuses by depth. See `looks §19`. */
readonly "dofTiltShift"?: (number);
/** Luminance-weighted noise amplitude in display units, a new pattern every tick (0.04 is subtle); 0 is off. */
readonly "filmGrain"?: (number);
/** The grain size in pixels. */
readonly "filmGrainSize"?: (number);
/** The flash's linear display colour. */
readonly "flashColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Order-independent 2D glow: sprites with SpriteDesc.glow (and 2D particles with a glow) are blurred and added back, with or without bloom. */
readonly "glow"?: (boolean);
/** How much of the blurred glow is added back. */
readonly "glowIntensity"?: (number);
/** Screen-space light shafts from the DirectionalLight (the sun must be in front of the camera). The sun casts them with its own light, as a source of a fixed angular size whatever size its disc is drawn, dimmed by the clouds over it; the bright sky around it adds its own. */
readonly "godRays"?: (boolean);
/** How fast the rays fade along their length. */
readonly "godRaysDecay"?: (number);
/** How far towards the sun each ray samples (a fraction of the distance). */
readonly "godRaysDensity"?: (number);
/** How bright the shafts are. */
readonly "godRaysIntensity"?: (number);
/** Taps per ray. */
readonly "godRaysSamples"?: (number);
/** Brightness (linear, before exposure) below which the sky and the sun carry no rays: only what is brighter casts shafts. */
readonly "godRaysThreshold"?: (number);
/** A linear tint on the light the rays carry. */
readonly "godRaysTint"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** How far each colour goes towards its luminance (grey); 0 is off. */
readonly "grayscale"?: (number);
/** Turns every hue about the grey axis (red towards green at 120); 0 is off. */
readonly "hue"?: (number);
/** How far each colour goes towards its complement; 0 is off. */
readonly "invert"?: (number);
/** Barrel (positive: lines bow out) or pincushion (negative: in) distortion; 0 is off. Radius in pixels at any aspect; a barrel keeps its farthest corner on the image. After the depth-reading Stylise passes, before palette quantisation; the UI is not bent, pointer raycasts see the undistorted frame. See `render §5`. */
readonly "lensDistortion"?: (number);
/** The lens's centre, in fractions of the frame from the top left. */
readonly "lensDistortionCenter"?: (ReadonlyArray<(number)> & { readonly length: 2 });
/** Colour grading by a lookup table from og.load.lut; 0 is none (and so is a LUT whose streaming group is not resident). */
readonly "lut"?: ({
readonly "$ogRef": (string);
});
/** 0 leaves the image as tone mapped, 1 applies the table fully. */
readonly "lutIntensity"?: (number);
/** Per-object and camera motion blur along the screen-space velocity. */
readonly "motionBlur"?: (boolean);
/** The longest blur in pixels. */
readonly "motionBlurMaxPixels"?: (number);
/** Taps along the velocity. */
readonly "motionBlurSamples"?: (number);
/** The shutter as a fraction of the frame (0.5 is a 180 degree shutter). */
readonly "motionBlurShutter"?: (number);
/** Outlines every opaque surface from depth and normal edges, creases between the faces of one object included (materials with outlineWidth get their own hull outline instead). */
readonly "outline"?: (boolean);
/** The line colour, linear. */
readonly "outlineColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** The relative depth step that counts as a silhouette (0.04 is 4 %). */
readonly "outlineDepthThreshold"?: (number);
/** The normal change that counts as a crease (1 - cos of the angle; 0.4 is about 53 degrees). */
readonly "outlineNormalThreshold"?: (number);
/** Line width in pixels, drawn inside the silhouette. */
readonly "outlineWidth"?: (number);
/** The colour balance keeps each pixel's luminance, so it moves only the hue (Photoshop's Preserve Luminosity). */
readonly "preserveLuminosity"?: (boolean);
/** -1 grey, 0 unchanged, 1 twice the colour (the chroma around the luminance scaled by 1 + saturation). */
readonly "saturation"?: (number);
/** A zoom, spin or directional blur of the whole frame (RedGPU's zoom, radial and directional blurs), weighted to the middle of its reach: one pass after motion blur and before bloom and the tone curve, so highlights streak and glow; the UI is not blurred. Off at none or at screenBlurStrength 0. */
readonly "screenBlur"?: ("none" | "zoom" | "spin" | "directional") & (string);
/** Directional's direction, counter-clockwise from the right (90 is vertical). */
readonly "screenBlurAngle"?: (number);
/** Zoom's and spin's centre, in fractions of the frame from the top left. */
readonly "screenBlurCenter"?: (ReadonlyArray<(number)> & { readonly length: 2 });
/** Taps per pixel for spin and directional. Zoom exceeds this quality floor with three eight-tap stages (512 effective positions), or direct adaptive sampling when strength >= 1 crosses the centre. Both use stable per-pixel jitter. */
readonly "screenBlurSamples"?: (number);
/** How far it reaches each way: for zoom the fraction of the way to the centre, for spin radians, for directional pixels at 720 lines; 0 is off. */
readonly "screenBlurStrength"?: (number);
/** How fast the shake moves. */
readonly "shakeFrequency"?: (number);
/** The largest offset at trauma 1, as a fraction of the view height. */
readonly "shakeMaxOffset"?: (number);
/** The largest roll at trauma 1. */
readonly "shakeMaxRoll"?: (number);
/** The shake's noise seed. */
readonly "shakeSeed"?: (number);
/** Contrast-adaptive sharpening (CAS) of the final image, as a three.js CAS pass after the output pass: each pixel pushed away from its four neighbours by a weight that falls where the neighbourhood is already near black or white. 0 is the mildest, 1 the strongest; below 0 is off. */
readonly "sharpen"?: (number);
/** Stochastic, denoised screen-space reflections (after FidelityFX SSSR and Frostbite): a GGX-lobe ray a pixel up to ssrMaxRoughness through a depth pyramid, reflecting the previous frame's lit image; on a miss, the probes', probe GI's and environment's. Opaque, perspective 3D; turns on the prepass. See `render §5`. */
readonly "ssr"?: (boolean);
/** The fraction of the screen's edge over which a reflection fades into the probes' (the last fifth of ssrMaxDistance fades too). */
readonly "ssrFade"?: (number);
/** How much of the reflection replaces the probes' and the environment's where it resolved; 1 is physical (weighted by the surface's Fresnel and lobe as the probes' are). */
readonly "ssrIntensity"?: (number);
/** How far a reflected ray marches, in world units; the last fifth fades into the probes'. The march's cost is set by ssrSteps, not by the distance. */
readonly "ssrMaxDistance"?: (number);
/** The roughest surface traced: rougher ones keep the probes' reflection, and the last quarter below it fades into them. See `render §5`. */
readonly "ssrMaxRoughness"?: (number);
/** Half (the default): one ray per 2x2 block, a different pixel of the block each frame, denoised on the blocks and upsampled to the pixels by their depth, normal and roughness. Full: one ray per pixel, denoised at full resolution. See `render §5`. */
readonly "ssrResolution"?: ("full" | "half") & (string);
/** The hierarchical march's iterations along each ray, each crossing a cell of the min-max depth pyramid's current level; a ray out of steps keeps the probes' reflection. Only the rays that need the steps pay for them. See `render §5`. */
readonly "ssrSteps"?: (number);
/** How far behind the depth buffer a surface is taken to extend: a ray passing behind a surface by less than this hits it. */
readonly "ssrThickness"?: (number);
/** White balance: towards warm orange (positive) or cool blue (negative), Unity's Temperature divided by 100 (a von Kries scaling in LMS from a white point moved along the daylight locus); 0 is off. See `render §5`. */
readonly "temperature"?: (number);
/** Above 0, each pixel white where its luminance reaches this level and black below it (the table's 32 steps soften the edge over about 1/31 of the range); 0 is off. */
readonly "threshold"?: (number);
/** White balance: towards magenta (positive) or green (negative), as Lightroom's and Unity's Tint divided by 100; 0 is off. */
readonly "tint"?: (number);
/** Saturation that spares colours already saturated and skin tones: positive livens dull colours, negative mutes them; 0 is off. */
readonly "vibrance"?: (number);
/** Darkened (or tinted) edges of the frame. */
readonly "vignette"?: (boolean);
/** The linear colour the edges move towards. */
readonly "vignetteColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** 0 none, 1 black corners. */
readonly "vignetteIntensity"?: (number);
/** Where darkening starts, as a fraction of the half-diagonal. */
readonly "vignetteRadius"?: (number);
/** The width of the fade. */
readonly "vignetteSmoothness"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::RenderSettings": ({
/** How fast auto exposure falls when the view gets brighter, in EV per second of game time; 0 is instant. See `render §5`. */
readonly "adaptSpeedDown"?: (number);
/** How fast auto exposure rises when the view gets darker, in EV per second of game time: each drawn frame moves it by at most the speed times the ticks since the last drawn frame, never by the wall clock; 0 is instant. Snapshots and probe sweeps carry the adapted exposure. */
readonly "adaptSpeedUp"?: (number);
/** PostEffects.ao darkens creases; false draws none. PostEffects keeps its settings either way. */
readonly "ambientOcclusion"?: (boolean);
/** The resolution ambient occlusion is searched at: postEffects (the default) draws PostEffects.aoResolution; full or half is drawn in its place, and PostEffects keeps its own for when this is postEffects again. See `render §5`. */
readonly "aoResolution"?: ("postEffects" | "full" | "half") & (string);
/** Meter every frame to a key that follows its brightness (middle grey for a daylit view), then apply Camera.exposure as compensation. With both adapt speeds 0 the exposure is the frame's own; otherwise it adapts from the exposure metered on the first frame after the first tick, so hosts draw alike. See `render §5`. */
readonly "autoExposure"?: (boolean);
/** The most exposure auto exposure applies, in stops (EV, compensation included): a dark view is brightened no further, so a night stays as dark as its light makes it. Below autoExposureMin it is taken as autoExposureMin. See `render §5`. */
readonly "autoExposureMax"?: (number);
/** The least exposure auto exposure applies, in stops (EV: the image times 2^EV, Camera.exposure's compensation included): a very bright view is darkened no further. See `render §5`. */
readonly "autoExposureMin"?: (number);
/** Draw the baked lighting of the scenes that have it (og bake lightmaps): static models with their lightmaps, everything else inside a LightVolume with its probes. False draws every scene as if unbaked, its ambient light from the sky or the environment map. See `render §4`. */
readonly "bakedLighting"?: (boolean);
/** Light above the threshold bleeds into its surroundings. */
readonly "bloom"?: (boolean);
/** With bloomRadius 0 or more: the high pass (and the blur) sees each channel as 0.18 x (colour / 0.18) to this power before exposure, as a contrast grade run ahead of UnrealBloomPass (the way a three.js grade pass before the bloom does) would leave it. 1, the default, leaves the colour as lit. */
readonly "bloomContrast"?: (number);
/** How much of the blurred light is added back. */
readonly "bloomIntensity"?: (number);
/** Softens the threshold over threshold plus or minus knee; 0 is a hard cut. */
readonly "bloomKnee"?: (number);
/** Below 0 (the default) keeps Origin's bloom. 0 to 1 is three.js UnrealBloomPass at this radius: a half-resolution high pass from bloomThreshold to bloomThreshold + bloomKnee, five Gaussian mips mixed by mix(f, 1.2 - f, radius), added at bloomIntensity as EffectComposer's additive copy. See `render §5`. */
readonly "bloomRadius"?: (number);
/** Brightness after exposure where bloom starts; 1 is display white. */
readonly "bloomThreshold"?: (number);
/** The strength of the image-based ambient light from the sky on surfaces; og.Environment.intensity takes its place while an environment map is set. The sky and its clouds are drawn as they are whatever it is. */
readonly "environmentIntensity"?: (number);
/** A multiplier on every camera's Camera.exposure, as three.js renderer.toneMappingExposure is one for the whole frame: a look's brightness without touching the cameras; 1 leaves them as they are. */
readonly "exposure"?: (number);
/** Draw what moves between the last two ticks: a frame between ticks blends entities, og.Camera and skinned joints from the tick before the last to the last, showing the world up to one tick late. Frames on a tick boundary, screenshots and captures draw the last tick. Presentation only. See `render §9`. */
readonly "interpolate"?: (boolean);
/** A multiplier on every model's projected level-of-detail error (og.Model.lodBias multiplies it again): a level is drawn while its error, measured by the cook against the full mesh, covers under a pixel. 1 is the engine's choice, 2 keeps full meshes twice as far; 0 draws every model in full. See `render §10`. */
readonly "lodBias"?: (number);
/** At most this many composed effects (og.Vfx) drawn at once: past it the oldest one-shot effects are left out, then the oldest looping ones, and the probe's dump_render counts them (vfx.dropped). 64; the low preset 32, high 128. Drawing only: their events, shakes and sounds still come. */
readonly "maxVfx"?: (number);
/** At most this many particle slots (the emitters' maxParticles, summed) among the og.Vfx drawn: past it the oldest one-shot effects are left out, then the oldest looping ones, as with maxVfx. 65536; the low preset 32768, high 131072. */
readonly "maxVfxParticles"?: (number);
/** Samples per pixel where the antialiasing multisamples (msaa4): 4, or 1, which draws FXAA instead. RenderCapabilities.maxMsaaSamples caps it: 1 where the device resolved no MSAA, which draws one sample whatever this says. Changing it remakes the multisampled targets and pipelines next frame. See `render §2`. */
readonly "msaaSamples"?: (number);
/** Every particle system's emission times this (0 to 1); an emitter left with fewer than 8 particles alive at once keeps more, so a flash or a ring stays whole. The low preset draws 0.5. Drawing only: particle events and collisions are the full emitter's. */
readonly "particleEmission"?: (number);
/** The graphics preset og.render.applyQuality last wrote into the quality fields below (rectShadows to upscaler), or none. A record for the settings screen and the probe: the renderer reads the fields, never this, so a field the game sets after the preset is drawn as set. */
readonly "quality"?: ("none" | "low" | "medium" | "high") & (string);
/** Rect lights whose castShadow is on cast their soft shadows; false draws every rect light unshadowed, each keeping its castShadow for when this is on again. The low preset turns it off. See `render §3`. */
readonly "rectShadows"?: (boolean);
/** PostEffects.ssr draws; false leaves smooth surfaces the reflection probes' and the environment's reflection. PostEffects keeps its settings either way. */
readonly "screenSpaceReflections"?: (boolean);
/** The directional light's shadow cascades, 1 to 4: more cover the same shadowDistance with finer shadows near the camera, at one more shadow pass and one more map each. The high preset draws 4, medium 3, low 2. Changing it makes the cascade maps again at the next frame. */
readonly "shadowCascades"?: (number);
/** The shadow maps at this fraction of their full side: the cascades (2048 texels by default) and local lights' layers (512), each within 64 texels and the device's limit (4096 for cascades). Changing it remakes the maps next frame. */
readonly "shadowMapScale"?: (number);
/** At most this many point, spot and rect lights cast shadows in a frame, the brightest at the camera kept (RenderCapabilities.maxShadowedLights caps it); 0 casts none. The high preset keeps 16, medium 8, low 2. The directional light's cascades are not counted. */
readonly "shadowedLights"?: (number);
/** Draw the procedural sky behind the scene instead of Camera.background (with an og.Environment map set, the map). */
readonly "sky"?: (boolean);
/** Spread live sky capture and filtering over 13 rendered frames instead of completing each frame. Low sets true; medium, high and the default keep false so terrain ambient and reflections follow the current sky. Slicing lowers peak GPU cost but retains older lighting during fast sky changes. */
readonly "skyCaptureSliced"?: (boolean);
/** An entity or og.Camera that moved further than this between the last two ticks is drawn at the last tick, not blended: a teleport or a respawn this far needs no og.render.snap. 0 never snaps by distance. */
readonly "snapDistance"?: (number);
/** Particles with a softDistance fade against the scene behind them; false draws them hard-edged and asks no depth prepass for them. */
readonly "softParticles"?: (boolean);
/** PostEffects.ssrSteps times this (within the 4 to 128 steps the pass takes): 2 is 128 steps for the default 64, finer contact with thin and distant reflected surfaces at a higher GPU time. PostEffects keeps its steps. */
readonly "ssrStepScale"?: (number);
/** How far from itself a stationary or static light (LightMobility) has its diffuse light shaded per pixel; past it, that light comes from a grid of probes round the camera and such a rect light skips rough surfaces. Glossy surfaces keep the highlights. 0 shades every light whole. See `render §3`. */
readonly "staticLightReach"?: (number);
/** Tone mapping by toneMapping; false clamps, so 2D colours come out exactly as authored at exposure 1. */
readonly "toneMapped"?: (boolean);
/** Which curve tone maps the frame while toneMapped is true: aces (the default), agx, neutral or none. */
readonly "toneMapping"?: ("aces" | "agx" | "none" | "neutral") & (string);
/** Under TAA, the temporal resolve from the render scale (DisplaySettings.renderScale) to the output: auto (the default and medium: fsr3 below scale 1, taau at it), taau (low) or fsr3 (high). A device lacking fsr3 (RenderCapabilities.upscalers) draws taau. Switching restarts the history. See `render §1`. */
readonly "upscaler"?: ("taau" | "fsr3" | "auto") & (string);
/** A volumetric og.Fog (Fog.volumetric) is drawn lit, with its light shafts, and so is the sun's light under an og.Water (its shafts, where it has caustics); false draws the fog as plain distance and height fog and the water's light even. */
readonly "volumetricFog"?: (boolean);
/** Fog.volumetric's samples along each view ray times this: the froxel grid's 64 slices deep (8 to 256), or the raymarch's 24 steps per pixel (4 to 96) where the renderer draws one (RenderCapabilities.volumetricFog). Changing it makes the grid again at the next frame. See `render §7`. */
readonly "volumetricStepScale"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::Sky": ({
/** A multiplier on the whole sky. */
readonly "brightness"?: (number);
/** Draw and light by this sky. */
readonly "enabled"?: (boolean);
/** The glow around the sun. */
readonly "glow"?: (number);
/** How tight the glow is. */
readonly "glowPower"?: (number);
/** Below the horizon. */
readonly "ground"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** At the horizon. */
readonly "horizon"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** The moon's colour. */
readonly "moonColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Towards the moon; zero puts it opposite sunDirection. */
readonly "moonDirection"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The lit moon's brightness. */
readonly "moonIntensity"?: (number);
/** How much of the moon is lit: 0 new, 0.25 the first quarter (its right half), 0.5 full, 0.75 the last quarter (its left half). */
readonly "moonPhase"?: (number);
/** The moon disc's angular radius (0: no moon). It sets below the horizon and hides the stars behind it. */
readonly "moonSize"?: (number);
/** Stars where the sky is dark: 0 none, 1 a full night sky. */
readonly "stars"?: (number);
/** The sun's colour. */
readonly "sunColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Towards the sun: the opposite of the way its light shines. Zero (the default) follows the frame's light: towards the DirectionalLight drawn (its entity's +Z; it shines down -Z; the brightest of several), or the built-in sun's (0.45, 0.8, 0.35) with none. A non-zero direction is used as given. */
readonly "sunDirection"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The disc's brightness. */
readonly "sunIntensity"?: (number);
/** The sun disc's angular radius (0: no disc). */
readonly "sunSize"?: (number);
/** Overhead. */
readonly "zenith"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::Stylise": ({
/** How far texture coordinates are interpolated without perspective correction: 1 fully affine, so textures warp and swim across large polygons as on early consoles; 0 is off. */
readonly "affineTexturing"?: (number);
/** Colour bands by depth (2 to 8), interpolating depthBandsNearColor to depthBandsFarColor in lightness, chroma and hue. By default the layers divide depthBandsNear..depthBandsFar evenly; depthBandsFit instead fits logarithmic layers to visible geometry. 0 is off. See `looks §11`. */
readonly "depthBands"?: (number);
/** Where the last band ends; the far colour starts one band's depth before it and continues beyond it. Ignored when depthBandsFit is true. */
readonly "depthBandsFar"?: (number);
/** The farthest band's colour (linear, before exposure). */
readonly "depthBandsFarColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Fit logarithmic bands to the full visible non-sky depth range, measured each drawn frame. Nearer layers span less distance than farther ones. Ignores depthBandsNear and depthBandsFar; false keeps their absolute linear layers. The farthest layer still merges geometry within it. */
readonly "depthBandsFit"?: (boolean);
/** How far a band replaces the surfaces' own shading: 1 flat silhouettes in the band's colour; 0 a tint over the lit scene, its colours times the band's at unit luminance (a coloured filter: the shading, the value and the surfaces' own hues show through); between, a mix of the two. */
readonly "depthBandsFlatten"?: (number);
/** Where the first band starts, unless depthBandsFit is true. */
readonly "depthBandsNear"?: (number);
/** The nearest band's colour (linear, before exposure). */
readonly "depthBandsNearColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Quantise depth to this many bits, for the coarse depth buffers of early 3D (12 to 16 are period-like: distant surfaces fight); 0 keeps full precision. */
readonly "depthBits"?: (number);
/** How the steps of posterize and palette are mixed: none rounds every pixel, an ordered or blue-noise pattern alternates the two nearest steps in proportion. */
readonly "dither"?: ("none" | "bayer2" | "bayer4" | "bayer8" | "blueNoise" | "interleavedGradient") & (string);
/** The size of one dither cell in logical pixels. */
readonly "ditherScale"?: (number);
/** Where the dither's cells lie: screen, a grid over the frame (it crawls as the camera moves); surface, cells on the surfaces at least ditherScale pixels on the screen each way, the sky's on a cube round the camera; object, in each mesh instance's own space. Off screen the frame runs the prepass. See `looks §8`. */
readonly "ditherSpace"?: ("screen" | "surface" | "object") & (string);
/** The dither's amplitude in steps: 1 spreads exactly one step, less keeps flatter bands. */
readonly "ditherStrength"?: (number);
/** Halftone print: tones become dots on a rotated grid, larger where the image is darker; 0 is off, 1 dots only. */
readonly "halftone"?: (number);
/** The dot grid's angle. */
readonly "halftoneAngle"?: (number);
/** The ink of the dots (linear). */
readonly "halftoneColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** With halftoneMode cmyk: how far the cyan, magenta and yellow plates sit off the black one, in logical pixels, as a cheap print's colour slips past its line art; 0 is in register. */
readonly "halftoneMisregistration"?: (number);
/** How the halftone inks: one ink (halftoneColor) or four-colour process print (cmyk). */
readonly "halftoneMode"?: ("ink" | "cmyk") & (string);
/** Which tones the halftone prints on: all, the shade only or the highlights only (the darkest or lightest third of the tones). Under a region the dots are Ben-Day dots, a tint over the region's own colour at most 0.4 of a cell wide. They fade to flat ink on surfaces turned away from the camera. See `looks §9`. */
readonly "halftoneRegion"?: ("all" | "shade" | "highlight") & (string);
/** The dot grid's pitch in logical pixels. */
readonly "halftoneSize"?: (number);
/** Where the dots lie: screen, a grid over the frame; surface, grids on the surfaces that stay on them as the camera moves and wrap round forms (halftoneSize is then about the pitch in pixels where they lie); object, the same grids in each instance's own space, so a moving object carries its dots. */
readonly "halftoneSpace"?: ("screen" | "surface" | "object") & (string);
/** Pencil or ink cross-hatching from a tonal art map: darker tones add strokes in more directions over a paper-light ground, each set thinning and fading in from clean paper at its tone; 0 is off, 1 strokes only. */
readonly "hatching"?: (number);
/** The direction of the first stroke set. */
readonly "hatchingAngle"?: (number);
/** The stroke colour (linear): graphite, sepia or ink. */
readonly "hatchingColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Hatching from depth as well as luminance: the depth-normal prepass's silhouettes and creases (inkDepthThreshold's and inkNormalThreshold's edges) darken the tone the strokes follow at this strength, so strokes gather along contours and folds. 0 is the tone alone; above 0 the frame runs the prepass. See `looks §9`. */
readonly "hatchingDepth"?: (number);
/** Aerial perspective: view depth in metres at which stroke width and ink strength halve, thinning exponentially into bare paper. 0 is off. Above 0 runs the prepass in every pattern space and leaves the sky bare. The wash and contour weighting stay unchanged. See `looks §9`. */
readonly "hatchingDistance"?: (number);
/** Colour hatching: the ground is a light wash of its own colour and each stroke is that colour darkened towards hatchingColor, so hue and chroma stay under the strokes; false lays strokes of hatchingColor over a paper-light ground. See `looks §9`. */
readonly "hatchingKeepColor"?: (boolean);
/** How many tones the hatching distinguishes, each adding one stroke direction or density (1 to 6). */
readonly "hatchingLevels"?: (number);
/** The paper the strokes lie on (linear): one-ink hatching lifts the ground's light tones towards it, and under hatchingKeepColor the wash lies on the paper's colour at the wash's own lightness (a cream sketchbook, a grey card, a blue print); white keeps the frame's own. */
readonly "hatchingPaper"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** The spacing of hatch lines in logical pixels. */
readonly "hatchingScale"?: (number);
/** Where the strokes lie: screen, stripes over the frame that follow the tone alone; surface, strokes on the surfaces that follow the tone and the form and stay on them as the camera moves (hatchingScale is then about the spacing in pixels where they lie); object, the same in each instance's own space. */
readonly "hatchingSpace"?: ("screen" | "surface" | "object") & (string);
/** Ink lines where the tone changes sharply (luminance edges), over every surface, sprite and texture alike: comic and storybook contours without PostEffects.outline's depth prepass. inkDepth adds the prepass's silhouettes and creases. */
readonly "ink"?: (boolean);
/** The line colour (linear). */
readonly "inkColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** With ink: lines also where the depth-normal prepass breaks (silhouettes against what is behind, creases between faces), at this strength: comic contours round flat colours that luminance alone misses. 0 is luminance only; above 0 the frame runs the prepass PostEffects.outline uses. */
readonly "inkDepth"?: (number);
/** With inkDepth: the relative depth step to a farther neighbour that counts as a silhouette (0.04 is 4 %), as PostEffects.outlineDepthThreshold: lower inks smaller steps. */
readonly "inkDepthThreshold"?: (number);
/** With inkDepth: the normal change between neighbours of about the same depth that counts as a crease (1 - cos of the angle; 0.4 is about 53 degrees), as PostEffects.outlineNormalThreshold: lower inks gentler folds. */
readonly "inkNormalThreshold"?: (number);
/** The luminance step that counts as an edge (0 to 1 in display units): lower draws more lines. */
readonly "inkThreshold"?: (number);
/** The line width in logical pixels. */
readonly "inkWidth"?: (number);
/** How far the strokes follow the image's structure: 0 round brush cells, 0.5 the filter's classic ellipses, 1 thin strokes along every coherent contour, which keep a thin line's shape. See `looks §10`. */
readonly "paintAnisotropy"?: (number);
/** Colour bleeding across edges into its neighbours, wet in wet; 0 is none. */
readonly "paintBleed"?: (number);
/** A tileable paper or canvas texture (og.load.texture) laid over the frame by paintCanvasStrength; 0 is none. */
readonly "paintCanvas"?: ({
readonly "$ogRef": (string);
});
/** Canvas texels per logical pixel, with paintCanvasSpace screen (and on the sky). */
readonly "paintCanvasScale"?: (number);
/** Where the canvas lies: screen, over the frame (the scene slides under the paper as the camera moves); surface, on the surfaces, so the grain sticks to the scene, foreshortened with the floor and walls; object, in each instance's own space, so a moving object carries its grain. */
readonly "paintCanvasSpace"?: ("screen" | "surface" | "object") & (string);
/** How much the canvas texture modulates the frame: 0 none, 1 its full grain. */
readonly "paintCanvasStrength"?: (number);
/** Canvas texels per metre with paintCanvasSpace surface: 128 repeats a 256-texel canvas every two metres. */
readonly "paintCanvasWorldScale"?: (number);
/** The detail of a skinned mesh's pixels (a character's face and hands) where its material sets less: 0 painted as the rest, 1 kept as drawn. */
readonly "paintCharacterDetail"?: (number);
/** How far detail keeps its shape under paintRadius: a pixel's radius shrinks by its detail (its material's StandardMaterialDesc.paintDetail, or paintCharacterDetail on a skinned mesh) times this, and no stroke crosses an object's edge. Above 0 the frame runs the prepass; 0 paints every pixel alike. See `looks §10`. */
readonly "paintDetail"?: (number);
/** Pigment pooling at the edges of colour regions, as wet watercolour dries darker at its rim; 0 is none. */
readonly "paintEdgeDarkening"?: (number);
/** Painterly smoothing (an anisotropic Kuwahara filter): flat regions and incoherent texture become strokes of one colour this many logical pixels across; edges and thin lines keep their shape; 0 is off. Under a 2D project's art contract the cook paints each texture and the frame draws neither pass. See `looks §10`. */
readonly "paintRadius"?: (number);
/** Brush strokes in the paint, sized by paintRadius: marks along each stroke (the image's structure, or a slowly turning lay over flat regions), their own light and dark, and ragged edges where strokes meet. 0 is none; 1 bold, visible at a glance. See `looks §10`. */
readonly "paintStrokes"?: (number);
/** Quantise every pixel to the nearest colour of this palette (og.load.palette), measured in a perceptual space after posterize; 0 is none. */
readonly "palette"?: ({
readonly "$ogRef": (string);
});
/** The logical resolution: the scene is rendered this many pixels tall and scaled up to the output by a whole number (pixelUpscale) with nearest filtering; 0 renders at the output's resolution. The width follows the output's aspect, at most the output's width over the scale. See `looks §7`. */
readonly "pixelHeight"?: (number);
/** Snap the camera and world sprites to the logical pixel grid, so a moving orthographic camera scrolls by whole pixels and sprites in a 3D world keep their edges (pixel sprites in 3D sets). */
readonly "pixelSnap"?: (boolean);
/** The whole-number scale from the logical to the output resolution, a hard request: honoured whenever pixelHeight times it fits the output's height, else the largest that fits; 0 takes the largest that fits. A frame that does not fill the output is centred with Camera.background round it. See `looks §7`. */
readonly "pixelUpscale"?: (number);
/** Tones per colour channel after tone mapping (2 to 64): smooth gradients become flat steps; 0 is off. */
readonly "posterize"?: (number);
/** How posterize steps a colour: each channel apart (channel), or its value and its tint apart so light never changes its hue (tone). */
readonly "posterizeMode"?: ("channel" | "tone") & (string);
/** Snap every vertex of the 3D scene to a screen grid this many logical pixels wide, as fixed-point hardware did: geometry jitters by whole steps as it moves; 0 is off. */
readonly "vertexSnap"?: (number);
/** The wash thinned with water: the pigment over watercolorPaper is this much less dense, so darks lift towards the paper, hues keep and the paper stays the paper. About 0.3 to 0.45 turns a lit scene's heavy darks into pale washes; 0 leaves the frame's tones as lit. See `looks §10`. */
readonly "watercolorDilution"?: (number);
/** A dry brush skipping over the paper's tooth: its peaks (the relief past half its spread, in the paper's own clusters) left bare as the paper (watercolorPaper) in the rim at a wash's outer boundary (watercolorEdgeDarkening's) and on the light-catching tones, the brightest washes first; 0 is none. */
readonly "watercolorDryBrush"?: (number);
/** The wash's dark rim, drawn after paintRadius's strokes: pigment pooled two to three pixels wide at a wash's outer boundary (a colour step wider than a line, or a silhouette against something farther), the colour there raised to one plus the strength. Seams and creases get none; 0 is none. See `looks §10`. */
readonly "watercolorEdgeDarkening"?: (number);
/** Pigment settling into the paper's grain: a density from the paper's relief (paintCanvas, else a noise) thins each colour towards watercolorPaper on the peaks and thickens it in the pits, weighed by the pigment load so the paper and pale washes stay clean; the wash keeps its mean value. 0 is none. See `looks §10`. */
readonly "watercolorGranulation"?: (number);
/** The paper's colour (linear): what the wash thins towards and the dry brush leaves bare. */
readonly "watercolorPaper"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Pigment drifting as the wash dries: the same density from a slow noise about a hundred canvas texels across, laid where the canvas lies (paintCanvasSpace) and seeded by the object under the pixel so no blotch spans two washes, weighed by the pigment load; 0 is none. See `looks §10`. */
readonly "watercolorTurbulence"?: (number);
/** The painted frame displaced by the paper's relief, up to this many logical pixels, so every edge wavers with the tooth as a wash's does (MNPR's substrate distortion); 0 is none. */
readonly "watercolorWobble"?: (number);
});
readonly "UiSeeds": ({
readonly "accent"?: (((Input_lookDefinitions["HexColor"])) | ((null)));
readonly "danger"?: (((Input_lookDefinitions["HexColor"])) | ((null)));
readonly "info"?: (((Input_lookDefinitions["HexColor"])) | ((null)));
readonly "neutral"?: (((Input_lookDefinitions["HexColor"])) | ((null)));
readonly "primary"?: (((Input_lookDefinitions["HexColor"])) | ((null)));
readonly "success"?: (((Input_lookDefinitions["HexColor"])) | ((null)));
readonly "team"?: (((Input_lookDefinitions["HexColor"])) | ((null)));
readonly "warn"?: (((Input_lookDefinitions["HexColor"])) | ((null)));
});
readonly "UiStyle": (((string)) | ((Input_lookDefinitions["ContentRef"])));
}
export type Input_materials = ({
/** Order is stable; duplicate names and generation requirements are checked
together by the material-library validator. */
readonly "materials": (ReadonlyArray<(Input_materialsDefinitions["LibraryMaterial"])> | (number extends ((Input_materialsDefinitions["LibraryMaterial"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
export interface Input_materialsDefinitions {
readonly "Generate": ({
/** Authored image used to prepare the explicitly named map destinations. */
readonly "source": (string);
});
readonly "LibraryMaterial": ({
readonly "brief"?: ((string) | (null));
readonly "facade"?: ((boolean) | (null));
/** Explicit image-map preparation input, not a TS executor or runtime handle. */
readonly "generate"?: (((Input_materialsDefinitions["Generate"])) | ((null)));
/** Material labels produced by grammar surfaces. */
readonly "labels": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "name": (string);
readonly "occlusion"?: (((Input_materialsDefinitions["Occlusion"])) | ((null)));
readonly "relief"?: ((number) | (null));
readonly "roughnessRange"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** glTF material fields shared with authored models and fracture surfaces. */
readonly "surface": (Input_materialsDefinitions["Material"]);
/** Physical tile dimensions. Both dimensions are strictly positive. */
readonly "tile"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "vary"?: (((Input_materialsDefinitions["Variation"])) | ((null)));
readonly "weight"?: ((number) | (null));
});
/** Omitted fields remain absent until the material domain normalizes them. */
readonly "Material": ({
/** Alpha-mask cutoff takes precedence over opacity blending. */
readonly "alphaTest"?: ((number) | (null));
/** Ambient occlusion in red. */
readonly "aoMap"?: ((string) | (null));
readonly "autumnColor"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "autumnColor2"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "clearcoat"?: ((number) | (null));
readonly "clearcoatRoughness"?: ((number) | (null));
/** Linear base RGB (glTF factor); not a runtime colour handle. */
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "domeNormals"?: ((number) | (null));
readonly "doubleSided"?: ((boolean) | (null));
/** Linear emitted RGB, multiplied by emissiveIntensity. */
readonly "emissive"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "emissiveIntensity"?: ((number) | (null));
readonly "grime"?: ((number) | (null));
readonly "grimeColor"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "grimeHeight"?: ((number) | (null));
readonly "ior"?: ((number) | (null));
/** Source image inputs; not cooked texture references. Resolution and path
containment are checked at the input boundary. */
readonly "map"?: ((string) | (null));
readonly "metalness"?: ((number) | (null));
/** Roughness in green, metalness in blue. */
readonly "metalnessMap"?: ((string) | (null));
readonly "normalMap"?: ((string) | (null));
readonly "normalScale"?: ((number) | (null));
readonly "opacity"?: ((number) | (null));
readonly "roughness"?: ((number) | (null));
/** Zero evergreen; one deciduous. */
readonly "season"?: ((number) | (null));
readonly "seasonShift"?: ((number) | (null));
readonly "stains"?: ((number) | (null));
readonly "stochastic"?: ((number) | (null));
readonly "thickness"?: ((number) | (null));
/** Diffuse transmission through thin leaves; enables both faces. */
readonly "translucency"?: ((number) | (null));
readonly "transmission"?: ((number) | (null));
});
readonly "Occlusion": ({
readonly "radius": (number);
readonly "strength": (number);
});
readonly "Variation": ({
readonly "value"?: ((number) | (null));
readonly "warmth"?: ((number) | (null));
});
}
export type Input_model = ((({
readonly "desc": (Input_modelDefinitions["GeometryModel"]);
readonly "kind": ("geometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["ProgramModel"]);
readonly "kind": ("program") & (string);
})));
export interface Input_modelDefinitions {
readonly "Anchor": (((({
readonly "kind": ("none") & (string);
})) & { readonly "bottom"?: never; readonly "distance"?: never; readonly "ends"?: never; readonly "normal"?: never; }) | ((({
readonly "bottom": (boolean);
readonly "ends": (boolean);
readonly "kind": ("supports") & (string);
})) & { readonly "distance"?: never; readonly "normal"?: never; }) | ((({
readonly "distance": (number);
readonly "kind": ("plane") & (string);
readonly "normal": (ReadonlyArray<(number)> & { readonly length: 3 });
})) & { readonly "bottom"?: never; readonly "ends"?: never; }));
readonly "Animation": ({
readonly "clips"?: (ReadonlyArray<(Input_modelDefinitions["Clip"])> | (number extends ((Input_modelDefinitions["Clip"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "compress"?: ((boolean) | (null));
readonly "events"?: (ReadonlyArray<(Input_modelDefinitions["Event"])> | (number extends ((Input_modelDefinitions["Event"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "footIk"?: ((boolean) | (null));
readonly "libraries"?: (ReadonlyArray<(Input_modelDefinitions["ContentRef"])> | (number extends ((Input_modelDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "mirrors"?: (ReadonlyArray<(Input_modelDefinitions["Mirror2"])> | (number extends ((Input_modelDefinitions["Mirror2"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "rate"?: ((number) | (null));
readonly "rig"?: (((Input_modelDefinitions["ContentRef"])) | ((null)));
readonly "tolerance"?: ((number) | (null));
});
readonly "Arc": ({
readonly "center": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "clockwise"?: (boolean);
readonly "end": (number);
readonly "radius": (number);
readonly "start": (number);
});
readonly "Axis": ("x" | "y" | "z") & (string);
readonly "Bake": ({
/** Zero fills all uncovered texels. */
readonly "dilate"?: ((number) | (null));
readonly "distance"?: ((number) | (null));
readonly "normals"?: ((boolean) | (null));
readonly "padding"?: ((number) | (null));
readonly "size"?: ((number) | (null));
});
readonly "BoxGeometry": ({
/** Z extent, metres; unset 1. */
readonly "depth"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Y extent, metres; unset 1. */
readonly "height"?: ((number) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** X extent, metres; unset 1. */
readonly "width"?: ((number) | (null));
});
readonly "Branch": ({
/** -1 is the trunk; otherwise an earlier branch index. */
readonly "parent"?: ((number) | (null));
readonly "path": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "radii": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "segments"?: ((number) | (null));
});
readonly "BufferGeometry": ({
/** One linear RGB or RGBA colour per vertex. */
readonly "colors"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Counter-clockwise triangle indices; absent uses consecutive vertices. */
readonly "indices"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** One xyz normal per vertex; absent computes normals. */
readonly "normals"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Finite x/y/z vertex positions in metres. Triangle count, attribute
counts and index bounds are checked together by the domain validator. */
readonly "positions": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** One xyzw tangent frame per vertex; w is handedness. */
readonly "tangents"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** One uv pair per vertex; v down the image. */
readonly "uvs"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
});
readonly "CapsuleGeometry": ({
readonly "capSegments"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Straight middle length, not total capsule length. */
readonly "height"?: ((number) | (null));
readonly "heightSegments"?: ((number) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radialSegments"?: ((number) | (null));
readonly "radius"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "Channel": ({
readonly "interpolation"?: (((Input_modelDefinitions["Interpolation"])) | ((null)));
readonly "keys": (ReadonlyArray<(Input_modelDefinitions["Key"])> | (number extends ((Input_modelDefinitions["Key"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "node": (string);
readonly "path": (Input_modelDefinitions["ChannelPath"]);
});
readonly "ChannelPath": ("translation" | "rotation" | "scale" | "weights" | "visibility") & (string);
readonly "CircleGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radius"?: ((number) | (null));
readonly "segments"?: ((number) | (null));
readonly "thetaLength"?: ((number) | (null));
readonly "thetaStart"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "Clip": ({
readonly "channels": (ReadonlyArray<(Input_modelDefinitions["Channel"])> | (number extends ((Input_modelDefinitions["Channel"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "name": (string);
});
readonly "Collider": (((boolean)) | ((Input_modelDefinitions["ColliderMethod"])));
readonly "ColliderMethod": ("convex" | "vhacd") & (string);
readonly "ConeGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "height"?: ((number) | (null));
readonly "heightSegments"?: ((number) | (null));
readonly "openEnded"?: ((boolean) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radialSegments"?: ((number) | (null));
readonly "radius"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "Conform": ("off" | "character" | "effect") & (string);
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "CsgGeometry": ({
readonly "a": (Input_modelDefinitions["PlacedGeometry"]);
readonly "b": (Input_modelDefinitions["PlacedGeometry"]);
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "op"?: (((Input_modelDefinitions["CsgOp"])) | ((null)));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "CsgOp": ("union" | "subtract" | "intersect") & (string);
readonly "Curve": ({
readonly "closed"?: ((boolean) | (null));
readonly "kind"?: (((Input_modelDefinitions["CurveKind"])) | ((null)));
readonly "points": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Catmull-Rom tangent scale; unset 0.5. */
readonly "tension"?: ((number) | (null));
});
readonly "CurveKind": ("catmullRom" | "bezier" | "linear") & (string);
readonly "CylinderGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "height"?: ((number) | (null));
readonly "heightSegments"?: ((number) | (null));
readonly "openEnded"?: ((boolean) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radialSegments"?: ((number) | (null));
readonly "radiusBottom"?: ((number) | (null));
readonly "radiusTop"?: ((number) | (null));
readonly "thetaLength"?: ((number) | (null));
readonly "thetaStart"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "Decimate": ({
readonly "crossColorBorders"?: ((boolean) | (null));
readonly "lockBorder"?: ((boolean) | (null));
readonly "maxError"?: ((number) | (null));
readonly "mergeCharts"?: ((number) | (null));
readonly "ratio"?: ((number) | (null));
/** Zero leaves the triangle-count budget unset. */
readonly "target"?: ((number) | (null));
});
readonly "Ellipse": ({
readonly "center": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "clockwise"?: (boolean);
readonly "end": (number);
readonly "radii": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "rotation"?: (number);
readonly "start": (number);
});
readonly "Event": ({
readonly "clip": (string);
readonly "marker": (string);
readonly "time": (number);
});
readonly "ExtrudeGeometry": ({
readonly "bevelEnabled"?: ((boolean) | (null));
readonly "bevelOffset"?: ((number) | (null));
readonly "bevelSegments"?: ((number) | (null));
/** Unset is bevelThickness - 0.1, not a second fixed default. */
readonly "bevelSize"?: ((number) | (null));
readonly "bevelThickness"?: ((number) | (null));
readonly "curveSegments"?: ((number) | (null));
readonly "depth"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "extrudePath"?: (((Input_modelDefinitions["Curve"])) | ((null)));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Unset uses the generator's default square. */
readonly "shapes"?: ((ReadonlyArray<(Input_modelDefinitions["Shape"])> | (number extends ((Input_modelDefinitions["Shape"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "steps"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "Flowers": ({
readonly "centerColor"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "color2"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "count"?: ((number) | (null));
readonly "cup"?: ((number) | (null));
readonly "petals"?: ((number) | (null));
readonly "share"?: ((number) | (null));
readonly "size"?: ((number) | (null));
readonly "width"?: ((number) | (null));
});
readonly "Fracture": ({
readonly "anchor"?: (((Input_modelDefinitions["Anchor"])) | ((null)));
readonly "brick"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "children"?: ((number) | (null));
readonly "chunks"?: ((number) | (null));
readonly "grain"?: (((Input_modelDefinitions["Axis"])) | ((null)));
readonly "impact"?: (((Input_modelDefinitions["Impact"])) | ((null)));
readonly "interior"?: (((Input_modelDefinitions["Material"])) | ((null)));
readonly "interiorScale"?: ((number) | (null));
readonly "mortar"?: (((Input_modelDefinitions["Material"])) | ((null)));
readonly "pattern"?: (((Input_modelDefinitions["Pattern"])) | ((null)));
readonly "rings"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
/** Explicit centres replace sampling; count 2..128, distinct, volume
Voronoi only, exclusive with chunks/impact (domain validation). */
readonly "sites"?: ((ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "style"?: (((Input_modelDefinitions["Style"])) | ((null)));
});
/** Exactly one geometry, named as the corresponding render mechanism. */
readonly "Geometry": ((({
readonly "desc": (Input_modelDefinitions["ModelGeometry"]);
readonly "kind": ("modelGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["BufferGeometry"]);
readonly "kind": ("bufferGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["BoxGeometry"]);
readonly "kind": ("boxGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["PlaneGeometry"]);
readonly "kind": ("planeGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["SphereGeometry"]);
readonly "kind": ("sphereGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["CylinderGeometry"]);
readonly "kind": ("cylinderGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["ConeGeometry"]);
readonly "kind": ("coneGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["CapsuleGeometry"]);
readonly "kind": ("capsuleGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["TorusGeometry"]);
readonly "kind": ("torusGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["RingGeometry"]);
readonly "kind": ("ringGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["CircleGeometry"]);
readonly "kind": ("circleGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["TubeGeometry"]);
readonly "kind": ("tubeGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["ShapeGeometry"]);
readonly "kind": ("shapeGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["ExtrudeGeometry"]);
readonly "kind": ("extrudeGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["LatheGeometry"]);
readonly "kind": ("latheGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["PolyhedronGeometry"]);
readonly "kind": ("polyhedronGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["SolidGeometry"]);
readonly "kind": ("icosahedronGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["SolidGeometry"]);
readonly "kind": ("dodecahedronGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["SolidGeometry"]);
readonly "kind": ("octahedronGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["SolidGeometry"]);
readonly "kind": ("tetrahedronGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["SweepGeometry"]);
readonly "kind": ("sweepGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["CsgGeometry"]);
readonly "kind": ("csgGeometry") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["MergeGeometries"]);
readonly "kind": ("mergeGeometries") & (string);
})));
readonly "GeometryModel": ({
readonly "conform"?: (((Input_modelDefinitions["Conform"])) | ((null)));
readonly "fracture"?: (((Input_modelDefinitions["Fracture"])) | ((null)));
/** Ordered declarations: a named dependency must precede its consumer. */
readonly "items": (ReadonlyArray<(Input_modelDefinitions["Item"])> | (number extends ((Input_modelDefinitions["Item"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "options"?: (((Input_modelDefinitions["ModelOptions"])) | ((null)));
readonly "process"?: (((Input_modelDefinitions["Process"])) | ((null)));
});
readonly "Growth": ("deciduous" | "evergreen") & (string);
readonly "Impact": ({
readonly "position": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "radius": (number);
});
readonly "Interpolation": ("step" | "linear" | "cubicSpline") & (string);
readonly "Item": ({
readonly "input": (Input_modelDefinitions["ItemInput"]);
readonly "name": (string);
});
readonly "ItemInput": ((({
readonly "desc": (Input_modelDefinitions["Material"]);
readonly "kind": ("material") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["Pivot"]);
readonly "kind": ("pivot") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["Part"]);
readonly "kind": ("part") & (string);
})) | (({
readonly "desc": (Input_modelDefinitions["Plant"]);
readonly "kind": ("plant") & (string);
})));
readonly "Joint": ({
readonly "name": (string);
/** Empty string denotes root; all other parents must precede the child. */
readonly "parent": (string);
readonly "position": (ReadonlyArray<(number)> & { readonly length: 3 });
});
readonly "Key": ({
readonly "time": (number);
/** Metres, xyzw unit quaternion, scale factors, morph weights or 0/1.
Cubic spline keys contain in-tangent, value, out-tangent (per second). */
readonly "values": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "LatheGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "phiLength"?: ((number) | (null));
readonly "phiStart"?: ((number) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Radial x and height y, metres; unset uses the generator's profile. */
readonly "points"?: ((ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "segments"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "LeafShape": ("blade" | "card" | "cross") & (string);
readonly "Leaves": ({
readonly "angle"?: ((number) | (null));
readonly "autumnColor"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "autumnColor2"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "color2"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "colorVariation"?: ((number) | (null));
readonly "count"?: ((number) | (null));
readonly "domeNormals"?: ((number) | (null));
readonly "end"?: ((number) | (null));
readonly "facing"?: ((number) | (null));
readonly "material"?: ((string) | (null));
readonly "rows"?: ((number) | (null));
readonly "season"?: ((number) | (null));
readonly "seasonShift"?: ((number) | (null));
readonly "shape"?: (((Input_modelDefinitions["LeafShape"])) | ((null)));
readonly "size"?: ((number) | (null));
readonly "sizeVariance"?: ((number) | (null));
readonly "start"?: ((number) | (null));
readonly "translucency"?: ((number) | (null));
/** Width as a share of leaf length. */
readonly "width"?: ((number) | (null));
});
readonly "Level": ({
readonly "angle"?: ((number) | (null));
readonly "children"?: ((number) | (null));
readonly "gnarliness"?: ((number) | (null));
readonly "length"?: ((number) | (null));
/** Trunk radius in metres; branch radius as a share of the parent's. */
readonly "radius"?: ((number) | (null));
readonly "sections"?: ((number) | (null));
readonly "segments"?: ((number) | (null));
readonly "start"?: ((number) | (null));
readonly "taper"?: ((number) | (null));
readonly "twist"?: ((number) | (null));
});
/** Omitted fields remain absent until the material domain normalizes them. */
readonly "Material": ({
/** Alpha-mask cutoff takes precedence over opacity blending. */
readonly "alphaTest"?: ((number) | (null));
/** Ambient occlusion in red. */
readonly "aoMap"?: ((string) | (null));
readonly "autumnColor"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "autumnColor2"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "clearcoat"?: ((number) | (null));
readonly "clearcoatRoughness"?: ((number) | (null));
/** Linear base RGB (glTF factor); not a runtime colour handle. */
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "domeNormals"?: ((number) | (null));
readonly "doubleSided"?: ((boolean) | (null));
/** Linear emitted RGB, multiplied by emissiveIntensity. */
readonly "emissive"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "emissiveIntensity"?: ((number) | (null));
readonly "grime"?: ((number) | (null));
readonly "grimeColor"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "grimeHeight"?: ((number) | (null));
readonly "ior"?: ((number) | (null));
/** Source image inputs; not cooked texture references. Resolution and path
containment are checked at the input boundary. */
readonly "map"?: ((string) | (null));
readonly "metalness"?: ((number) | (null));
/** Roughness in green, metalness in blue. */
readonly "metalnessMap"?: ((string) | (null));
readonly "normalMap"?: ((string) | (null));
readonly "normalScale"?: ((number) | (null));
readonly "opacity"?: ((number) | (null));
readonly "roughness"?: ((number) | (null));
/** Zero evergreen; one deciduous. */
readonly "season"?: ((number) | (null));
readonly "seasonShift"?: ((number) | (null));
readonly "stains"?: ((number) | (null));
readonly "stochastic"?: ((number) | (null));
readonly "thickness"?: ((number) | (null));
/** Diffuse transmission through thin leaves; enables both faces. */
readonly "translucency"?: ((number) | (null));
readonly "transmission"?: ((number) | (null));
});
readonly "MergeGeometries": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "parts": (ReadonlyArray<(Input_modelDefinitions["PlacedGeometry"])> | (number extends ((Input_modelDefinitions["PlacedGeometry"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "Mirror": ("none" | "x" | "y" | "z") & (string);
readonly "Mirror2": ({
readonly "clip": (string);
readonly "name"?: ((string) | (null));
});
readonly "ModelGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Select a mesh node, or merge the visible default scene at rest. */
readonly "node"?: ((string) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** A model cooked before this geometry; dependency cycles are domain errors. */
readonly "source": (Input_modelDefinitions["ContentRef"]);
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "ModelOptions": ({
readonly "animation"?: (((Input_modelDefinitions["Animation"])) | ((null)));
readonly "collider"?: (((Input_modelDefinitions["Collider"])) | ((null)));
readonly "impostor"?: ((boolean) | (null));
readonly "impostorBelow"?: ((boolean) | (null));
readonly "impostorGrid"?: ((number) | (null));
/** Atlas width (grid × frame size) must not exceed 4096 pixels. */
readonly "impostorSize"?: ((number) | (null));
readonly "lodError"?: ((number) | (null));
/** At most eight strictly decreasing fractions beginning at one. */
readonly "lods"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "raycast"?: ((boolean) | (null));
/** Opt-in probe-region skin relaxation (12 passes); absent means off. */
readonly "relaxSkin"?: ((boolean) | (null));
readonly "streamTextures"?: ((boolean) | (null));
});
readonly "Part": ({
readonly "geometry": (Input_modelDefinitions["Geometry"]);
readonly "material"?: ((string) | (null));
readonly "pivot"?: ((string) | (null));
/** Omitted: identity placement. Explicit placements must be nonempty; a
single placement also represents the old part's own TRS, with no XOR. */
readonly "placements"?: ((ReadonlyArray<(Input_modelDefinitions["Placement"])> | (number extends ((Input_modelDefinitions["Placement"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
});
/** Ordered Path2 operations. These are structured calls, not embedded OGT. */
readonly "Path2Command": ((({
readonly "args": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "op": ("moveTo") & (string);
})) | (({
readonly "args": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "op": ("lineTo") & (string);
})) | (({
readonly "args": (ReadonlyArray<(number)> & { readonly length: 4 });
readonly "op": ("quadraticCurveTo") & (string);
})) | (({
readonly "args": (ReadonlyArray<(number)> & { readonly length: 6 });
readonly "op": ("bezierCurveTo") & (string);
})) | (({
readonly "args": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "op": ("splineThru") & (string);
})) | (({
readonly "args": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "op": ("points") & (string);
})) | (({
readonly "args": (Input_modelDefinitions["Arc"]);
readonly "op": ("arc") & (string);
})) | (({
readonly "args": (Input_modelDefinitions["Arc"]);
readonly "op": ("absarc") & (string);
})) | (({
readonly "args": (Input_modelDefinitions["Ellipse"]);
readonly "op": ("ellipse") & (string);
})) | (({
readonly "args": (Input_modelDefinitions["Ellipse"]);
readonly "op": ("absellipse") & (string);
})) | ((({
readonly "op": ("closePath") & (string);
})) & { readonly "args"?: never; }));
readonly "Pattern": ("auto" | "volume" | "slab") & (string);
readonly "Pivot": ({
/** Previously declared pivot. Positive scale and nonsingular rotation are
model-domain constraints; reflected Transform is not duplicated here. */
readonly "parent"?: ((string) | (null));
readonly "transform"?: (((Input_modelDefinitions["Reflected_og_cook::cook::source_schemas::world::geometry::Transform"])) | ((null)));
});
/** One CSG operand or merged part. The transform is the actual reflected
engine component, not a copied table of position/quaternion/scale fields. */
readonly "PlacedGeometry": ({
readonly "geometry": (Input_modelDefinitions["Geometry"]);
readonly "mirror"?: (((Input_modelDefinitions["Mirror"])) | ((null)));
readonly "transform"?: (((Input_modelDefinitions["Reflected_og_cook::cook::source_schemas::world::geometry::Transform"])) | ((null)));
});
readonly "Placement": ({
/** Reflections bake into geometry with corrected triangle winding. */
readonly "mirror"?: (((Input_modelDefinitions["Mirror"])) | ((null)));
readonly "transform"?: (((Input_modelDefinitions["Reflected_og_cook::cook::source_schemas::world::geometry::Transform"])) | ((null)));
});
readonly "PlaneGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Y extent, metres; unset 1. */
readonly "height"?: ((number) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** X extent, metres; unset 1. The plane faces +Z. */
readonly "width"?: ((number) | (null));
});
readonly "Plant": ({
readonly "barkColor"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
/** Model-space minimum/maximum, metres; ordering is a domain check. */
readonly "bounds"?: ((ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> & { readonly length: 2 }) | (null));
readonly "branch1"?: (((Input_modelDefinitions["Level"])) | ((null)));
readonly "branch2"?: (((Input_modelDefinitions["Level"])) | ((null)));
readonly "branch3"?: (((Input_modelDefinitions["Level"])) | ((null)));
readonly "branches"?: (ReadonlyArray<(Input_modelDefinitions["Branch"])> | (number extends ((Input_modelDefinitions["Branch"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "flowerGeometry"?: (((Input_modelDefinitions["Geometry"])) | ((null)));
readonly "flowers"?: (((Input_modelDefinitions["Flowers"])) | ((null)));
/** Direction xyz and bending strength. */
readonly "force"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "growth"?: (((Input_modelDefinitions["Growth"])) | ((null)));
readonly "kind"?: (((Input_modelDefinitions["PlantKind"])) | ((null)));
readonly "leafGeometry"?: (((Input_modelDefinitions["Geometry"])) | ((null)));
readonly "leaves"?: (((Input_modelDefinitions["Leaves"])) | ((null)));
readonly "levels"?: ((number) | (null));
readonly "material"?: ((string) | (null));
readonly "path"?: (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "seed"?: ((number) | (null));
readonly "trunk"?: (((Input_modelDefinitions["Level"])) | ((null)));
});
readonly "PlantKind": ("oak" | "birch" | "willow" | "pine" | "palm" | "bamboo" | "cactus" | "shrub" | "fern" | "reed" | "daisy" | "tulip") & (string);
readonly "PolyhedronGeometry": ({
readonly "detail"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** At least one triangle, indexing the corner triples. */
readonly "indices": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radius"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** Flat xyz corner coordinates, metres. */
readonly "vertices": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Process": ({
readonly "bake"?: (((Input_modelDefinitions["Bake"])) | ((null)));
readonly "decimate"?: (((Input_modelDefinitions["Decimate"])) | ((null)));
readonly "project"?: (((Input_modelDefinitions["Project"])) | ((null)));
readonly "repair"?: (((Input_modelDefinitions["Repair"])) | ((null)));
readonly "rig"?: (((Input_modelDefinitions["Rig"])) | ((null)));
readonly "transfer"?: (((Input_modelDefinitions["Transfer"])) | ((null)));
readonly "unwrap"?: (((Input_modelDefinitions["Unwrap"])) | ((null)));
});
/** Program material overrides have their own source semantics (sheen and
roughness/emissive maps); they are not the procedural material's field set. */
readonly "ProgramMaterial": ({
readonly "alphaTest"?: ((number) | (null));
readonly "aoMap"?: ((string) | (null));
readonly "aoMapIntensity"?: ((number) | (null));
readonly "clearcoat"?: ((number) | (null));
readonly "clearcoatRoughness"?: ((number) | (null));
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "doubleSided"?: ((boolean) | (null));
readonly "emissive"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "emissiveIntensity"?: ((number) | (null));
readonly "emissiveMap"?: ((string) | (null));
readonly "iridescence"?: ((number) | (null));
readonly "map"?: ((string) | (null));
readonly "metalness"?: ((number) | (null));
readonly "name": (string);
readonly "normalMap"?: ((string) | (null));
readonly "normalScale"?: ((number) | (null));
readonly "opacity"?: ((number) | (null));
readonly "roughness"?: ((number) | (null));
readonly "roughnessMap"?: ((string) | (null));
readonly "sheen"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "sheenRoughness"?: ((number) | (null));
readonly "transmission"?: ((number) | (null));
readonly "transparent"?: ((boolean) | (null));
});
readonly "ProgramModel": ({
readonly "atlas"?: ((number) | (null));
readonly "conform"?: (((Input_modelDefinitions["Conform"])) | ((null)));
readonly "materials"?: (ReadonlyArray<(Input_modelDefinitions["ProgramMaterial"])> | (number extends ((Input_modelDefinitions["ProgramMaterial"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "options"?: (((Input_modelDefinitions["ModelOptions"])) | ((null)));
readonly "params"?: (Readonly<Record<string, (number)>>);
readonly "process"?: (((Input_modelDefinitions["Process"])) | ((null)));
readonly "program": (Input_modelDefinitions["ContentRef"]);
});
readonly "Project": ({
readonly "padding"?: ((number) | (null));
readonly "size"?: ((number) | (null));
readonly "views": (ReadonlyArray<(Input_modelDefinitions["View"])> | (number extends ((Input_modelDefinitions["View"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Projection": (((({
readonly "fov": (number);
readonly "kind": ("perspective") & (string);
})) & { readonly "ortho"?: never; }) | ((({
readonly "kind": ("orthographic") & (string);
readonly "ortho": (number);
})) & { readonly "fov"?: never; }));
readonly "Reflected_og_cook::cook::source_schemas::world::geometry::Transform": ({
/** The position, as three.js Object3D.position. */
readonly "position"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The rotation, as three.js Object3D.quaternion. */
readonly "quaternion"?: (ReadonlyArray<(number)> & { readonly length: 4 });
/** The scale, as three.js Object3D.scale. */
readonly "scale"?: (ReadonlyArray<(number)> & { readonly length: 3 });
});
readonly "Repair": ({
readonly "floaterFraction"?: ((number) | (null));
readonly "floaters"?: ((number) | (null));
readonly "fuse"?: ((boolean) | (null));
readonly "holeEdges"?: ((number) | (null));
readonly "holesThrough"?: ((boolean) | (null));
readonly "minEdge"?: ((number) | (null));
readonly "nonManifold"?: ((boolean) | (null));
readonly "thicken"?: ((number) | (null));
readonly "weld"?: ((number) | (null));
readonly "weldAll"?: ((boolean) | (null));
});
readonly "Rig": (((({
readonly "kind": ("auto") & (string);
})) & { readonly "joints"?: never; }) | (({
readonly "joints": (ReadonlyArray<(Input_modelDefinitions["Joint"])> | (number extends ((Input_modelDefinitions["Joint"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("joints") & (string);
})));
readonly "RingGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "innerRadius"?: ((number) | (null));
readonly "outerRadius"?: ((number) | (null));
readonly "phiSegments"?: ((number) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "thetaLength"?: ((number) | (null));
readonly "thetaSegments"?: ((number) | (null));
readonly "thetaStart"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "Shape": ({
readonly "commands": (ReadonlyArray<(Input_modelDefinitions["Path2Command"])> | (number extends ((Input_modelDefinitions["Path2Command"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Hole contours cannot contain more holes. */
readonly "holes"?: (ReadonlyArray<(ReadonlyArray<(Input_modelDefinitions["Path2Command"])> | (number extends ((Input_modelDefinitions["Path2Command"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))> | (number extends ((ReadonlyArray<(Input_modelDefinitions["Path2Command"])> | (number extends ((Input_modelDefinitions["Path2Command"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "ShapeGeometry": ({
readonly "curveSegments"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Unset uses the generator's default triangle. */
readonly "shapes"?: ((ReadonlyArray<(Input_modelDefinitions["Shape"])> | (number extends ((Input_modelDefinitions["Shape"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "SolidGeometry": ({
readonly "detail"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radius"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "SphereGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "heightSegments"?: ((number) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radius"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "widthSegments"?: ((number) | (null));
});
readonly "Style": ("voronoi" | "concrete" | "brick" | "wood" | "glass") & (string);
readonly "SweepGeometry": ({
readonly "caps"?: ((boolean) | (null));
readonly "curveSegments"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "path": (Input_modelDefinitions["Curve"]);
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Ordered interpolation keys along the path (t, scale). */
readonly "scale"?: ((ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Cross-section without holes. Curve flattening uses curveSegments. */
readonly "section": (ReadonlyArray<(Input_modelDefinitions["Path2Command"])> | (number extends ((Input_modelDefinitions["Path2Command"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "steps"?: ((number) | (null));
/** Ordered interpolation keys (t, radians). */
readonly "twist"?: ((ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "TorusGeometry": ({
readonly "arc"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radialSegments"?: ((number) | (null));
readonly "radius"?: ((number) | (null));
readonly "tube"?: ((number) | (null));
readonly "tubularSegments"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "Transfer": ({
readonly "padding"?: ((number) | (null));
readonly "size"?: ((number) | (null));
readonly "source": (Input_modelDefinitions["ContentRef"]);
});
readonly "TubeGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** A sampled path or a Catmull-Rom point list, never a runtime Path handle. */
readonly "path": (Input_modelDefinitions["TubePath"]);
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radialSegments"?: ((number) | (null));
readonly "radius"?: ((number) | (null));
readonly "tubularSegments"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "TubePath": (((({
readonly "closed"?: ((boolean) | (null));
readonly "kind": ("points") & (string);
readonly "points": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "curve"?: never; }) | ((({
readonly "curve": (Input_modelDefinitions["Curve"]);
readonly "kind": ("curve") & (string);
})) & { readonly "closed"?: never; readonly "points"?: never; }));
readonly "Unwrap": ({
readonly "angle"?: ((number) | (null));
readonly "conformal"?: ((boolean) | (null));
readonly "padding"?: ((number) | (null));
readonly "resolution"?: ((number) | (null));
readonly "seams"?: ((boolean) | (null));
readonly "shared"?: ((boolean) | (null));
});
readonly "View": ({
/** Square RGBA source; actual dimensions/decoding are input-domain checks. */
readonly "image": (string);
readonly "position": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "projection": (Input_modelDefinitions["Projection"]);
readonly "target": (ReadonlyArray<(number)> & { readonly length: 3 });
});
}
/** Operations use Z up and the declared model unit (mm by default); cooking
converts the result to metres/Y up. Angles are radians, times seconds. */
export type Input_model_program = ({
/** Omitted: the shared kernel's model name. */
readonly "name"?: ((string) | (null));
readonly "ops": (ReadonlyArray<(Input_model_programDefinitions["Operation"])> | (number extends ((Input_model_programDefinitions["Operation"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "params"?: (Readonly<Record<string, (number)>>);
});
export interface Input_model_programDefinitions {
readonly "AlongPath": ({
readonly "closed"?: ((boolean) | (null));
readonly "points": (ReadonlyArray<(ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Arc": ({
readonly "through": (ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 2 });
readonly "to": (ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 2 });
});
readonly "Arc2": ({
readonly "through": (ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 });
readonly "to": (ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 });
});
readonly "Axis": ("+Y" | "+Z") & (string);
readonly "BasicOperation": (((({
readonly "normal": (ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 });
readonly "op": ("frame") & (string);
readonly "origin": (ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 });
readonly "up"?: ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 }) | (null));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "distance": (Input_model_programDefinitions["Number"]);
readonly "frame": (Input_model_programDefinitions["Frame"]);
readonly "op": ("offset") & (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "frame": (Input_model_programDefinitions["Frame"]);
readonly "op": ("translate") & (string);
readonly "x"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "y"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "z"?: (((Input_model_programDefinitions["Number"])) | ((null)));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; }) | ((({
readonly "frame": (Input_model_programDefinitions["Frame"]);
readonly "op": ("turn") & (string);
readonly "rx"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "ry"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "rz"?: (((Input_model_programDefinitions["Number"])) | ((null)));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "frame": (Input_model_programDefinitions["Frame"]);
readonly "local": (Input_model_programDefinitions["Frame"]);
readonly "op": ("compose") & (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "ids": (Input_model_programDefinitions["Ids"]);
readonly "op": ("bbox") & (string);
readonly "sides": (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "name": (string);
readonly "op": ("face") & (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "name": (string);
readonly "op": ("edge") & (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "frame": (Input_model_programDefinitions["Frame"]);
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "op": ("local") & (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "name": (string);
readonly "op": ("anchor") & (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "op": ("placement") & (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "nx"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "ny"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": ("grid") & (string);
readonly "px"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "py"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "stagger"?: ((boolean) | (null));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "angle"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "n"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": ("ring") & (string);
readonly "radius": (Input_model_programDefinitions["Number"]);
readonly "start"?: (((Input_model_programDefinitions["Number"])) | ((null)));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "count"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "op": ("along") & (string);
readonly "path": (Input_model_programDefinitions["AlongPath"]);
readonly "pitch"?: (((Input_model_programDefinitions["Number"])) | ((null)));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "direction"?: ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 }) | (null));
readonly "ids": (Input_model_programDefinitions["Ids"]);
readonly "op": ("snap") & (string);
readonly "points": (ReadonlyArray<(ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "op": ("names") & (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "op": ("measure") & (string);
readonly "posed"?: ((boolean) | (null));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "op": ("units") & (string);
readonly "unit": (Input_model_programDefinitions["Unit"]);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "op": ("uvs") & (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "depth": (Input_model_programDefinitions["Number"]);
readonly "height": (Input_model_programDefinitions["Number"]);
readonly "name": (string);
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": ("box") & (string);
readonly "uvs"?: ((boolean) | (null));
readonly "width": (Input_model_programDefinitions["Number"]);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "height": (Input_model_programDefinitions["Number"]);
readonly "name": (string);
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": ("cylinder") & (string);
readonly "radius": (Input_model_programDefinitions["Number"]);
readonly "radiusTop"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "segments"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "uvs"?: ((boolean) | (null));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "height": (Input_model_programDefinitions["Number"]);
readonly "name": (string);
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": ("cone") & (string);
readonly "radius": (Input_model_programDefinitions["Number"]);
readonly "segments"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "uvs"?: ((boolean) | (null));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "axis"?: (((Input_model_programDefinitions["Axis"])) | ((null)));
readonly "name": (string);
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": ("sphere") & (string);
readonly "radius": (Input_model_programDefinitions["Number"]);
readonly "segments"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "uvs"?: ((boolean) | (null));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "axis"?: (((Input_model_programDefinitions["Axis"])) | ((null)));
readonly "height"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "name": (string);
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": ("dome") & (string);
readonly "radius": (Input_model_programDefinitions["Number"]);
readonly "segments"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "uvs"?: ((boolean) | (null));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "height": (Input_model_programDefinitions["Number"]);
readonly "inner": (Input_model_programDefinitions["Number"]);
readonly "name": (string);
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": ("tube") & (string);
readonly "outer": (Input_model_programDefinitions["Number"]);
readonly "segments"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "uvs"?: ((boolean) | (null));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "angle"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "axis"?: (((Input_model_programDefinitions["Axis"])) | ((null)));
readonly "name": (string);
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": ("torus") & (string);
readonly "radius": (Input_model_programDefinitions["Number"]);
readonly "segments"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "tube": (Input_model_programDefinitions["Number"]);
readonly "tubeSegments"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "uvs"?: ((boolean) | (null));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "at"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "depth": (Input_model_programDefinitions["Number"]);
readonly "height": (Input_model_programDefinitions["Number"]);
readonly "name": (string);
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": ("wedge") & (string);
readonly "uvs"?: ((boolean) | (null));
readonly "width": (Input_model_programDefinitions["Number"]);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "angle"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "axis"?: (((Input_model_programDefinitions["Axis"])) | ((null)));
readonly "name": (string);
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": ("revolve") & (string);
readonly "profile": (Input_model_programDefinitions["Profile"]);
readonly "segments"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "uvs"?: ((boolean) | (null));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "at"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "name": (string);
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": ("sweep") & (string);
readonly "path": (Input_model_programDefinitions["Path3"]);
readonly "profile": (Input_model_programDefinitions["Profile"]);
readonly "scaleEnd"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "twist"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "up"?: ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 }) | (null));
readonly "uvs"?: ((boolean) | (null));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "unit"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "closed"?: ((boolean) | (null));
readonly "curve"?: (((Input_model_programDefinitions["Curve"])) | ((null)));
readonly "name": (string);
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": ("loft") & (string);
readonly "samples"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "sections": (Input_model_programDefinitions["Sections"]);
readonly "steps"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "uvs"?: ((boolean) | (null));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "depth": (Input_model_programDefinitions["Number"]);
readonly "length": (Input_model_programDefinitions["Number"]);
readonly "name": (string);
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": ("thread") & (string);
readonly "pitch": (Input_model_programDefinitions["Number"]);
readonly "radius": (Input_model_programDefinitions["Number"]);
readonly "uvs"?: ((boolean) | (null));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "closed"?: ((boolean) | (null));
readonly "diameter": (Input_model_programDefinitions["Number"]);
readonly "fillet"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "fillets"?: ((ReadonlyArray<(Input_model_programDefinitions["Number"])> | (number extends ((Input_model_programDefinitions["Number"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "name": (string);
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": ("wire") & (string);
readonly "path": (Input_model_programDefinitions["Path3"]);
readonly "segments"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "uvs"?: ((boolean) | (null));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "bottom": (Input_model_programDefinitions["View"]);
readonly "n"?: (((Input_model_programDefinitions["View"])) | ((null)));
readonly "name": (string);
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": ("body") & (string);
readonly "plan": (Input_model_programDefinitions["View"]);
readonly "top": (Input_model_programDefinitions["View"]);
readonly "uvs"?: ((boolean) | (null));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "diameter": (Input_model_programDefinitions["Number"]);
readonly "height": (Input_model_programDefinitions["Number"]);
readonly "name": (string);
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": ("knurl") & (string);
readonly "pitch"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "straight"?: ((boolean) | (null));
readonly "uvs"?: ((boolean) | (null));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "bore"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "hub"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "hubHeight"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "module": (Input_model_programDefinitions["Number"]);
readonly "name": (string);
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": ("gear") & (string);
readonly "pressureAngle"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "teeth"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "uvs"?: ((boolean) | (null));
readonly "width": (Input_model_programDefinitions["Number"]);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "frame": (Input_model_programDefinitions["Frame"]);
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "op": ("at") & (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "frame": (Input_model_programDefinitions["Frame"]);
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "op": ("transform") & (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "children": (Input_model_programDefinitions["Ids"]);
readonly "name": (string);
readonly "op": ("group") & (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "name": (string);
readonly "op": ("wrap") & (string);
readonly "parts": (Input_model_programDefinitions["Ids"]);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "anchors"?: ((Readonly<Record<string, (Input_model_programDefinitions["Frame"])>>) | (null));
readonly "children": (Input_model_programDefinitions["Ids"]);
readonly "name": (string);
readonly "op": ("assembly") & (string);
})) & { readonly "amount"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "children": (Input_model_programDefinitions["Ids"]);
readonly "name": (string);
readonly "op": ("attach") & (string);
readonly "to": (Input_model_programDefinitions["Id"]);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "name": (string);
readonly "op": ("copy") & (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "name": (string);
readonly "op": ("mirror") & (string);
readonly "plane": (Input_model_programDefinitions["Frame"]);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "check"?: ((boolean) | (null));
readonly "op": ("cut") & (string);
readonly "target": (Input_model_programDefinitions["Id"]);
readonly "tool": (Input_model_programDefinitions["Id"]);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "check"?: ((boolean) | (null));
readonly "op": ("join") & (string);
readonly "target": (Input_model_programDefinitions["Id"]);
readonly "tool": (Input_model_programDefinitions["Id"]);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "check"?: ((boolean) | (null));
readonly "op": ("intersect") & (string);
readonly "target": (Input_model_programDefinitions["Id"]);
readonly "tool": (Input_model_programDefinitions["Id"]);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "op": ("merge") & (string);
readonly "target": (Input_model_programDefinitions["Id"]);
readonly "tool": (Input_model_programDefinitions["Id"]);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "op": ("delete") & (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "host": (Input_model_programDefinitions["Id"]);
readonly "op": ("seat") & (string);
readonly "part": (Input_model_programDefinitions["Id"]);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "op": ("scale") & (string);
readonly "x"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "y"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "z"?: (((Input_model_programDefinitions["Number"])) | ((null)));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; }) | ((({
readonly "at": (Input_model_programDefinitions["Frames"]);
readonly "counterbore"?: ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 2 }) | (null));
readonly "countersink"?: ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 2 }) | (null));
readonly "depth"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "diameter": (Input_model_programDefinitions["Number"]);
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "op": ("hole") & (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "at"?: (((Input_model_programDefinitions["Frames"])) | ((null)));
readonly "depth"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "op": ("pocket") & (string);
readonly "profile": (Input_model_programDefinitions["Profile"]);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "frame": (Input_model_programDefinitions["Frame"]);
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "keep"?: (((Input_model_programDefinitions["Keep"])) | ((null)));
readonly "op": ("split") & (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "op": ("shell") & (string);
readonly "open"?: (((Input_model_programDefinitions["Names"])) | ((null)));
readonly "thickness": (Input_model_programDefinitions["Number"]);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "at": (ReadonlyArray<(ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "depth": (Input_model_programDefinitions["Number"]);
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "op": ("dent") & (string);
readonly "radius": (Input_model_programDefinitions["Number"]);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "levels"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "op": ("subdivide") & (string);
readonly "sharp"?: (((Input_model_programDefinitions["Names"])) | ((null)));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "amount": (Input_model_programDefinitions["Number"]);
readonly "density"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "noise"?: (((Input_model_programDefinitions["Noise"])) | ((null)));
readonly "octaves"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "op": ("displace") & (string);
readonly "scale": (Input_model_programDefinitions["Number"]);
readonly "seed"?: (((Input_model_programDefinitions["Count"])) | ((null)));
})) & { readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "edges"?: (((Input_model_programDefinitions["Names"])) | ((null)));
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "op": ("fillet") & (string);
readonly "profile"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "radius": (Input_model_programDefinitions["Number"]);
readonly "segments"?: (((Input_model_programDefinitions["Count"])) | ((null)));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "angle"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "distance": (Input_model_programDefinitions["Number"]);
readonly "distance2"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "edges"?: (((Input_model_programDefinitions["Names"])) | ((null)));
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "op": ("chamfer") & (string);
readonly "segments"?: (((Input_model_programDefinitions["Count"])) | ((null)));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "angle"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "edges"?: (((Input_model_programDefinitions["Names"])) | ((null)));
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "op": ("soften") & (string);
readonly "radius": (Input_model_programDefinitions["Number"]);
readonly "segments"?: (((Input_model_programDefinitions["Count"])) | ((null)));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "axis"?: (((Input_model_programDefinitions["Direction"])) | ((null)));
readonly "child": (Input_model_programDefinitions["Id"]);
readonly "kind"?: (((Input_model_programDefinitions["JointKind"])) | ((null)));
readonly "mimic"?: (((Input_model_programDefinitions["Mimic"])) | ((null)));
readonly "name": (string);
readonly "on": (Input_model_programDefinitions["Frame"]);
readonly "op": ("joint") & (string);
readonly "parent": (Input_model_programDefinitions["Id"]);
readonly "range"?: ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 2 }) | (null));
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(Input_model_programDefinitions["PoseKey"])> | (number extends ((Input_model_programDefinitions["PoseKey"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "name": (string);
readonly "op": ("clip") & (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bind"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "id"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }) | ((({
readonly "bind"?: ((boolean) | (null));
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "op": ("humanoid") & (string);
})) & { readonly "amount"?: never; readonly "anchors"?: never; readonly "angle"?: never; readonly "at"?: never; readonly "axis"?: never; readonly "bore"?: never; readonly "bottom"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "closed"?: never; readonly "count"?: never; readonly "counterbore"?: never; readonly "countersink"?: never; readonly "curve"?: never; readonly "density"?: never; readonly "depth"?: never; readonly "diameter"?: never; readonly "direction"?: never; readonly "distance"?: never; readonly "distance2"?: never; readonly "edges"?: never; readonly "fillet"?: never; readonly "fillets"?: never; readonly "frame"?: never; readonly "height"?: never; readonly "host"?: never; readonly "hub"?: never; readonly "hubHeight"?: never; readonly "ids"?: never; readonly "inner"?: never; readonly "keep"?: never; readonly "keys"?: never; readonly "kind"?: never; readonly "length"?: never; readonly "levels"?: never; readonly "local"?: never; readonly "mimic"?: never; readonly "module"?: never; readonly "n"?: never; readonly "name"?: never; readonly "noise"?: never; readonly "normal"?: never; readonly "nx"?: never; readonly "ny"?: never; readonly "octaves"?: never; readonly "on"?: never; readonly "open"?: never; readonly "origin"?: never; readonly "outer"?: never; readonly "parent"?: never; readonly "part"?: never; readonly "parts"?: never; readonly "path"?: never; readonly "pitch"?: never; readonly "plan"?: never; readonly "plane"?: never; readonly "points"?: never; readonly "posed"?: never; readonly "pressureAngle"?: never; readonly "profile"?: never; readonly "px"?: never; readonly "py"?: never; readonly "radius"?: never; readonly "radiusTop"?: never; readonly "range"?: never; readonly "rx"?: never; readonly "ry"?: never; readonly "rz"?: never; readonly "samples"?: never; readonly "scale"?: never; readonly "scaleEnd"?: never; readonly "sections"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "sharp"?: never; readonly "sides"?: never; readonly "stagger"?: never; readonly "start"?: never; readonly "steps"?: never; readonly "straight"?: never; readonly "target"?: never; readonly "teeth"?: never; readonly "thickness"?: never; readonly "to"?: never; readonly "tool"?: never; readonly "top"?: never; readonly "tube"?: never; readonly "tubeSegments"?: never; readonly "twist"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "uvs"?: never; readonly "width"?: never; readonly "x"?: never; readonly "y"?: never; readonly "z"?: never; }));
readonly "Bone": ({
readonly "name": (string);
readonly "parent"?: ((string) | (null));
readonly "pivot"?: (((Input_model_programDefinitions["Pivot"])) | ((null)));
readonly "regions"?: ((ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
});
readonly "Circle": ({
readonly "radius"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "segments"?: (((Input_model_programDefinitions["Count"])) | ((null)));
});
readonly "ClipPose": ({
readonly "clip": (string);
readonly "op": (Input_model_programDefinitions["PoseTag"]);
readonly "time": (Input_model_programDefinitions["Number"]);
});
/** sRGB colour. Numeric triples may contain parameter expressions. */
readonly "Color": (((string)) | ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 })));
readonly "Corner": ({
/** Circular bulge in half-chords. */
readonly "bulge"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "radius"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "x": (Input_model_programDefinitions["Number"]);
readonly "y": (Input_model_programDefinitions["Number"]);
});
/** Whole u32 counts, or expressions whose evaluated result must be whole u32.
Zero selects a kernel automatic choice only where that mechanism supports it. */
readonly "Count": (((number)) | ((string)));
readonly "Curve": ("auto" | "straight" | "catmullRom" | "spline") & (string);
readonly "CurvePiece": (((({
readonly "arc": (Input_model_programDefinitions["Arc"]);
})) & { readonly "spline"?: never; readonly "tangentArc"?: never; }) | ((({
readonly "tangentArc": (Input_model_programDefinitions["TangentArc"]);
})) & { readonly "arc"?: never; readonly "spline"?: never; }) | ((({
readonly "spline": (Input_model_programDefinitions["PathSpline"]);
})) & { readonly "arc"?: never; readonly "tangentArc"?: never; }));
readonly "CurvePiece2": (((({
readonly "arc": (Input_model_programDefinitions["Arc2"]);
})) & { readonly "spline"?: never; readonly "tangentArc"?: never; }) | ((({
readonly "tangentArc": (Input_model_programDefinitions["TangentArc2"]);
})) & { readonly "arc"?: never; readonly "spline"?: never; }) | ((({
readonly "spline": (Input_model_programDefinitions["PathSpline2"]);
})) & { readonly "arc"?: never; readonly "tangentArc"?: never; }));
readonly "DeltaMorph": ({
readonly "deltas": (ReadonlyArray<(ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "name": (string);
readonly "op": (Input_model_programDefinitions["MorphTag"]);
readonly "weight"?: (((Input_model_programDefinitions["Number"])) | ((null)));
});
readonly "Direction": (((Input_model_programDefinitions["SignedAxis"])) | ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 })));
readonly "Ellipse": ({
readonly "rx"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "ry"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "segments"?: (((Input_model_programDefinitions["Count"])) | ((null)));
});
readonly "ExtrudeHeight": ({
readonly "bothSides"?: ((boolean) | (null));
readonly "height": (Input_model_programDefinitions["Number"]);
readonly "name": (string);
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": (Input_model_programDefinitions["ExtrudeTag"]);
readonly "profile": (Input_model_programDefinitions["Profile"]);
readonly "taper"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "twist"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "uvs"?: ((boolean) | (null));
});
readonly "ExtrudeTag": ("extrude") & (string);
readonly "ExtrudeUntil": ({
readonly "name": (string);
readonly "on"?: (((Input_model_programDefinitions["Frame"])) | ((null)));
readonly "op": (Input_model_programDefinitions["ExtrudeTag"]);
readonly "profile": (Input_model_programDefinitions["Profile"]);
readonly "until": (Input_model_programDefinitions["Until"]);
readonly "uvs"?: ((boolean) | (null));
});
readonly "Extrusion": ((((Input_model_programDefinitions["ExtrudeHeight"])) & { readonly "until"?: never; }) | (((Input_model_programDefinitions["ExtrudeUntil"])) & { readonly "bothSides"?: never; readonly "height"?: never; readonly "taper"?: never; readonly "twist"?: never; }));
/** Orthonormal right-handed basis. The kernel rejects singular/non-rigid frames. */
readonly "Frame": ({
readonly "origin": (ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 });
readonly "x": (ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 });
readonly "y": (ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 });
readonly "z": (ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 });
});
readonly "Frames": (((Input_model_programDefinitions["Frame"])) | ((ReadonlyArray<(Input_model_programDefinitions["Frame"])> | (number extends ((Input_model_programDefinitions["Frame"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))));
readonly "Gear": ({
readonly "module"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "pressureAngle"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "teeth"?: (((Input_model_programDefinitions["Count"])) | ((null)));
});
readonly "HeatSkin": ({
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "joints": (ReadonlyArray<(Input_model_programDefinitions["SkinJoint"])> | (number extends ((Input_model_programDefinitions["SkinJoint"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "op": (Input_model_programDefinitions["SkinTag"]);
});
readonly "HeightSection": ({
readonly "profile": (Input_model_programDefinitions["Profile"]);
readonly "z": (Input_model_programDefinitions["Number"]);
});
readonly "Helix": ({
readonly "pitch": (Input_model_programDefinitions["Number"]);
readonly "radius": (Input_model_programDefinitions["Number"]);
readonly "turns": (Input_model_programDefinitions["Number"]);
});
readonly "HelixPath": ({
readonly "helix": (Input_model_programDefinitions["Helix"]);
});
readonly "Id": (((number)) | ((Input_model_programDefinitions["IdHandle"])));
readonly "IdHandle": ({
readonly "id": (number);
});
readonly "Ids": (((Input_model_programDefinitions["Id"])) | ((ReadonlyArray<(Input_model_programDefinitions["Id"])> | (number extends ((Input_model_programDefinitions["Id"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))));
readonly "ImageMaterial": ({
readonly "clearcoat"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "clearcoatRoughness"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "color"?: (((Input_model_programDefinitions["Color"])) | ((null)));
readonly "emissive"?: (((Input_model_programDefinitions["Color"])) | ((null)));
readonly "ids": (Input_model_programDefinitions["Ids"]);
readonly "iridescence"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "map": (string);
/** Model units spanned by the image; only valid with map. */
readonly "mapSize"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "metalness"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "name"?: ((string) | (null));
readonly "op": (Input_model_programDefinitions["MaterialTag"]);
readonly "opacity"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "roughness"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "sheenColor"?: (((Input_model_programDefinitions["Color"])) | ((null)));
readonly "sheenRoughness"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "transmission"?: (((Input_model_programDefinitions["Number"])) | ((null)));
});
readonly "Join": ("round" | "miter" | "bevel") & (string);
readonly "JointKind": ("revolute" | "continuous" | "prismatic" | "fixed" | "floating") & (string);
readonly "JointPose": ({
readonly "joint": (string);
readonly "op": (Input_model_programDefinitions["PoseTag"]);
readonly "value": (Input_model_programDefinitions["Number"]);
});
readonly "Keep": ("above" | "below") & (string);
readonly "Material": ((((Input_model_programDefinitions["PlainMaterial"])) & { readonly "map"?: never; readonly "mapSize"?: never; readonly "surface"?: never; }) | (((Input_model_programDefinitions["ImageMaterial"])) & { readonly "surface"?: never; }) | (((Input_model_programDefinitions["PaintedMaterial"])) & { readonly "map"?: never; readonly "mapSize"?: never; }));
readonly "MaterialTag": ("material") & (string);
readonly "Mimic": ({
readonly "joint": (string);
readonly "multiplier"?: (((Input_model_programDefinitions["Number"])) | ((null)));
});
readonly "Mirror": ({
readonly "axis"?: (((Input_model_programDefinitions["MirrorAxis"])) | ((null)));
readonly "profile": (Input_model_programDefinitions["Profile"]);
});
readonly "MirrorAxis": ("x" | "y") & (string);
readonly "ModelPath2": (readonly [(ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 2 }), ...((Input_model_programDefinitions["Piece"]))[]]);
readonly "ModelPath3": (readonly [(ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 }), ...((Input_model_programDefinitions["Piece2"]))[]]);
readonly "Morph": ((((Input_model_programDefinitions["TargetMorph"])) & { readonly "deltas"?: never; readonly "offset"?: never; readonly "regions"?: never; readonly "scale"?: never; }) | (((Input_model_programDefinitions["DeltaMorph"])) & { readonly "offset"?: never; readonly "regions"?: never; readonly "scale"?: never; readonly "target"?: never; }) | (((Input_model_programDefinitions["RegionMorph"])) & { readonly "deltas"?: never; readonly "target"?: never; }) | (((Input_model_programDefinitions["OffsetMorph"])) & { readonly "deltas"?: never; readonly "regions"?: never; readonly "target"?: never; }) | (((Input_model_programDefinitions["ScaleMorph"])) & { readonly "deltas"?: never; readonly "offset"?: never; readonly "regions"?: never; readonly "target"?: never; }));
readonly "MorphTag": ("morph") & (string);
readonly "Names": (((string)) | ((ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))));
readonly "Noise": ("fbm" | "ridged" | "crater") & (string);
/** A finite scalar or a parameter expression containing `$`. The kernel checks
grammar, declared names and finite results before executing any operation. */
readonly "Number": (((number)) | ((string)));
readonly "Offset": ({
readonly "distance"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "profile": (Input_model_programDefinitions["Profile"]);
});
readonly "OffsetMorph": ({
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "name": (string);
readonly "offset": (ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 });
readonly "op": (Input_model_programDefinitions["MorphTag"]);
readonly "scale"?: ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 }) | (null));
readonly "weight"?: (((Input_model_programDefinitions["Number"])) | ((null)));
});
readonly "Operation": (((Input_model_programDefinitions["BasicOperation"])) | ((Input_model_programDefinitions["Extrusion"])) | ((Input_model_programDefinitions["Material"])) | ((Input_model_programDefinitions["Pose"])) | ((Input_model_programDefinitions["Skin"])) | ((Input_model_programDefinitions["Morph"])));
readonly "PaintedMaterial": ({
readonly "clearcoat"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "clearcoatRoughness"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "color"?: (((Input_model_programDefinitions["Color"])) | ((null)));
readonly "emissive"?: (((Input_model_programDefinitions["Color"])) | ((null)));
readonly "ids": (Input_model_programDefinitions["Ids"]);
readonly "iridescence"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "metalness"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "name"?: ((string) | (null));
readonly "op": (Input_model_programDefinitions["MaterialTag"]);
readonly "opacity"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "roughness"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "sheenColor"?: (((Input_model_programDefinitions["Color"])) | ((null)));
readonly "sheenRoughness"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "surface": (Input_model_programDefinitions["Surface"]);
readonly "transmission"?: (((Input_model_programDefinitions["Number"])) | ((null)));
});
readonly "Path3": (((Input_model_programDefinitions["ModelPath3"])) | ((Input_model_programDefinitions["HelixPath"])));
readonly "PathSpline": ({
readonly "points": (ReadonlyArray<(ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "PathSpline2": ({
readonly "points": (ReadonlyArray<(ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Pattern": ("noise" | "ridged" | "craters" | "strata" | "stripes" | "panels" | "rivets" | "veins" | "edges" | "cavities" | "facing" | "gradient") & (string);
readonly "Piece": (((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 2 })) | ((Input_model_programDefinitions["CurvePiece"])));
readonly "Piece2": (((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 })) | ((Input_model_programDefinitions["CurvePiece2"])));
readonly "Pivot": (((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 })) | ((Input_model_programDefinitions["Frame"])));
readonly "PlacedSection": ({
readonly "at": (Input_model_programDefinitions["Frame"]);
readonly "profile": (Input_model_programDefinitions["Profile"]);
});
readonly "PlainMaterial": ({
readonly "clearcoat"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "clearcoatRoughness"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "color"?: (((Input_model_programDefinitions["Color"])) | ((null)));
readonly "emissive"?: (((Input_model_programDefinitions["Color"])) | ((null)));
readonly "ids": (Input_model_programDefinitions["Ids"]);
readonly "iridescence"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "metalness"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "name"?: ((string) | (null));
readonly "op": (Input_model_programDefinitions["MaterialTag"]);
readonly "opacity"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "roughness"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "sheenColor"?: (((Input_model_programDefinitions["Color"])) | ((null)));
readonly "sheenRoughness"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "transmission"?: (((Input_model_programDefinitions["Number"])) | ((null)));
});
readonly "Poly": ({
readonly "points": (ReadonlyArray<(Input_model_programDefinitions["PolyPoint"])> | (number extends ((Input_model_programDefinitions["PolyPoint"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "PolyPoint": (((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 2 })) | ((Input_model_programDefinitions["Corner"])));
readonly "Polygon": ({
readonly "flats"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "radius"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "sides"?: (((Input_model_programDefinitions["Count"])) | ((null)));
});
readonly "Pose": ((((Input_model_programDefinitions["JointPose"])) & { readonly "clip"?: never; readonly "time"?: never; }) | (((Input_model_programDefinitions["ClipPose"])) & { readonly "joint"?: never; readonly "value"?: never; }));
readonly "PoseKey": ({
readonly "pose": (Readonly<Record<string, (Input_model_programDefinitions["Number"])>>);
/** Seconds, strictly increasing across keys (domain validation). */
readonly "time": (Input_model_programDefinitions["Number"]);
});
readonly "PoseTag": ("pose") & (string);
readonly "Profile": (((({
readonly "rect": (Input_model_programDefinitions["Rect"]);
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "translate"?: never; readonly "union"?: never; }) | ((({
readonly "circle": (Input_model_programDefinitions["Circle"]);
})) & { readonly "ellipse"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "translate"?: never; readonly "union"?: never; }) | ((({
readonly "ellipse": (Input_model_programDefinitions["Ellipse"]);
})) & { readonly "circle"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "translate"?: never; readonly "union"?: never; }) | ((({
readonly "slot": (Input_model_programDefinitions["Slot"]);
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "translate"?: never; readonly "union"?: never; }) | ((({
readonly "polygon": (Input_model_programDefinitions["Polygon"]);
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "translate"?: never; readonly "union"?: never; }) | ((({
readonly "poly": (Input_model_programDefinitions["Poly"]);
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "translate"?: never; readonly "union"?: never; }) | ((({
readonly "gear": (Input_model_programDefinitions["Gear"]);
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "translate"?: never; readonly "union"?: never; }) | ((({
readonly "union": (ReadonlyArray<(Input_model_programDefinitions["Profile"])> | (number extends ((Input_model_programDefinitions["Profile"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "translate"?: never; }) | ((({
readonly "subtract": (ReadonlyArray<(Input_model_programDefinitions["Profile"])> | (number extends ((Input_model_programDefinitions["Profile"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "translate"?: never; readonly "union"?: never; }) | ((({
readonly "intersect": (ReadonlyArray<(Input_model_programDefinitions["Profile"])> | (number extends ((Input_model_programDefinitions["Profile"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "translate"?: never; readonly "union"?: never; }) | ((({
readonly "offset": (Input_model_programDefinitions["Offset"]);
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "translate"?: never; readonly "union"?: never; }) | ((({
readonly "translate": (Input_model_programDefinitions["Translate"]);
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "union"?: never; }) | ((({
readonly "rotate": (Input_model_programDefinitions["Rotate"]);
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "translate"?: never; readonly "union"?: never; }) | ((({
readonly "scale": (Input_model_programDefinitions["Scale"]);
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "translate"?: never; readonly "union"?: never; }) | ((({
readonly "superellipse": (Input_model_programDefinitions["Superellipse"]);
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "text"?: never; readonly "translate"?: never; readonly "union"?: never; }) | ((({
readonly "spline": (Input_model_programDefinitions["Spline"]);
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "translate"?: never; readonly "union"?: never; }) | ((({
readonly "path": (Input_model_programDefinitions["ModelPath2"]);
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "translate"?: never; readonly "union"?: never; }) | ((({
readonly "stroke": (Input_model_programDefinitions["Stroke"]);
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "translate"?: never; readonly "union"?: never; }) | ((({
readonly "hull": (ReadonlyArray<(Input_model_programDefinitions["Profile"])> | (number extends ((Input_model_programDefinitions["Profile"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "gear"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "translate"?: never; readonly "union"?: never; }) | ((({
readonly "mirror": (Input_model_programDefinitions["Mirror"]);
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "text"?: never; readonly "translate"?: never; readonly "union"?: never; }) | ((({
readonly "text": (Input_model_programDefinitions["Text"]);
})) & { readonly "circle"?: never; readonly "ellipse"?: never; readonly "gear"?: never; readonly "hull"?: never; readonly "intersect"?: never; readonly "mirror"?: never; readonly "offset"?: never; readonly "path"?: never; readonly "poly"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotate"?: never; readonly "scale"?: never; readonly "slot"?: never; readonly "spline"?: never; readonly "stroke"?: never; readonly "subtract"?: never; readonly "superellipse"?: never; readonly "translate"?: never; readonly "union"?: never; }));
readonly "Rect": ({
readonly "height"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "radius"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "width"?: (((Input_model_programDefinitions["Number"])) | ((null)));
});
readonly "RegionMorph": ({
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "name": (string);
readonly "offset"?: ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 }) | (null));
readonly "op": (Input_model_programDefinitions["MorphTag"]);
readonly "regions": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "scale"?: ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 }) | (null));
readonly "weight"?: (((Input_model_programDefinitions["Number"])) | ((null)));
});
readonly "RegionSkin": ({
readonly "blend"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "bones": (ReadonlyArray<(Input_model_programDefinitions["Bone"])> | (number extends ((Input_model_programDefinitions["Bone"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "maxEdge"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "op": (Input_model_programDefinitions["SkinTag"]);
});
readonly "Rotate": ({
readonly "angle"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "profile": (Input_model_programDefinitions["Profile"]);
});
readonly "Scale": ({
readonly "profile": (Input_model_programDefinitions["Profile"]);
readonly "x"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "y"?: (((Input_model_programDefinitions["Number"])) | ((null)));
});
readonly "ScaleMorph": ({
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "name": (string);
readonly "op": (Input_model_programDefinitions["MorphTag"]);
readonly "scale": (ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 });
readonly "weight"?: (((Input_model_programDefinitions["Number"])) | ((null)));
});
readonly "Sections": (((ReadonlyArray<(Input_model_programDefinitions["HeightSection"])> | (number extends ((Input_model_programDefinitions["HeightSection"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))) | ((ReadonlyArray<(Input_model_programDefinitions["PlacedSection"])> | (number extends ((Input_model_programDefinitions["PlacedSection"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))));
readonly "SignedAxis": ("+X" | "-X" | "+Y" | "-Y" | "+Z" | "-Z") & (string);
readonly "Skin": ((((Input_model_programDefinitions["RegionSkin"])) & { readonly "joints"?: never; }) | (((Input_model_programDefinitions["HeatSkin"])) & { readonly "blend"?: never; readonly "bones"?: never; readonly "maxEdge"?: never; }));
readonly "SkinJoint": ({
readonly "at": (ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 });
readonly "name": (string);
readonly "parent"?: ((string) | (null));
});
readonly "SkinTag": ("skin") & (string);
readonly "Slot": ({
readonly "length"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "width"?: (((Input_model_programDefinitions["Number"])) | ((null)));
});
readonly "Spline": ({
readonly "points": (ReadonlyArray<(ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Stroke": ({
readonly "closed"?: ((boolean) | (null));
readonly "join"?: (((Input_model_programDefinitions["Join"])) | ((null)));
readonly "path": (Input_model_programDefinitions["ModelPath2"]);
readonly "roundCaps"?: ((boolean) | (null));
readonly "width"?: (((Input_model_programDefinitions["Number"])) | ((null)));
});
readonly "Superellipse": ({
readonly "exponent"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "rx"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "ry"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "segments"?: (((Input_model_programDefinitions["Count"])) | ((null)));
});
readonly "Surface": ({
/** Texels per model unit. */
readonly "density"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "layers"?: (ReadonlyArray<(Input_model_programDefinitions["SurfaceLayer"])> | (number extends ((Input_model_programDefinitions["SurfaceLayer"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "normalScale"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "seed"?: (((Input_model_programDefinitions["Count"])) | ((null)));
});
readonly "SurfaceLayer": ({
readonly "amount"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "axis"?: (((Input_model_programDefinitions["Direction"])) | ((null)));
readonly "breakup"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "cell"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "color"?: (((Input_model_programDefinitions["Color"])) | ((null)));
readonly "emissive"?: (((Input_model_programDefinitions["Color"])) | ((null)));
readonly "glow"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "inset"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "jitter"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "metalness"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "octaves"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "pattern": (Input_model_programDefinitions["Pattern"]);
readonly "relief"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "roughness"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "scale"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "seed"?: (((Input_model_programDefinitions["Count"])) | ((null)));
readonly "softness"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "threshold"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "width"?: (((Input_model_programDefinitions["Number"])) | ((null)));
});
readonly "TangentArc": ({
readonly "to": (ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 2 });
});
readonly "TangentArc2": ({
readonly "to": (ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 3 });
});
readonly "TargetMorph": ({
readonly "id": (Input_model_programDefinitions["Id"]);
readonly "name": (string);
readonly "op": (Input_model_programDefinitions["MorphTag"]);
readonly "target": (Input_model_programDefinitions["Id"]);
readonly "weight"?: (((Input_model_programDefinitions["Number"])) | ((null)));
});
readonly "Text": ({
readonly "font": (string);
readonly "size"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "spacing"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "text": (string);
});
readonly "Translate": ({
readonly "profile": (Input_model_programDefinitions["Profile"]);
readonly "x"?: (((Input_model_programDefinitions["Number"])) | ((null)));
readonly "y"?: (((Input_model_programDefinitions["Number"])) | ((null)));
});
readonly "Unit": ("mm" | "cm" | "m") & (string);
readonly "Until": ({
readonly "face": (string);
readonly "part": (Input_model_programDefinitions["Id"]);
});
readonly "View": (((Input_model_programDefinitions["Number"])) | ((ReadonlyArray<(ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))) | ((Input_model_programDefinitions["ViewCurve"])));
readonly "ViewCurve": ({
readonly "points": (ReadonlyArray<(ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(Input_model_programDefinitions["Number"])> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "smooth"?: ((boolean) | (null));
});
}
export type Input_motion_action = ({
readonly "clip": (string);
readonly "hand"?: (((Input_motion_actionDefinitions["Hand"])) | ((null)));
/** In the head's local frame. */
readonly "handTarget"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
/** Must be greater than holdStart (domain validation). */
readonly "holdEnd": (number);
readonly "holdStart"?: (number);
readonly "rig"?: (((Input_motion_actionDefinitions["ContentRef"])) | ((null)));
readonly "source": (Input_motion_actionDefinitions["ContentRef"]);
});
export interface Input_motion_actionDefinitions {
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Hand": ("left" | "right") & (string);
}
export type Input_navmesh = ({
/** Ordered profiles; unnamed profile is the default, otherwise the first. */
readonly "agents"?: (ReadonlyArray<(Input_navmeshDefinitions["Agent"])> | (number extends ((Input_navmeshDefinitions["Agent"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Later shapes override earlier overlaps. */
readonly "areas"?: (ReadonlyArray<(Input_navmeshDefinitions["Area"])> | (number extends ((Input_navmeshDefinitions["Area"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "bounds"?: (((Input_navmeshDefinitions["Bounds"])) | ((null)));
readonly "boxes"?: (ReadonlyArray<(Input_navmeshDefinitions["NavBox"])> | (number extends ((Input_navmeshDefinitions["NavBox"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "cellHeight"?: ((number) | (null));
/** Voxel dimensions in metres; omitted: 0.2 horizontally, 0.1 vertically. */
readonly "cellSize"?: ((number) | (null));
readonly "links"?: (ReadonlyArray<(Input_navmeshDefinitions["Link"])> | (number extends ((Input_navmeshDefinitions["Link"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Contour simplification error; omitted: 1.3. */
readonly "maxError"?: ((number) | (null));
readonly "models"?: (ReadonlyArray<(Input_navmeshDefinitions["NavModel"])> | (number extends ((Input_navmeshDefinitions["NavModel"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "scene"?: (((Input_navmeshDefinitions["ContentRef"])) | ((null)));
readonly "terrains"?: (ReadonlyArray<(Input_navmeshDefinitions["NavTerrain"])> | (number extends ((Input_navmeshDefinitions["NavTerrain"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Clamped to 8..256 and truncated by normalization; omitted: 32. */
readonly "tileSize"?: ((number) | (null));
});
export interface Input_navmeshDefinitions {
readonly "Agent": ({
readonly "climb"?: ((number) | (null));
readonly "height"?: ((number) | (null));
readonly "name"?: ((string) | (null));
/** Metres; omitted radius/height/climb: 0.4/1.8/0.4. */
readonly "radius"?: ((number) | (null));
/** Maximum walkable slope; omitted: 45 degrees. */
readonly "slope"?: ((number) | (null));
});
readonly "Area": ({
readonly "id": (number);
readonly "shape": (Input_navmeshDefinitions["AreaShape"]);
});
/** Explicit branch selection replaces the parser's radius/point precedence. */
readonly "AreaShape": (((({
readonly "center"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "kind": ("box") & (string);
readonly "size"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "yaw"?: ((number) | (null));
})) & { readonly "bottom"?: never; readonly "height"?: never; readonly "points"?: never; readonly "radius"?: never; readonly "top"?: never; }) | ((({
readonly "center"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "height"?: ((number) | (null));
readonly "kind": ("cylinder") & (string);
/** Metres; negative values clamp to zero during normalization. */
readonly "radius": (number);
})) & { readonly "bottom"?: never; readonly "points"?: never; readonly "size"?: never; readonly "top"?: never; readonly "yaw"?: never; }) | ((({
readonly "bottom"?: ((number) | (null));
readonly "kind": ("convex") & (string);
/** x/z coordinates in metres; vertical bounds are bottom/top. */
readonly "points": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "top"?: ((number) | (null));
})) & { readonly "center"?: never; readonly "height"?: never; readonly "radius"?: never; readonly "size"?: never; readonly "yaw"?: never; }));
readonly "Bounds": ({
readonly "max"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "min"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
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
readonly "Link": ({
readonly "bidirectional"?: ((boolean) | (null));
/** Negative cost/duration clamp to zero; defaults are zero. */
readonly "cost"?: ((number) | (null));
readonly "duration"?: ((number) | (null));
readonly "end": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "id": (number);
readonly "kind"?: (((Input_navmeshDefinitions["LinkKind"])) | ((null)));
readonly "open"?: ((boolean) | (null));
readonly "start": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "traverse"?: (((Input_navmeshDefinitions["Traverse"])) | ((null)));
});
readonly "LinkKind": ("jump" | "ladder" | "door" | "teleport") & (string);
readonly "NavBox": ({
readonly "center"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
/** Normalized by the domain; zero-length rotations are invalid. */
readonly "quaternion"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "size"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
});
readonly "NavModel": ({
readonly "model": (Input_navmeshDefinitions["ContentRef"]);
readonly "transform"?: (((Input_navmeshDefinitions["Reflected_og_cook::cook::source_schemas::world::geometry::Transform"])) | ((null)));
});
readonly "NavTerrain": ({
readonly "position"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
/** Sampling stride; normalization clamps to >=1 and truncates; omitted: 1. */
readonly "step"?: ((number) | (null));
readonly "terrain": (Input_navmeshDefinitions["ContentRef"]);
});
readonly "Reflected_og_cook::cook::source_schemas::world::geometry::Transform": ({
/** The position, as three.js Object3D.position. */
readonly "position"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The rotation, as three.js Object3D.quaternion. */
readonly "quaternion"?: (ReadonlyArray<(number)> & { readonly length: 4 });
/** The scale, as three.js Object3D.scale. */
readonly "scale"?: (ReadonlyArray<(number)> & { readonly length: 3 });
});
readonly "Traverse": ("engine" | "game") & (string);
}
/** One style layer. Class resolution, CSS parsing, scheme-pair semantics,
slot names and referenced content are checked by the pack domain validator. */
export type Input_pack = ({
/** Ignored author annotation, deliberately arbitrary JSON data. */
readonly "about"?: unknown;
readonly "contrast"?: (((Input_packDefinitions["Theme"])) | ((null)));
/** Ordered font faces; family, weight and style belong to each face. */
readonly "fonts"?: (ReadonlyArray<(Input_packDefinitions["ContentRef"])> | (number extends ((Input_packDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "haptics"?: (Readonly<Record<string, (Input_packDefinitions["HapticBinding"])>>);
readonly "iconSet"?: (((Input_packDefinitions["ContentRef"])) | ((null)));
readonly "icons"?: (Readonly<Record<string, (Input_packDefinitions["ContentRef"])>>);
/** Named nine-slice art; names are kebab-case. */
readonly "images"?: (Readonly<Record<string, (Input_packDefinitions["Adaptive11"])>>);
readonly "scheme"?: (((Input_packDefinitions["Scheme"])) | ((null)));
readonly "seeds"?: (((Input_packDefinitions["Seeds"])) | ((null)));
/** An event maps to a sound; a Component.slot[.variant] maps to events. */
readonly "sounds"?: (Readonly<Record<string, (Input_packDefinitions["SoundBinding"])>>);
readonly "theme"?: (((Input_packDefinitions["Theme"])) | ((null)));
/** Component.slot[.variant] -> class string; empty declares the slot. */
readonly "variants"?: (Readonly<Record<string, (string)>>);
});
export interface Input_packDefinitions {
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive": (((Input_packDefinitions["Schemes"])) | ((string)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive10": (((Input_packDefinitions["Schemes10"])) | ((ReadonlyArray<(number)> & { readonly length: 4 })));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive11": ((((Input_packDefinitions["Schemes11"])) & { readonly "fill"?: never; readonly "repeat"?: never; readonly "slice"?: never; readonly "src"?: never; readonly "tint"?: never; readonly "width"?: never; }) | (((Input_packDefinitions["NineSlice"])) & { readonly "dark"?: never; readonly "light"?: never; }));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive12": ((((Input_packDefinitions["Schemes12"])) & { readonly "$ogRef"?: never; }) | (((Input_packDefinitions["ContentRef"])) & { readonly "dark"?: never; readonly "light"?: never; }));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive13": (((Input_packDefinitions["Schemes13"])) | ((Input_packDefinitions["Insets"])));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive14": (((Input_packDefinitions["Schemes14"])) | ((boolean)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive15": (((Input_packDefinitions["Schemes15"])) | ((Input_packDefinitions["Repeat"])));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive2": (((Input_packDefinitions["Schemes2"])) | ((Readonly<Record<string, (Input_packDefinitions["Adaptive3"])>>)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive3": (((Input_packDefinitions["Schemes3"])) | ((Input_packDefinitions["Palette"])));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive4": (((Input_packDefinitions["Schemes4"])) | ((number)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive5": (((Input_packDefinitions["Schemes5"])) | ((Readonly<Record<string, (Input_packDefinitions["Adaptive4"])>>)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive6": (((Input_packDefinitions["Schemes6"])) | ((Readonly<Record<string, (Input_packDefinitions["Adaptive7"])>>)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive7": (((Input_packDefinitions["Schemes7"])) | ((Input_packDefinitions["FontSize"])));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive8": (((Input_packDefinitions["Schemes8"])) | ((Readonly<Record<string, (Input_packDefinitions["Adaptive"])>>)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive9": (((Input_packDefinitions["Schemes9"])) | ((Readonly<Record<string, (Input_packDefinitions["Adaptive10"])>>)));
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
readonly "back"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "cancel"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "close"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "confirm"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "countdown"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "error"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "hover"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "levelup"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "notify"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "open"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "press"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "purchase"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "reward"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "select"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "step"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "swipe"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "tab"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "tick"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "toggle"?: (((Input_packDefinitions["Sound"])) | ((null)));
readonly "unlock"?: (((Input_packDefinitions["Sound"])) | ((null)));
});
/** The same interaction event vocabulary for global and slot-local values. */
readonly "Events2": ({
readonly "back"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "cancel"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "close"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "confirm"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "countdown"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "error"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "hover"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "levelup"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "notify"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "open"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "press"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "purchase"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "reward"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "select"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "step"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "swipe"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "tab"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "tick"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "toggle"?: (((Input_packDefinitions["Haptic"])) | ((null)));
readonly "unlock"?: (((Input_packDefinitions["Haptic"])) | ((null)));
});
readonly "FontSize": (((number)) | ((ReadonlyArray<(number)> & { readonly length: 2 })));
readonly "Haptic": ("tick" | "tap" | "thud" | "double" | "buzz" | "success" | "warning" | "error" | "none") & (string);
readonly "HapticBinding": (((Input_packDefinitions["Haptic"])) | ((Input_packDefinitions["Events2"])));
readonly "Insets": (((Input_packDefinitions["Nonnegative"])) | ((ReadonlyArray<(Input_packDefinitions["Nonnegative"])> & { readonly length: 4 })));
readonly "NineSlice": ({
/** Unset paints the centre. */
readonly "fill"?: (((Input_packDefinitions["Adaptive14"])) | ((null)));
readonly "repeat"?: (((Input_packDefinitions["Adaptive15"])) | ((null)));
readonly "slice": (Input_packDefinitions["Adaptive13"]);
readonly "src": (Input_packDefinitions["Adaptive12"]);
readonly "tint"?: (((Input_packDefinitions["Adaptive14"])) | ((null)));
/** Unset draws at the slice's own widths, in points. */
readonly "width"?: (((Input_packDefinitions["Adaptive13"])) | ((null)));
});
readonly "Nonnegative": (number);
readonly "Palette": (((string)) | ((Readonly<Record<string, (Input_packDefinitions["Adaptive"])>>)));
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
readonly "dark": (Input_packDefinitions["NineSlice"]);
readonly "light": (Input_packDefinitions["NineSlice"]);
});
readonly "Schemes12": ({
readonly "dark": (Input_packDefinitions["ContentRef"]);
readonly "light": (Input_packDefinitions["ContentRef"]);
});
readonly "Schemes13": ({
readonly "dark": (Input_packDefinitions["Insets"]);
readonly "light": (Input_packDefinitions["Insets"]);
});
readonly "Schemes14": ({
readonly "dark": (boolean);
readonly "light": (boolean);
});
readonly "Schemes15": ({
readonly "dark": (Input_packDefinitions["Repeat"]);
readonly "light": (Input_packDefinitions["Repeat"]);
});
readonly "Schemes2": ({
readonly "dark": (Readonly<Record<string, (Input_packDefinitions["Adaptive3"])>>);
readonly "light": (Readonly<Record<string, (Input_packDefinitions["Adaptive3"])>>);
});
readonly "Schemes3": ({
readonly "dark": (Input_packDefinitions["Palette"]);
readonly "light": (Input_packDefinitions["Palette"]);
});
readonly "Schemes4": ({
readonly "dark": (number);
readonly "light": (number);
});
readonly "Schemes5": ({
readonly "dark": (Readonly<Record<string, (Input_packDefinitions["Adaptive4"])>>);
readonly "light": (Readonly<Record<string, (Input_packDefinitions["Adaptive4"])>>);
});
readonly "Schemes6": ({
readonly "dark": (Readonly<Record<string, (Input_packDefinitions["Adaptive7"])>>);
readonly "light": (Readonly<Record<string, (Input_packDefinitions["Adaptive7"])>>);
});
readonly "Schemes7": ({
readonly "dark": (Input_packDefinitions["FontSize"]);
readonly "light": (Input_packDefinitions["FontSize"]);
});
readonly "Schemes8": ({
readonly "dark": (Readonly<Record<string, (Input_packDefinitions["Adaptive"])>>);
readonly "light": (Readonly<Record<string, (Input_packDefinitions["Adaptive"])>>);
});
readonly "Schemes9": ({
readonly "dark": (Readonly<Record<string, (Input_packDefinitions["Adaptive10"])>>);
readonly "light": (Readonly<Record<string, (Input_packDefinitions["Adaptive10"])>>);
});
readonly "Seeds": ({
readonly "accent"?: (((Input_packDefinitions["Adaptive"])) | ((null)));
readonly "danger"?: (((Input_packDefinitions["Adaptive"])) | ((null)));
readonly "hud"?: (((Input_packDefinitions["Adaptive"])) | ((null)));
readonly "info"?: (((Input_packDefinitions["Adaptive"])) | ((null)));
readonly "neutral"?: (((Input_packDefinitions["Adaptive"])) | ((null)));
readonly "primary"?: (((Input_packDefinitions["Adaptive"])) | ((null)));
readonly "success"?: (((Input_packDefinitions["Adaptive"])) | ((null)));
readonly "team"?: (((Input_packDefinitions["Adaptive"])) | ((null)));
readonly "warn"?: (((Input_packDefinitions["Adaptive"])) | ((null)));
});
readonly "Sound": ({
/** Empty clips silence an underlying layer only when volume is zero. */
readonly "clips"?: (ReadonlyArray<(Input_packDefinitions["ContentRef"])> | (number extends ((Input_packDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Playback rate or [low, high]; unset is 1. */
readonly "pitch"?: (((Input_packDefinitions["Range"])) | ((null)));
/** Gain or [low, high]; unset is 1. */
readonly "volume"?: (((Input_packDefinitions["Range"])) | ((null)));
});
readonly "SoundBinding": ((((Input_packDefinitions["Sound"])) & { readonly "back"?: never; readonly "cancel"?: never; readonly "close"?: never; readonly "confirm"?: never; readonly "countdown"?: never; readonly "error"?: never; readonly "hover"?: never; readonly "levelup"?: never; readonly "notify"?: never; readonly "open"?: never; readonly "press"?: never; readonly "purchase"?: never; readonly "reward"?: never; readonly "select"?: never; readonly "step"?: never; readonly "swipe"?: never; readonly "tab"?: never; readonly "tick"?: never; readonly "toggle"?: never; readonly "unlock"?: never; }) | (((Input_packDefinitions["Events"])) & { readonly "clips"?: never; readonly "pitch"?: never; readonly "volume"?: never; }));
/** NSH Tailwind theme sections, as consumed by Theme::from_json after scheme
splitting. Numerical scales are numbers, not arbitrary CSS/JSON values. */
readonly "Theme": ({
readonly "aspect"?: (((Input_packDefinitions["Adaptive5"])) | ((null)));
readonly "blur"?: (((Input_packDefinitions["Adaptive5"])) | ((null)));
readonly "breakpoints"?: (((Input_packDefinitions["Adaptive5"])) | ((null)));
readonly "colors"?: (((Input_packDefinitions["Adaptive2"])) | ((null)));
readonly "container"?: (((Input_packDefinitions["Adaptive5"])) | ((null)));
readonly "dropShadow"?: (((Input_packDefinitions["Adaptive8"])) | ((null)));
readonly "duration"?: (((Input_packDefinitions["Adaptive5"])) | ((null)));
/** Cubic Bezier control points [x1, y1, x2, y2]. */
readonly "ease"?: (((Input_packDefinitions["Adaptive9"])) | ((null)));
readonly "fontFamily"?: (((Input_packDefinitions["Adaptive8"])) | ((null)));
readonly "fontSize"?: (((Input_packDefinitions["Adaptive6"])) | ((null)));
readonly "fontWeight"?: (((Input_packDefinitions["Adaptive5"])) | ((null)));
readonly "insetShadow"?: (((Input_packDefinitions["Adaptive8"])) | ((null)));
readonly "leading"?: (((Input_packDefinitions["Adaptive5"])) | ((null)));
readonly "opacity"?: (((Input_packDefinitions["Adaptive5"])) | ((null)));
readonly "perspective"?: (((Input_packDefinitions["Adaptive5"])) | ((null)));
readonly "radius"?: (((Input_packDefinitions["Adaptive5"])) | ((null)));
/** Root em in points. */
readonly "rem"?: (((Input_packDefinitions["Adaptive4"])) | ((null)));
readonly "scale"?: (((Input_packDefinitions["Adaptive5"])) | ((null)));
/** CSS box-shadow values. */
readonly "shadow"?: (((Input_packDefinitions["Adaptive8"])) | ((null)));
/** Base spacing unit in points. */
readonly "spacing"?: (((Input_packDefinitions["Adaptive4"])) | ((null)));
readonly "stroke"?: (((Input_packDefinitions["Adaptive5"])) | ((null)));
/** CSS text-shadow values. */
readonly "textShadow"?: (((Input_packDefinitions["Adaptive8"])) | ((null)));
readonly "tracking"?: (((Input_packDefinitions["Adaptive5"])) | ((null)));
});
}
/** Emitter declaration. Rendering-mode relationships, curve ordering and
child-emitter cycles require the existing domain/graph validators. */
export type Input_particles = ({
readonly "align"?: (((Input_particlesDefinitions["Align"])) | ((null)));
readonly "angularVelocity"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "array"?: (((Input_particlesDefinitions["ParticleArray"])) | ((null)));
readonly "attractors"?: ((ReadonlyArray<(Input_particlesDefinitions["Attractor"])> | (number extends ((Input_particlesDefinitions["Attractor"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "blend"?: (((Input_particlesDefinitions["Blend"])) | ((null)));
readonly "bursts"?: ((ReadonlyArray<(Input_particlesDefinitions["Burst"])> | (number extends ((Input_particlesDefinitions["Burst"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "collision"?: (((Input_particlesDefinitions["Collision"])) | ((null)));
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "colorOverLife"?: (((Input_particlesDefinitions["ColorCurve"])) | ((null)));
readonly "distortion"?: ((number) | (null));
readonly "drag"?: ((number) | (null));
readonly "duration"?: ((number) | (null));
readonly "effect"?: (((Input_particlesDefinitions["ContentRef"])) | ((null)));
readonly "events"?: ((ReadonlyArray<(Input_particlesDefinitions["ParticleEvent"])> | (number extends ((Input_particlesDefinitions["ParticleEvent"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "field"?: (((Input_particlesDefinitions["ParticleField"])) | ((null)));
readonly "forceFields"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "gravity"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "heightOverLife"?: (((Input_particlesDefinitions["Curve"])) | ((null)));
readonly "inheritVelocity"?: ((number) | (null));
readonly "lifetime"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "lit"?: ((boolean) | (null));
readonly "looping"?: ((boolean) | (null));
readonly "maxParticles"?: ((number) | (null));
readonly "noise"?: (((Input_particlesDefinitions["ParticleNoise"])) | ((null)));
readonly "randomColorOverLife"?: (((Input_particlesDefinitions["ColorCurve"])) | ((null)));
readonly "rate"?: ((number) | (null));
readonly "rateOverTime"?: (((Input_particlesDefinitions["Curve"])) | ((null)));
readonly "render"?: (((Input_particlesDefinitions["ParticleRender"])) | ((null)));
readonly "rotation"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "roughness"?: ((number) | (null));
readonly "round"?: ((boolean) | (null));
readonly "shape"?: (((Input_particlesDefinitions["EmitterShape"])) | ((null)));
readonly "sheet"?: (((Input_particlesDefinitions["Sheet"])) | ((null)));
readonly "size"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "sizeOverLife"?: (((Input_particlesDefinitions["Curve"])) | ((null)));
readonly "softDistance"?: ((number) | (null));
readonly "space"?: (((Input_particlesDefinitions["Space"])) | ((null)));
readonly "speed"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "speedOverLife"?: (((Input_particlesDefinitions["Curve"])) | ((null)));
readonly "spinOverLife"?: (((Input_particlesDefinitions["Curve"])) | ((null)));
readonly "spread"?: ((number) | (null));
readonly "subEmitters"?: ((ReadonlyArray<(Input_particlesDefinitions["SubEmitter"])> | (number extends ((Input_particlesDefinitions["SubEmitter"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "texture"?: (((Input_particlesDefinitions["ContentRef"])) | ((null)));
readonly "trail"?: (((Input_particlesDefinitions["ParticleTrail"])) | ((null)));
readonly "translucency"?: ((number) | (null));
readonly "wind"?: ((number) | (null));
});
export interface Input_particlesDefinitions {
readonly "Align": (((({
readonly "kind": ("camera") & (string);
})) & { readonly "axis"?: never; readonly "normal"?: never; readonly "stretch"?: never; }) | ((({
readonly "kind": ("velocity") & (string);
readonly "stretch"?: ((number) | (null));
})) & { readonly "axis"?: never; readonly "normal"?: never; }) | ((({
readonly "axis": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "kind": ("upright") & (string);
})) & { readonly "normal"?: never; readonly "stretch"?: never; }) | ((({
readonly "kind": ("facing") & (string);
readonly "normal": (ReadonlyArray<(number)> & { readonly length: 3 });
})) & { readonly "axis"?: never; readonly "stretch"?: never; }));
readonly "Attractor": ({
readonly "position": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "radius"?: ((number) | (null));
readonly "strength": (number);
});
readonly "Blend": ("alpha" | "additive" | "multiply" | "subtractive") & (string);
readonly "Burst": ({
readonly "count": (number);
readonly "cycles"?: ((number) | (null));
readonly "interval"?: ((number) | (null));
readonly "time": (number);
});
readonly "Collision": ({
readonly "against"?: (((Input_particlesDefinitions["CollisionTarget"])) | ((null)));
readonly "bounce"?: ((number) | (null));
readonly "die"?: ((boolean) | (null));
readonly "friction"?: ((number) | (null));
});
readonly "CollisionTarget": ("depth" | "colliders") & (string);
readonly "ColorCurve": (ReadonlyArray<(readonly [(number), (ReadonlyArray<(number)> & { readonly length: 4 })])> | (number extends ((readonly [(number), (ReadonlyArray<(number)> & { readonly length: 4 })])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Curve": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "EmitterShape": (((({
readonly "kind": ("point") & (string);
})) & { readonly "angleDegrees"?: never; readonly "closed"?: never; readonly "halfExtents"?: never; readonly "mesh"?: never; readonly "points"?: never; readonly "radius"?: never; readonly "surface"?: never; }) | ((({
readonly "kind": ("sphere") & (string);
readonly "radius": (number);
readonly "surface"?: ((boolean) | (null));
})) & { readonly "angleDegrees"?: never; readonly "closed"?: never; readonly "halfExtents"?: never; readonly "mesh"?: never; readonly "points"?: never; }) | ((({
readonly "angleDegrees": (number);
readonly "kind": ("cone") & (string);
readonly "radius": (number);
})) & { readonly "closed"?: never; readonly "halfExtents"?: never; readonly "mesh"?: never; readonly "points"?: never; readonly "surface"?: never; }) | ((({
readonly "halfExtents": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "kind": ("box") & (string);
})) & { readonly "angleDegrees"?: never; readonly "closed"?: never; readonly "mesh"?: never; readonly "points"?: never; readonly "radius"?: never; readonly "surface"?: never; }) | ((({
readonly "closed"?: ((boolean) | (null));
readonly "kind": ("spline") & (string);
readonly "points": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "angleDegrees"?: never; readonly "halfExtents"?: never; readonly "mesh"?: never; readonly "radius"?: never; readonly "surface"?: never; }) | ((({
readonly "kind": ("mesh") & (string);
readonly "mesh": (Input_particlesDefinitions["MeshInput"]);
})) & { readonly "angleDegrees"?: never; readonly "closed"?: never; readonly "halfExtents"?: never; readonly "points"?: never; readonly "radius"?: never; readonly "surface"?: never; }));
readonly "MeshInput": (((Input_particlesDefinitions["ParticleMesh"])) | ((Input_particlesDefinitions["ContentRef"])));
readonly "ParticleArray": ({
readonly "asset": (Input_particlesDefinitions["ContentRef"]);
readonly "layers"?: ((number) | (null));
readonly "mode"?: (((Input_particlesDefinitions["SheetMode"])) | ((null)));
});
readonly "ParticleEvent": ("birth" | "death" | "collision") & (string);
readonly "ParticleField": (((({
readonly "frequency"?: ((number) | (null));
readonly "kind": ("curl") & (string);
readonly "scroll"?: ((number) | (null));
readonly "strength": (number);
})) & { readonly "asset"?: never; readonly "cells"?: never; readonly "size"?: never; readonly "vectors"?: never; }) | ((({
readonly "asset": (Input_particlesDefinitions["ContentRef"]);
readonly "kind": ("asset") & (string);
readonly "size": (number);
readonly "strength": (number);
})) & { readonly "cells"?: never; readonly "frequency"?: never; readonly "scroll"?: never; readonly "vectors"?: never; }) | ((({
readonly "cells": (number);
readonly "kind": ("grid") & (string);
readonly "size": (number);
readonly "strength": (number);
readonly "vectors": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "asset"?: never; readonly "frequency"?: never; readonly "scroll"?: never; }));
readonly "ParticleMesh": ("box" | "sphere" | "tetrahedron") & (string);
readonly "ParticleNoise": ({
readonly "frequency"?: ((number) | (null));
readonly "scroll"?: ((number) | (null));
readonly "strength": (number);
});
readonly "ParticleRender": (((({
readonly "kind": ("billboard") & (string);
})) & { readonly "layer"?: never; readonly "mesh"?: never; }) | ((({
readonly "kind": ("sprite") & (string);
readonly "layer"?: ((number) | (null));
})) & { readonly "mesh"?: never; }) | ((({
readonly "kind": ("mesh") & (string);
readonly "mesh": (Input_particlesDefinitions["MeshInput"]);
})) & { readonly "layer"?: never; }));
readonly "ParticleTrail": ({
readonly "head"?: ((boolean) | (null));
readonly "lifetime": (number);
readonly "width"?: ((number) | (null));
});
readonly "Sheet": ({
readonly "blend"?: ((boolean) | (null));
readonly "columns": (number);
readonly "frames"?: ((number) | (null));
readonly "mode"?: (((Input_particlesDefinitions["SheetMode"])) | ((null)));
readonly "rows": (number);
});
readonly "SheetMode": (((({
readonly "kind": ("off") & (string);
})) & { readonly "cycles"?: never; readonly "fps"?: never; }) | ((({
readonly "kind": ("random") & (string);
})) & { readonly "cycles"?: never; readonly "fps"?: never; }) | ((({
readonly "cycles"?: ((number) | (null));
readonly "kind": ("life") & (string);
})) & { readonly "fps"?: never; }) | ((({
readonly "fps": (number);
readonly "kind": ("fps") & (string);
})) & { readonly "cycles"?: never; }));
readonly "Space": ("world" | "local") & (string);
readonly "SubEmitter": ({
/** Required for event triggers, omitted for life emission. */
readonly "count"?: ((number) | (null));
readonly "emitter": (Input_particlesDefinitions["ContentRef"]);
readonly "inheritVelocity"?: ((number) | (null));
readonly "trigger": (Input_particlesDefinitions["Trigger"]);
});
readonly "Trigger": ("birth" | "death" | "collision" | "life") & (string);
}
export type Input_post = ({
/** WGSL defining fn post(f: PostInput, p: Params) -> vec4<f32>. */
readonly "code": (string);
readonly "params"?: (ReadonlyArray<(Input_postDefinitions["Param"])> | (number extends ((Input_postDefinitions["Param"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
export interface Input_postDefinitions {
readonly "Param": ((({
readonly "default"?: ((number) | (null));
readonly "kind": ("float") & (string);
readonly "name": (string);
})) | (({
readonly "default"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "kind": ("vec2") & (string);
readonly "name": (string);
})) | (({
readonly "default"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "kind": ("vec3") & (string);
readonly "name": (string);
})) | (({
readonly "default"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "kind": ("vec4") & (string);
readonly "name": (string);
})) | (({
readonly "default"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "kind": ("color") & (string);
readonly "name": (string);
})));
}
export type Input_quest = ({
readonly "category"?: ((string) | (null));
readonly "giver"?: ((string) | (null));
/** Quest variables always belong to the shared story, not a conversation. */
readonly "globals"?: (Readonly<Record<string, (Input_questDefinitions["Value"])>>);
readonly "id": (string);
readonly "region"?: ((string) | (null));
readonly "rewards"?: (ReadonlyArray<(Input_questDefinitions["Reward"])> | (number extends ((Input_questDefinitions["Reward"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "stages": (ReadonlyArray<(Input_questDefinitions["Stage"])> | (number extends ((Input_questDefinitions["Stage"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "summary"?: ((string) | (null));
readonly "title": (string);
});
export interface Input_questDefinitions {
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Expr": (((({
readonly "kind": ("literal") & (string);
readonly "value": (Input_questDefinitions["Value"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }) | ((({
readonly "kind": ("variable") & (string);
readonly "name": (string);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "id": (string);
readonly "kind": ("once") & (string);
})) & { readonly "args"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("not") & (string);
readonly "value": (Input_questDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }) | ((({
readonly "kind": ("negate") & (string);
readonly "value": (Input_questDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }) | ((({
readonly "kind": ("binary") & (string);
readonly "left": (Input_questDefinitions["Expr"]);
readonly "op": (Input_questDefinitions["Op"]);
readonly "right": (Input_questDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "quest"?: never; readonly "value"?: never; }) | ((({
readonly "args": (ReadonlyArray<(Input_questDefinitions["Expr"])> | (number extends ((Input_questDefinitions["Expr"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
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
readonly "value": (Input_questDefinitions["Value"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }));
readonly "Objective": ({
readonly "filter"?: (((Input_questDefinitions["Expr"])) | ((null)));
readonly "hidden"?: ((boolean) | (null));
readonly "id": (string);
readonly "marker"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "on"?: ((string) | (null));
readonly "optional"?: ((boolean) | (null));
readonly "target"?: ((number) | (null));
readonly "text": (Input_questDefinitions["Text"]);
readonly "when"?: (((Input_questDefinitions["Expr"])) | ((null)));
});
readonly "Op": ("add" | "sub" | "mul" | "div" | "rem" | "eq" | "ne" | "lt" | "le" | "gt" | "ge" | "and" | "or" | "xor") & (string);
readonly "Reward": ({
readonly "amount"?: ((number) | (null));
readonly "image"?: (((Input_questDefinitions["ContentRef"])) | ((null)));
readonly "name": (string);
readonly "rarity"?: ((string) | (null));
});
readonly "Stage": ({
readonly "fail"?: (((Input_questDefinitions["Expr"])) | ((null)));
readonly "id": (string);
readonly "next"?: ((string) | (null));
readonly "objectives"?: (ReadonlyArray<(Input_questDefinitions["Objective"])> | (number extends ((Input_questDefinitions["Objective"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "summary"?: ((string) | (null));
});
readonly "Text": ({
readonly "args"?: (Readonly<Record<string, (Input_questDefinitions["Expr"])>>);
/** Localization key, not the rendered sentence. */
readonly "key": (string);
});
readonly "Value": (((number)) | ((boolean)) | ((string)));
}
export type Input_rig = ({
/** Humanoid bone name to skeleton node name; null explicitly unmaps it. */
readonly "bones"?: ({
readonly "chest"?: ((string) | (null));
readonly "head"?: ((string) | (null));
readonly "hips"?: ((string) | (null));
readonly "leftFoot"?: ((string) | (null));
readonly "leftHand"?: ((string) | (null));
readonly "leftIndexDistal"?: ((string) | (null));
readonly "leftIndexIntermediate"?: ((string) | (null));
readonly "leftIndexProximal"?: ((string) | (null));
readonly "leftLittleDistal"?: ((string) | (null));
readonly "leftLittleIntermediate"?: ((string) | (null));
readonly "leftLittleProximal"?: ((string) | (null));
readonly "leftLowerArm"?: ((string) | (null));
readonly "leftLowerLeg"?: ((string) | (null));
readonly "leftMiddleDistal"?: ((string) | (null));
readonly "leftMiddleIntermediate"?: ((string) | (null));
readonly "leftMiddleProximal"?: ((string) | (null));
readonly "leftRingDistal"?: ((string) | (null));
readonly "leftRingIntermediate"?: ((string) | (null));
readonly "leftRingProximal"?: ((string) | (null));
readonly "leftShoulder"?: ((string) | (null));
readonly "leftThumbDistal"?: ((string) | (null));
readonly "leftThumbMetacarpal"?: ((string) | (null));
readonly "leftThumbProximal"?: ((string) | (null));
readonly "leftToes"?: ((string) | (null));
readonly "leftUpperArm"?: ((string) | (null));
readonly "leftUpperLeg"?: ((string) | (null));
readonly "neck"?: ((string) | (null));
readonly "rightFoot"?: ((string) | (null));
readonly "rightHand"?: ((string) | (null));
readonly "rightIndexDistal"?: ((string) | (null));
readonly "rightIndexIntermediate"?: ((string) | (null));
readonly "rightIndexProximal"?: ((string) | (null));
readonly "rightLittleDistal"?: ((string) | (null));
readonly "rightLittleIntermediate"?: ((string) | (null));
readonly "rightLittleProximal"?: ((string) | (null));
readonly "rightLowerArm"?: ((string) | (null));
readonly "rightLowerLeg"?: ((string) | (null));
readonly "rightMiddleDistal"?: ((string) | (null));
readonly "rightMiddleIntermediate"?: ((string) | (null));
readonly "rightMiddleProximal"?: ((string) | (null));
readonly "rightRingDistal"?: ((string) | (null));
readonly "rightRingIntermediate"?: ((string) | (null));
readonly "rightRingProximal"?: ((string) | (null));
readonly "rightShoulder"?: ((string) | (null));
readonly "rightThumbDistal"?: ((string) | (null));
readonly "rightThumbMetacarpal"?: ((string) | (null));
readonly "rightThumbProximal"?: ((string) | (null));
readonly "rightToes"?: ((string) | (null));
readonly "rightUpperArm"?: ((string) | (null));
readonly "rightUpperLeg"?: ((string) | (null));
readonly "spine"?: ((string) | (null));
readonly "upperChest"?: ((string) | (null));
});
/** Left and right sole heights. */
readonly "footHeight"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** Naming scheme understood by the humanoid mapper. */
readonly "scheme"?: ("mixamo" | "vrm" | "unity" | "rigify" | "ue" | "blender" | "soma" | "auto" | null) & ((string) | (null));
});
export type Input_rig2d = ((((Input_rig2dDefinitions["Authored"])) & { readonly "atlas"?: never; readonly "dragonbones"?: never; readonly "parts"?: never; readonly "pivots"?: never; readonly "template"?: never; }) | (((Input_rig2dDefinitions["Dragonbones"])) & { readonly "armatures"?: never; readonly "parts"?: never; readonly "pivots"?: never; readonly "rig"?: never; readonly "template"?: never; }) | (((Input_rig2dDefinitions["Parts"])) & { readonly "armatures"?: never; readonly "atlas"?: never; readonly "dragonbones"?: never; readonly "rig"?: never; readonly "texture"?: never; }));
export interface Input_rig2dDefinitions {
readonly "Armature": ({
readonly "attachments"?: (Readonly<Record<string, (Input_rig2dDefinitions["Attachment"])>>);
readonly "bones": (ReadonlyArray<(Input_rig2dDefinitions["Bone"])> | (number extends ((Input_rig2dDefinitions["Bone"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "clips"?: (ReadonlyArray<(Input_rig2dDefinitions["Clip"])> | (number extends ((Input_rig2dDefinitions["Clip"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "iks"?: (ReadonlyArray<(Input_rig2dDefinitions["Ik"])> | (number extends ((Input_rig2dDefinitions["Ik"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "paths"?: (ReadonlyArray<(Input_rig2dDefinitions["PathConstraint"])> | (number extends ((Input_rig2dDefinitions["PathConstraint"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Slot attachment swaps, names resolved in this armature. */
readonly "skins"?: (Readonly<Record<string, (Readonly<Record<string, ((string) | (null))>>)>>);
readonly "slots"?: (ReadonlyArray<(Input_rig2dDefinitions["Slot"])> | (number extends ((Input_rig2dDefinitions["Slot"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "transforms"?: (ReadonlyArray<(Input_rig2dDefinitions["TransformConstraint"])> | (number extends ((Input_rig2dDefinitions["TransformConstraint"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Attachment": (((({
readonly "at"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "kind": ("region") & (string);
readonly "rect": (ReadonlyArray<(number)> & { readonly length: 4 });
readonly "rotation"?: ((number) | (null));
readonly "size"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "skew"?: ((number) | (null));
readonly "slot": (string);
})) & { readonly "armature"?: never; readonly "clip"?: never; readonly "closed"?: never; readonly "constant"?: never; readonly "lengths"?: never; readonly "points"?: never; readonly "polygon"?: never; readonly "scale"?: never; readonly "triangles"?: never; readonly "vertices"?: never; }) | ((({
readonly "kind": ("mesh") & (string);
readonly "slot": (string);
readonly "triangles": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "vertices": (ReadonlyArray<(Input_rig2dDefinitions["Vertex"])> | (number extends ((Input_rig2dDefinitions["Vertex"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "armature"?: never; readonly "at"?: never; readonly "clip"?: never; readonly "closed"?: never; readonly "constant"?: never; readonly "lengths"?: never; readonly "points"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotation"?: never; readonly "scale"?: never; readonly "size"?: never; readonly "skew"?: never; }) | ((({
readonly "kind": ("box") & (string);
readonly "polygon": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "slot": (string);
})) & { readonly "armature"?: never; readonly "at"?: never; readonly "clip"?: never; readonly "closed"?: never; readonly "constant"?: never; readonly "lengths"?: never; readonly "points"?: never; readonly "rect"?: never; readonly "rotation"?: never; readonly "scale"?: never; readonly "size"?: never; readonly "skew"?: never; readonly "triangles"?: never; readonly "vertices"?: never; }) | ((({
readonly "closed": (boolean);
readonly "constant": (boolean);
readonly "kind": ("path") & (string);
readonly "lengths": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "points": (ReadonlyArray<(ReadonlyArray<(Input_rig2dDefinitions["Influence"])> | (number extends ((Input_rig2dDefinitions["Influence"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))> | (number extends ((ReadonlyArray<(Input_rig2dDefinitions["Influence"])> | (number extends ((Input_rig2dDefinitions["Influence"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "slot": (string);
})) & { readonly "armature"?: never; readonly "at"?: never; readonly "clip"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotation"?: never; readonly "scale"?: never; readonly "size"?: never; readonly "skew"?: never; readonly "triangles"?: never; readonly "vertices"?: never; }) | ((({
readonly "armature": (string);
readonly "at"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "clip"?: ((string) | (null));
readonly "kind": ("child") & (string);
readonly "rotation"?: ((number) | (null));
readonly "scale"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "skew"?: ((number) | (null));
readonly "slot": (string);
})) & { readonly "closed"?: never; readonly "constant"?: never; readonly "lengths"?: never; readonly "points"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "size"?: never; readonly "triangles"?: never; readonly "vertices"?: never; }));
readonly "Authored": ({
readonly "armatures"?: (Readonly<Record<string, (Input_rig2dDefinitions["Armature"])>>);
readonly "conform"?: (((Input_rig2dDefinitions["TilesetConform"])) | ((null)));
readonly "filter"?: (((Input_rig2dDefinitions["Filter"])) | ((null)));
readonly "rig": (Input_rig2dDefinitions["Armature"]);
readonly "texture": (Input_rig2dDefinitions["ContentRef"]);
readonly "unit"?: ((number) | (null));
});
readonly "Bone": ({
readonly "at"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "length"?: ((number) | (null));
readonly "name": (string);
readonly "parent"?: ((string) | (null));
readonly "rotation"?: ((number) | (null));
readonly "scale"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "skew"?: ((number) | (null));
});
readonly "BoneKeys": ({
readonly "bone": (string);
readonly "rotate"?: (ReadonlyArray<(Input_rig2dDefinitions["Key"])> | (number extends ((Input_rig2dDefinitions["Key"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "scale"?: (ReadonlyArray<(Input_rig2dDefinitions["Key3"])> | (number extends ((Input_rig2dDefinitions["Key3"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "skew"?: (ReadonlyArray<(Input_rig2dDefinitions["Key2"])> | (number extends ((Input_rig2dDefinitions["Key2"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "translate"?: (ReadonlyArray<(Input_rig2dDefinitions["Key3"])> | (number extends ((Input_rig2dDefinitions["Key3"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Clip": ({
readonly "bones"?: (ReadonlyArray<(Input_rig2dDefinitions["BoneKeys"])> | (number extends ((Input_rig2dDefinitions["BoneKeys"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "deforms"?: (Readonly<Record<string, (ReadonlyArray<(Input_rig2dDefinitions["Key6"])> | (number extends ((Input_rig2dDefinitions["Key6"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))>>);
readonly "duration": (number);
readonly "events"?: (ReadonlyArray<(Input_rig2dDefinitions["Key9"])> | (number extends ((Input_rig2dDefinitions["Key9"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "iks"?: (Readonly<Record<string, (ReadonlyArray<(Input_rig2dDefinitions["Key8"])> | (number extends ((Input_rig2dDefinitions["Key8"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))>>);
readonly "looping"?: ((boolean) | (null));
readonly "name": (string);
readonly "order"?: (ReadonlyArray<(Input_rig2dDefinitions["Key7"])> | (number extends ((Input_rig2dDefinitions["Key7"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "slots"?: (ReadonlyArray<(Input_rig2dDefinitions["SlotKeys"])> | (number extends ((Input_rig2dDefinitions["SlotKeys"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
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
readonly "Dragonbones": ({
/** Explicit atlas exchange text from input.text. */
readonly "atlas": (string);
readonly "conform"?: (((Input_rig2dDefinitions["TilesetConform"])) | ((null)));
/** Explicit DragonBones 5.5 exchange text from input.text. */
readonly "dragonbones": (string);
readonly "filter"?: (((Input_rig2dDefinitions["Filter"])) | ((null)));
readonly "texture": (Input_rig2dDefinitions["ContentRef"]);
readonly "unit"?: ((number) | (null));
});
readonly "Ease": (((({
readonly "kind": ("linear") & (string);
})) & { readonly "points"?: never; }) | ((({
readonly "kind": ("step") & (string);
})) & { readonly "points"?: never; }) | (({
readonly "kind": ("bezier") & (string);
readonly "points": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})));
readonly "Filter": ("nearest" | "linear") & (string);
readonly "Ik": ({
readonly "bones": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "counterclockwise"?: ((boolean) | (null));
readonly "name": (string);
readonly "target": (string);
readonly "weight"?: ((number) | (null));
});
readonly "IkValue": ({
readonly "counterclockwise": (boolean);
readonly "weight": (number);
});
readonly "Influence": ({
/** Bone-local pixel position, y up. */
readonly "at": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "bone": (string);
readonly "weight": (number);
});
readonly "Key": ({
readonly "ease"?: (((Input_rig2dDefinitions["Ease"])) | ((null)));
readonly "time": (number);
readonly "value": (Input_rig2dDefinitions["Rotation"]);
});
readonly "Key2": ({
readonly "ease"?: (((Input_rig2dDefinitions["Ease"])) | ((null)));
readonly "time": (number);
readonly "value": (number);
});
readonly "Key3": ({
readonly "ease"?: (((Input_rig2dDefinitions["Ease"])) | ((null)));
readonly "time": (number);
readonly "value": (ReadonlyArray<(number)> & { readonly length: 2 });
});
readonly "Key4": ({
readonly "ease"?: (((Input_rig2dDefinitions["Ease"])) | ((null)));
readonly "time": (number);
readonly "value"?: ((string) | (null));
});
readonly "Key5": ({
readonly "ease"?: (((Input_rig2dDefinitions["Ease"])) | ((null)));
readonly "time": (number);
readonly "value": (ReadonlyArray<(number)> & { readonly length: 4 });
});
readonly "Key6": ({
readonly "ease"?: (((Input_rig2dDefinitions["Ease"])) | ((null)));
readonly "time": (number);
readonly "value": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Key7": ({
readonly "ease"?: (((Input_rig2dDefinitions["Ease"])) | ((null)));
readonly "time": (number);
readonly "value": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Key8": ({
readonly "ease"?: (((Input_rig2dDefinitions["Ease"])) | ((null)));
readonly "time": (number);
readonly "value": (Input_rig2dDefinitions["IkValue"]);
});
readonly "Key9": ({
readonly "ease"?: (((Input_rig2dDefinitions["Ease"])) | ((null)));
readonly "time": (number);
readonly "value": (string);
});
readonly "Parts": ({
readonly "conform"?: (((Input_rig2dDefinitions["TilesetConform"])) | ((null)));
readonly "filter"?: (((Input_rig2dDefinitions["Filter"])) | ((null)));
/** Canonical module-relative PSD/PSB or PNG-parts directory under assets/
or sources/. Directory paths have no trailing slash; no sibling lookup. */
readonly "parts": (string);
readonly "pivots"?: (Readonly<Record<string, (ReadonlyArray<(number)> & { readonly length: 2 })>>);
readonly "template": (string);
readonly "unit"?: ((number) | (null));
});
readonly "PathConstraint": ({
readonly "bones": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "mix"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "name": (string);
readonly "path": (string);
readonly "position"?: ((number) | (null));
readonly "positionUnit"?: (((Input_rig2dDefinitions["PathUnit"])) | ((null)));
readonly "rotate"?: (((Input_rig2dDefinitions["PathRotate"])) | ((null)));
readonly "rotation"?: ((number) | (null));
readonly "spacing"?: ((number) | (null));
readonly "spacingMode"?: (((Input_rig2dDefinitions["PathSpacing"])) | ((null)));
});
readonly "PathRotate": ("tangent" | "chain" | "chainScale") & (string);
readonly "PathSpacing": ("fixed" | "percent" | "length") & (string);
readonly "PathUnit": ("fixed" | "percent") & (string);
readonly "Rotation": ({
readonly "degrees": (number);
readonly "turn"?: (((Input_rig2dDefinitions["Turn"])) | ((null)));
});
readonly "Slot": ({
readonly "attachment"?: ((string) | (null));
readonly "bone": (string);
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "name": (string);
readonly "offset"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
});
readonly "SlotKeys": ({
readonly "attachment"?: (ReadonlyArray<(Input_rig2dDefinitions["Key4"])> | (number extends ((Input_rig2dDefinitions["Key4"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "color"?: (ReadonlyArray<(Input_rig2dDefinitions["Key5"])> | (number extends ((Input_rig2dDefinitions["Key5"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "offset"?: (ReadonlyArray<(Input_rig2dDefinitions["Key5"])> | (number extends ((Input_rig2dDefinitions["Key5"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "slot": (string);
});
readonly "TilesetConform": (((({
readonly "kind": ("off") & (string);
})) & { readonly "size"?: never; }) | (({
readonly "kind": ("size") & (string);
readonly "size": (number);
})));
readonly "TransformConstraint": ({
readonly "at"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "bones": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "mix"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "name": (string);
readonly "rotation"?: ((number) | (null));
readonly "scale"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "shear"?: ((number) | (null));
readonly "target": (string);
});
readonly "Turn": (((({
readonly "kind": ("shorter") & (string);
})) & { readonly "turns"?: never; }) | (({
readonly "kind": ("clockwise") & (string);
readonly "turns": (number);
})) | (({
readonly "kind": ("counterclockwise") & (string);
readonly "turns": (number);
})));
readonly "Vertex": ({
readonly "influences": (ReadonlyArray<(Input_rig2dDefinitions["Influence"])> | (number extends ((Input_rig2dDefinitions["Influence"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "uv": (ReadonlyArray<(number)> & { readonly length: 2 });
});
}
/** Ordered instance transforms. Array counts, nonzero quaternions and
terrain surface/holes are checked when the scatter is expanded. */
export type Input_scatter = ({
/** Rotate up to the projected normal before the authored quaternion. */
readonly "align"?: ((boolean) | (null));
readonly "positions": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "project"?: (((Input_scatterDefinitions["ContentRef"])) | ((null)));
/** One xyzw quaternion per position; normalized by domain expansion. */
readonly "quaternions"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "scales"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "target": (Input_scatterDefinitions["ScatterTarget"]);
});
export interface Input_scatterDefinitions {
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "ScatterTarget": ((({
readonly "asset": (Input_scatterDefinitions["ContentRef"]);
readonly "kind": ("model") & (string);
})) | (({
readonly "asset": (Input_scatterDefinitions["ContentRef"]);
readonly "kind": ("prefab") & (string);
})));
}
/** Procedural sound: optional preset plus explicit oscillator/sample layers.
At least one preset/layer and maximum rendered duration are domain checks. */
export type Input_sfx = ({
readonly "layers"?: ((ReadonlyArray<(Input_sfxDefinitions["SynthLayer"])> | (number extends ((Input_sfxDefinitions["SynthLayer"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "length"?: ((number) | (null));
readonly "pitch"?: ((number) | (null));
readonly "preset"?: (((Input_sfxDefinitions["SfxPreset"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "volume"?: ((number) | (null));
});
export interface Input_sfxDefinitions {
readonly "Pcm": (number);
readonly "SampleInput": (((({
readonly "kind": ("file") & (string);
readonly "looping"?: ((boolean) | (null));
readonly "reference"?: ((number) | (null));
/** Explicit original audio input, relative to the owning module. */
readonly "source": (string);
})) & { readonly "rate"?: never; readonly "samples"?: never; }) | ((({
readonly "kind": ("pcm") & (string);
readonly "looping"?: ((boolean) | (null));
readonly "rate": (number);
readonly "reference"?: ((number) | (null));
readonly "samples": (ReadonlyArray<(Input_sfxDefinitions["Pcm"])> | (number extends ((Input_sfxDefinitions["Pcm"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "source"?: never; }));
readonly "SfxPreset": ("coin" | "pickup" | "powerup" | "jump" | "land" | "footstep" | "hit" | "hurt" | "explosion" | "laser" | "whoosh" | "door" | "death" | "click" | "select" | "back" | "error" | "levelup") & (string);
/** A mono sample replaces the oscillator; sample layers reject FM. */
readonly "SynthLayer": ({
readonly "arp"?: ((number) | (null));
readonly "arpAt"?: ((number) | (null));
readonly "attack"?: ((number) | (null));
readonly "decay"?: ((number) | (null));
readonly "delay"?: ((number) | (null));
readonly "drive"?: ((number) | (null));
readonly "duty"?: ((number) | (null));
readonly "fmFreq"?: ((number) | (null));
readonly "fmIndex"?: ((number) | (null));
readonly "freq"?: ((number) | (null));
readonly "freqEnd"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "highpass"?: ((number) | (null));
readonly "lowpass"?: ((number) | (null));
readonly "lpSlide"?: ((number) | (null));
readonly "punch"?: ((number) | (null));
readonly "q"?: ((number) | (null));
readonly "release"?: ((number) | (null));
readonly "sample"?: (((Input_sfxDefinitions["SampleInput"])) | ((null)));
readonly "slide"?: ((number) | (null));
readonly "sustain"?: ((number) | (null));
readonly "sustainLevel"?: ((number) | (null));
readonly "vibDepth"?: ((number) | (null));
readonly "vibRate"?: ((number) | (null));
readonly "wave"?: (((Input_sfxDefinitions["Wave"])) | ((null)));
});
readonly "Wave": ("sine" | "square" | "saw" | "triangle" | "noise") & (string);
}
export type Input_shader = ({
/** WGSL hooks or vertex/fragment entries; use input.text for external WGSL. */
readonly "code": (string);
readonly "mode"?: (((Input_shaderDefinitions["Mode"])) | ((null)));
/** Names, reserved identifiers and backend limits are checked by the domain. */
readonly "params"?: (ReadonlyArray<(Input_shaderDefinitions["Param"])> | (number extends ((Input_shaderDefinitions["Param"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
export interface Input_shaderDefinitions {
readonly "Mode": (((({
readonly "kind": ("hooks") & (string);
})) & { readonly "lit"?: never; }) | ((({
readonly "kind": ("sprite") & (string);
})) & { readonly "lit"?: never; }) | (({
readonly "kind": ("full") & (string);
readonly "lit": (boolean);
})));
readonly "Param": ((({
readonly "default"?: ((number) | (null));
readonly "kind": ("float") & (string);
readonly "name": (string);
})) | (({
readonly "default"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "kind": ("vec2") & (string);
readonly "name": (string);
})) | (({
readonly "default"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "kind": ("vec3") & (string);
readonly "name": (string);
})) | (({
readonly "default"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "kind": ("vec4") & (string);
readonly "name": (string);
})) | (({
readonly "default"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "kind": ("color") & (string);
readonly "name": (string);
})));
}
export type Input_sound = ({
/** One mono channel or interleaved stereo. */
readonly "channels": (number);
/** Sample rate in hertz; no resampling before the codec import. */
readonly "rate": (number);
/** Finite PCM samples, at least one complete interleaved frame. */
readonly "samples": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
export type Input_strings = ({
/** Reference locale for key/argument checks; omitted is false. */
readonly "base"?: ((boolean) | (null));
/** Omitted: direction of locale. */
readonly "direction"?: (((Input_stringsDefinitions["Direction"])) | ((null)));
/** ICU-subset source messages, not an embedded author configuration. */
readonly "entries": (Readonly<Record<string, (string)>>);
readonly "fallback"?: ((string) | (null));
/** Ordered locale font fallback chain. */
readonly "fonts"?: (ReadonlyArray<(Input_stringsDefinitions["ContentRef"])> | (number extends ((Input_stringsDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Canonicalized as a locale tag by the localization domain. */
readonly "locale": (string);
});
export interface Input_stringsDefinitions {
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Direction": ("ltr" | "rtl") & (string);
}
/** A terrain's source, processing and material declarations. Sample counts,
ascending bands, channels, layer indices and projected edits remain domain
validation; this does not run or replace the existing terrain cooker. */
export type Input_terrain = ({
readonly "albedoArray"?: (((Input_terrainDefinitions["ContentRef"])) | ((null)));
readonly "castShadows"?: ((boolean) | (null));
readonly "collision"?: ((boolean) | (null));
/** RGBA control images, one per four layers. Exclusive with biome rules. */
readonly "control"?: ((ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Brushes and stamps in their written order, after generation/processing. */
readonly "edits"?: ((ReadonlyArray<(Input_terrainDefinitions["TerrainEdit"])> | (number extends ((Input_terrainDefinitions["TerrainEdit"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "heightBlend"?: ((number) | (null));
/** Raw hole image relative to the owning module: black/transparent cuts cells. */
readonly "holes"?: ((string) | (null));
readonly "layers"?: ((ReadonlyArray<(Input_terrainDefinitions["TerrainLayerInput"])> | (number extends ((Input_terrainDefinitions["TerrainLayerInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "lod"?: (((Input_terrainDefinitions["TerrainLod"])) | ((null)));
readonly "normalArray"?: (((Input_terrainDefinitions["ContentRef"])) | ((null)));
readonly "process"?: (((Input_terrainDefinitions["TerrainProcess"])) | ((null)));
/** Shared seed for the noise source and seeded processing stages. */
readonly "seed"?: ((number) | (null));
/** Positive world extents along X and Z, in metres. */
readonly "size"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "source": (Input_terrainDefinitions["TerrainSource"]);
readonly "tile"?: ((number) | (null));
});
export interface Input_terrainDefinitions {
readonly "Arc": ({
readonly "center": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "clockwise"?: (boolean);
readonly "end": (number);
readonly "radius": (number);
readonly "start": (number);
});
readonly "BiomeInput": ({
readonly "altitude"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "altitudeFade"?: ((number) | (null));
readonly "layer": (number);
readonly "moisture"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "moistureFade"?: ((number) | (null));
readonly "slope"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "slopeFade"?: ((number) | (null));
readonly "weight"?: ((number) | (null));
});
readonly "BoxGeometry": ({
/** Z extent, metres; unset 1. */
readonly "depth"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Y extent, metres; unset 1. */
readonly "height"?: ((number) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** X extent, metres; unset 1. */
readonly "width"?: ((number) | (null));
});
readonly "BrushMode": ("raise" | "lower" | "flatten" | "smooth" | "paint" | "hole" | "fill") & (string);
readonly "BufferGeometry": ({
/** One linear RGB or RGBA colour per vertex. */
readonly "colors"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Counter-clockwise triangle indices; absent uses consecutive vertices. */
readonly "indices"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** One xyz normal per vertex; absent computes normals. */
readonly "normals"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Finite x/y/z vertex positions in metres. Triangle count, attribute
counts and index bounds are checked together by the domain validator. */
readonly "positions": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** One xyzw tangent frame per vertex; w is handedness. */
readonly "tangents"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** One uv pair per vertex; v down the image. */
readonly "uvs"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
});
readonly "CapsuleGeometry": ({
readonly "capSegments"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Straight middle length, not total capsule length. */
readonly "height"?: ((number) | (null));
readonly "heightSegments"?: ((number) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radialSegments"?: ((number) | (null));
readonly "radius"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "CircleGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radius"?: ((number) | (null));
readonly "segments"?: ((number) | (null));
readonly "thetaLength"?: ((number) | (null));
readonly "thetaStart"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "ConeGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "height"?: ((number) | (null));
readonly "heightSegments"?: ((number) | (null));
readonly "openEnded"?: ((boolean) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radialSegments"?: ((number) | (null));
readonly "radius"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
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
readonly "CsgGeometry": ({
readonly "a": (Input_terrainDefinitions["PlacedGeometry"]);
readonly "b": (Input_terrainDefinitions["PlacedGeometry"]);
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "op"?: (((Input_terrainDefinitions["CsgOp"])) | ((null)));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "CsgOp": ("union" | "subtract" | "intersect") & (string);
readonly "Curve": ({
readonly "closed"?: ((boolean) | (null));
readonly "kind"?: (((Input_terrainDefinitions["CurveKind"])) | ((null)));
readonly "points": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Catmull-Rom tangent scale; unset 0.5. */
readonly "tension"?: ((number) | (null));
});
readonly "CurveKind": ("catmullRom" | "bezier" | "linear") & (string);
readonly "CylinderGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "height"?: ((number) | (null));
readonly "heightSegments"?: ((number) | (null));
readonly "openEnded"?: ((boolean) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radialSegments"?: ((number) | (null));
readonly "radiusBottom"?: ((number) | (null));
readonly "radiusTop"?: ((number) | (null));
readonly "thetaLength"?: ((number) | (null));
readonly "thetaStart"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "Ellipse": ({
readonly "center": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "clockwise"?: (boolean);
readonly "end": (number);
readonly "radii": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "rotation"?: (number);
readonly "start": (number);
});
readonly "ExtrudeGeometry": ({
readonly "bevelEnabled"?: ((boolean) | (null));
readonly "bevelOffset"?: ((number) | (null));
readonly "bevelSegments"?: ((number) | (null));
/** Unset is bevelThickness - 0.1, not a second fixed default. */
readonly "bevelSize"?: ((number) | (null));
readonly "bevelThickness"?: ((number) | (null));
readonly "curveSegments"?: ((number) | (null));
readonly "depth"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "extrudePath"?: (((Input_terrainDefinitions["Curve"])) | ((null)));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Unset uses the generator's default square. */
readonly "shapes"?: ((ReadonlyArray<(Input_terrainDefinitions["Shape"])> | (number extends ((Input_terrainDefinitions["Shape"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "steps"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
/** Exactly one geometry, named as the corresponding render mechanism. */
readonly "Geometry": ((({
readonly "desc": (Input_terrainDefinitions["ModelGeometry"]);
readonly "kind": ("modelGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["BufferGeometry"]);
readonly "kind": ("bufferGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["BoxGeometry"]);
readonly "kind": ("boxGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["PlaneGeometry"]);
readonly "kind": ("planeGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["SphereGeometry"]);
readonly "kind": ("sphereGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["CylinderGeometry"]);
readonly "kind": ("cylinderGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["ConeGeometry"]);
readonly "kind": ("coneGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["CapsuleGeometry"]);
readonly "kind": ("capsuleGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["TorusGeometry"]);
readonly "kind": ("torusGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["RingGeometry"]);
readonly "kind": ("ringGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["CircleGeometry"]);
readonly "kind": ("circleGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["TubeGeometry"]);
readonly "kind": ("tubeGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["ShapeGeometry"]);
readonly "kind": ("shapeGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["ExtrudeGeometry"]);
readonly "kind": ("extrudeGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["LatheGeometry"]);
readonly "kind": ("latheGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["PolyhedronGeometry"]);
readonly "kind": ("polyhedronGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["SolidGeometry"]);
readonly "kind": ("icosahedronGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["SolidGeometry"]);
readonly "kind": ("dodecahedronGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["SolidGeometry"]);
readonly "kind": ("octahedronGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["SolidGeometry"]);
readonly "kind": ("tetrahedronGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["SweepGeometry"]);
readonly "kind": ("sweepGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["CsgGeometry"]);
readonly "kind": ("csgGeometry") & (string);
})) | (({
readonly "desc": (Input_terrainDefinitions["MergeGeometries"]);
readonly "kind": ("mergeGeometries") & (string);
})));
/** A heightfield uses metres, not a normalised 0..1 image. Range mapping is
exclusively the terrain file's explicit `height` setting. */
readonly "Heightfield": ({
/** Sample columns; 2..4097. Together with rows, fixes the height count. */
readonly "columns": (number);
/** Finite row-major heights in metres; never normalised or clipped. */
readonly "heights": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Sample rows; 2..4097, ordered from -z to +z. */
readonly "rows": (number);
});
readonly "HydraulicInput": ({
readonly "capacity"?: ((number) | (null));
readonly "deposition"?: ((number) | (null));
readonly "drops"?: ((number) | (null));
readonly "erosion"?: ((number) | (null));
readonly "evaporation"?: ((number) | (null));
readonly "gravity"?: ((number) | (null));
readonly "inertia"?: ((number) | (null));
readonly "lifetime"?: ((number) | (null));
readonly "radius"?: ((number) | (null));
});
readonly "LatheGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "phiLength"?: ((number) | (null));
readonly "phiStart"?: ((number) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Radial x and height y, metres; unset uses the generator's profile. */
readonly "points"?: ((ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "segments"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "MergeGeometries": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "parts": (ReadonlyArray<(Input_terrainDefinitions["PlacedGeometry"])> | (number extends ((Input_terrainDefinitions["PlacedGeometry"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "Mirror": ("none" | "x" | "y" | "z") & (string);
readonly "ModelGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Select a mesh node, or merge the visible default scene at rest. */
readonly "node"?: ((string) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** A model cooked before this geometry; dependency cycles are domain errors. */
readonly "source": (Input_terrainDefinitions["ContentRef"]);
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "MoistureInput": ({
readonly "flow"?: ((number) | (null));
readonly "frequency"?: ((number) | (null));
readonly "noise"?: ((number) | (null));
readonly "scale"?: ((number) | (null));
});
/** Ordered Path2 operations. These are structured calls, not embedded OGT. */
readonly "Path2Command": ((({
readonly "args": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "op": ("moveTo") & (string);
})) | (({
readonly "args": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "op": ("lineTo") & (string);
})) | (({
readonly "args": (ReadonlyArray<(number)> & { readonly length: 4 });
readonly "op": ("quadraticCurveTo") & (string);
})) | (({
readonly "args": (ReadonlyArray<(number)> & { readonly length: 6 });
readonly "op": ("bezierCurveTo") & (string);
})) | (({
readonly "args": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "op": ("splineThru") & (string);
})) | (({
readonly "args": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "op": ("points") & (string);
})) | (({
readonly "args": (Input_terrainDefinitions["Arc"]);
readonly "op": ("arc") & (string);
})) | (({
readonly "args": (Input_terrainDefinitions["Arc"]);
readonly "op": ("absarc") & (string);
})) | (({
readonly "args": (Input_terrainDefinitions["Ellipse"]);
readonly "op": ("ellipse") & (string);
})) | (({
readonly "args": (Input_terrainDefinitions["Ellipse"]);
readonly "op": ("absellipse") & (string);
})) | ((({
readonly "op": ("closePath") & (string);
})) & { readonly "args"?: never; }));
/** One CSG operand or merged part. The transform is the actual reflected
engine component, not a copied table of position/quaternion/scale fields. */
readonly "PlacedGeometry": ({
readonly "geometry": (Input_terrainDefinitions["Geometry"]);
readonly "mirror"?: (((Input_terrainDefinitions["Mirror"])) | ((null)));
readonly "transform"?: (((Input_terrainDefinitions["Reflected_og_cook::cook::source_schemas::world::geometry::Transform"])) | ((null)));
});
readonly "PlaneGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Y extent, metres; unset 1. */
readonly "height"?: ((number) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** X extent, metres; unset 1. The plane faces +Z. */
readonly "width"?: ((number) | (null));
});
readonly "PolyhedronGeometry": ({
readonly "detail"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** At least one triangle, indexing the corner triples. */
readonly "indices": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radius"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** Flat xyz corner coordinates, metres. */
readonly "vertices": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::terrain::Transform": ({
/** The position, as three.js Object3D.position. */
readonly "position"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The rotation, as three.js Object3D.quaternion. */
readonly "quaternion"?: (ReadonlyArray<(number)> & { readonly length: 4 });
/** The scale, as three.js Object3D.scale. */
readonly "scale"?: (ReadonlyArray<(number)> & { readonly length: 3 });
});
readonly "Reflected_og_cook::cook::source_schemas::world::geometry::Transform": ({
/** The position, as three.js Object3D.position. */
readonly "position"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The rotation, as three.js Object3D.quaternion. */
readonly "quaternion"?: (ReadonlyArray<(number)> & { readonly length: 4 });
/** The scale, as three.js Object3D.scale. */
readonly "scale"?: (ReadonlyArray<(number)> & { readonly length: 3 });
});
readonly "RingGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "innerRadius"?: ((number) | (null));
readonly "outerRadius"?: ((number) | (null));
readonly "phiSegments"?: ((number) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "thetaLength"?: ((number) | (null));
readonly "thetaSegments"?: ((number) | (null));
readonly "thetaStart"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "RiversInput": ({
readonly "depth"?: ((number) | (null));
readonly "threshold"?: ((number) | (null));
readonly "width"?: ((number) | (null));
});
readonly "Shape": ({
readonly "commands": (ReadonlyArray<(Input_terrainDefinitions["Path2Command"])> | (number extends ((Input_terrainDefinitions["Path2Command"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Hole contours cannot contain more holes. */
readonly "holes"?: (ReadonlyArray<(ReadonlyArray<(Input_terrainDefinitions["Path2Command"])> | (number extends ((Input_terrainDefinitions["Path2Command"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))> | (number extends ((ReadonlyArray<(Input_terrainDefinitions["Path2Command"])> | (number extends ((Input_terrainDefinitions["Path2Command"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "ShapeGeometry": ({
readonly "curveSegments"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Unset uses the generator's default triangle. */
readonly "shapes"?: ((ReadonlyArray<(Input_terrainDefinitions["Shape"])> | (number extends ((Input_terrainDefinitions["Shape"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "SolidGeometry": ({
readonly "detail"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radius"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "SphereGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "heightSegments"?: ((number) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radius"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "widthSegments"?: ((number) | (null));
});
readonly "StampMode": ("carve" | "hole") & (string);
readonly "SweepGeometry": ({
readonly "caps"?: ((boolean) | (null));
readonly "curveSegments"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "path": (Input_terrainDefinitions["Curve"]);
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Ordered interpolation keys along the path (t, scale). */
readonly "scale"?: ((ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Cross-section without holes. Curve flattening uses curveSegments. */
readonly "section": (ReadonlyArray<(Input_terrainDefinitions["Path2Command"])> | (number extends ((Input_terrainDefinitions["Path2Command"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "steps"?: ((number) | (null));
/** Ordered interpolation keys (t, radians). */
readonly "twist"?: ((ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "TerracesInput": ({
readonly "blend"?: ((number) | (null));
readonly "step"?: ((number) | (null));
});
readonly "TerrainChannel": ({
readonly "depth"?: ((number) | (null));
/** Consecutive XZ points must differ; river Y must not rise downstream. */
readonly "points": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "river"?: ((boolean) | (null));
readonly "width": (number);
});
readonly "TerrainEdit": (((({
readonly "center": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "falloff"?: ((number) | (null));
readonly "kind": ("brush") & (string);
readonly "layer"?: ((number) | (null));
readonly "mode": (Input_terrainDefinitions["BrushMode"]);
readonly "radius": (number);
/** Raise/lower use metres; other brush modes clamp strength to one. */
readonly "strength"?: ((number) | (null));
})) & { readonly "depth"?: never; readonly "geometry"?: never; readonly "transform"?: never; }) | ((({
readonly "depth"?: ((number) | (null));
readonly "geometry": (Input_terrainDefinitions["Geometry"]);
readonly "kind": ("stamp") & (string);
readonly "mode"?: (((Input_terrainDefinitions["StampMode"])) | ((null)));
/** Existing Transform reflection supplies placement, never a copied field table. */
readonly "transform"?: (((Input_terrainDefinitions["Reflected_og_cook::cook::source_schemas::procedural::terrain::Transform"])) | ((null)));
})) & { readonly "center"?: never; readonly "falloff"?: never; readonly "layer"?: never; readonly "radius"?: never; readonly "strength"?: never; }));
readonly "TerrainLayerInput": ({
readonly "albedo"?: (((Input_terrainDefinitions["ContentRef"])) | ((null)));
readonly "albedoLayer"?: ((number) | (null));
readonly "altitude"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "altitudeFade"?: ((number) | (null));
readonly "antiTiling"?: ((number) | (null));
readonly "cellVariation"?: ((number) | (null));
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "displacement"?: ((number) | (null));
readonly "heightContrast"?: ((number) | (null));
readonly "macroVariation"?: ((number) | (null));
readonly "normal"?: (((Input_terrainDefinitions["ContentRef"])) | ((null)));
readonly "normalLayer"?: ((number) | (null));
readonly "normalStrength"?: ((number) | (null));
readonly "projectSlope"?: ((number) | (null));
readonly "roughness"?: ((number) | (null));
readonly "slope"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "slopeFade"?: ((number) | (null));
readonly "tile"?: ((number) | (null));
readonly "triplanar"?: ((boolean) | (null));
});
readonly "TerrainLod": ({
readonly "ring"?: ((number) | (null));
/** Zero uses the source sample spacing. */
readonly "spacing"?: ((number) | (null));
});
readonly "TerrainMask": (((({
readonly "kind": ("none") & (string);
})) & { readonly "falloff"?: never; readonly "strength"?: never; }) | ((({
readonly "falloff"?: ((number) | (null));
readonly "kind": ("island") & (string);
})) & { readonly "strength"?: never; }) | ((({
readonly "kind": ("ridge") & (string);
readonly "strength"?: ((number) | (null));
})) & { readonly "falloff"?: never; }));
readonly "TerrainNoise": ({
/** Noise features across the longer world extent. */
readonly "frequency"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "lacunarity"?: ((number) | (null));
readonly "octaves"?: ((number) | (null));
readonly "warp"?: ((number) | (null));
});
/** Processing runs in domain order: hydraulic, thermal, terraces, shore,
drainage, authored channels and biome controls. It never evaluates TS. */
readonly "TerrainProcess": ({
readonly "biomes"?: ((ReadonlyArray<(Input_terrainDefinitions["BiomeInput"])> | (number extends ((Input_terrainDefinitions["BiomeInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "channels"?: ((ReadonlyArray<(Input_terrainDefinitions["TerrainChannel"])> | (number extends ((Input_terrainDefinitions["TerrainChannel"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "erosion"?: (((Input_terrainDefinitions["HydraulicInput"])) | ((null)));
readonly "moisture"?: (((Input_terrainDefinitions["MoistureInput"])) | ((null)));
readonly "preserve"?: (((Input_terrainDefinitions["ContentRef"])) | ((null)));
readonly "preserveBorder"?: ((number) | (null));
readonly "rivers"?: (((Input_terrainDefinitions["RiversInput"])) | ((null)));
readonly "sea"?: ((number) | (null));
readonly "shore"?: ((number) | (null));
readonly "terraces"?: (((Input_terrainDefinitions["TerracesInput"])) | ((null)));
readonly "thermal"?: (((Input_terrainDefinitions["ThermalInput"])) | ((null)));
});
readonly "TerrainSource": (((({
/** Metres corresponding to image intensities zero and one. */
readonly "height"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "kind": ("image") & (string);
/** PNG/EXR source relative to the owning module; first channel is height. */
readonly "source": (string);
})) & { readonly "columns"?: never; readonly "field"?: never; readonly "mask"?: never; readonly "noise"?: never; readonly "rows"?: never; }) | ((({
/** The shared heightfield's domain validator checks rows × columns and finiteness. */
readonly "field": (Input_terrainDefinitions["Heightfield"]);
readonly "kind": ("heightfield") & (string);
})) & { readonly "columns"?: never; readonly "height"?: never; readonly "mask"?: never; readonly "noise"?: never; readonly "rows"?: never; readonly "source"?: never; }) | ((({
readonly "columns"?: ((number) | (null));
readonly "height"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "kind": ("noise") & (string);
readonly "mask"?: (((Input_terrainDefinitions["TerrainMask"])) | ((null)));
readonly "noise"?: (((Input_terrainDefinitions["TerrainNoise"])) | ((null)));
readonly "rows"?: ((number) | (null));
})) & { readonly "field"?: never; readonly "source"?: never; }));
readonly "ThermalInput": ({
readonly "iterations"?: ((number) | (null));
readonly "rate"?: ((number) | (null));
readonly "talus"?: ((number) | (null));
});
readonly "TorusGeometry": ({
readonly "arc"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radialSegments"?: ((number) | (null));
readonly "radius"?: ((number) | (null));
readonly "tube"?: ((number) | (null));
readonly "tubularSegments"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "TubeGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** A sampled path or a Catmull-Rom point list, never a runtime Path handle. */
readonly "path": (Input_terrainDefinitions["TubePath"]);
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radialSegments"?: ((number) | (null));
readonly "radius"?: ((number) | (null));
readonly "tubularSegments"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "TubePath": (((({
readonly "closed"?: ((boolean) | (null));
readonly "kind": ("points") & (string);
readonly "points": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "curve"?: never; }) | ((({
readonly "curve": (Input_terrainDefinitions["Curve"]);
readonly "kind": ("curve") & (string);
})) & { readonly "closed"?: never; readonly "points"?: never; }));
}
/** An ordered array of equally-sized external images. File resolution,
dimensions and encoder compatibility remain texture-array domain checks. */
export type Input_texture_array = ({
readonly "kind"?: (Input_texture_arrayDefinitions["TextureKind"]);
/** Explicit canonical paths relative to the owning module (e.g.
assets/tiles/grass.png), without sibling or asset-root fallback. */
readonly "layers": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
export interface Input_texture_arrayDefinitions {
readonly "TextureKind": ("color" | "normal" | "linear" | "single" | "pixel") & (string);
}
/** A recipe selects dimensions and overlays a complete structured pack patch.
Dimension names are data read by the domain, not a schema-maintained list. */
export type Input_theme = ({
readonly "about"?: ((string) | (null));
readonly "color"?: (((Input_themeDefinitions["ThemeColor"])) | ((null)));
readonly "density"?: ((string) | (null));
readonly "effects"?: ((string) | (null));
readonly "extends"?: (((Input_themeDefinitions["ThemeBase"])) | ((null)));
readonly "icons"?: ((string) | (null));
readonly "material"?: ((string) | (null));
readonly "motion"?: ((string) | (null));
readonly "ornament"?: (((Input_themeDefinitions["OrnamentInput"])) | ((null)));
readonly "pack"?: (((Input_themeDefinitions["PackInput"])) | ((null)));
readonly "shape"?: ((string) | (null));
readonly "sound"?: ((string) | (null));
readonly "type"?: ((string) | (null));
/** Built-in mix-in names, applied in order after the parent recipe. */
readonly "with"?: ((ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
});
export interface Input_themeDefinitions {
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive": (((Input_themeDefinitions["Schemes"])) | ((string)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive10": (((Input_themeDefinitions["Schemes10"])) | ((ReadonlyArray<(number)> & { readonly length: 4 })));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive11": ((((Input_themeDefinitions["Schemes11"])) & { readonly "fill"?: never; readonly "repeat"?: never; readonly "slice"?: never; readonly "src"?: never; readonly "tint"?: never; readonly "width"?: never; }) | (((Input_themeDefinitions["NineSlice"])) & { readonly "dark"?: never; readonly "light"?: never; }));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive12": ((((Input_themeDefinitions["Schemes12"])) & { readonly "$ogRef"?: never; }) | (((Input_themeDefinitions["ContentRef"])) & { readonly "dark"?: never; readonly "light"?: never; }));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive13": (((Input_themeDefinitions["Schemes13"])) | ((Input_themeDefinitions["Insets"])));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive14": (((Input_themeDefinitions["Schemes14"])) | ((boolean)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive15": (((Input_themeDefinitions["Schemes15"])) | ((Input_themeDefinitions["Repeat"])));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive2": (((Input_themeDefinitions["Schemes2"])) | ((Readonly<Record<string, (Input_themeDefinitions["Adaptive3"])>>)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive3": (((Input_themeDefinitions["Schemes3"])) | ((Input_themeDefinitions["Palette"])));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive4": (((Input_themeDefinitions["Schemes4"])) | ((number)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive5": (((Input_themeDefinitions["Schemes5"])) | ((Readonly<Record<string, (Input_themeDefinitions["Adaptive4"])>>)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive6": (((Input_themeDefinitions["Schemes6"])) | ((Readonly<Record<string, (Input_themeDefinitions["Adaptive7"])>>)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive7": (((Input_themeDefinitions["Schemes7"])) | ((Input_themeDefinitions["FontSize"])));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive8": (((Input_themeDefinitions["Schemes8"])) | ((Readonly<Record<string, (Input_themeDefinitions["Adaptive"])>>)));
/** A shared value or an explicit complete light/dark pair. */
readonly "Adaptive9": (((Input_themeDefinitions["Schemes9"])) | ((Readonly<Record<string, (Input_themeDefinitions["Adaptive10"])>>)));
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
readonly "back"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "cancel"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "close"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "confirm"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "countdown"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "error"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "hover"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "levelup"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "notify"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "open"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "press"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "purchase"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "reward"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "select"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "step"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "swipe"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "tab"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "tick"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "toggle"?: (((Input_themeDefinitions["Sound"])) | ((null)));
readonly "unlock"?: (((Input_themeDefinitions["Sound"])) | ((null)));
});
/** The same interaction event vocabulary for global and slot-local values. */
readonly "Events2": ({
readonly "back"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "cancel"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "close"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "confirm"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "countdown"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "error"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "hover"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "levelup"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "notify"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "open"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "press"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "purchase"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "reward"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "select"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "step"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "swipe"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "tab"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "tick"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "toggle"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
readonly "unlock"?: (((Input_themeDefinitions["Haptic"])) | ((null)));
});
readonly "FontSize": (((number)) | ((ReadonlyArray<(number)> & { readonly length: 2 })));
readonly "Haptic": ("tick" | "tap" | "thud" | "double" | "buzz" | "success" | "warning" | "error" | "none") & (string);
readonly "HapticBinding": (((Input_themeDefinitions["Haptic"])) | ((Input_themeDefinitions["Events2"])));
readonly "Insets": (((Input_themeDefinitions["Nonnegative"])) | ((ReadonlyArray<(Input_themeDefinitions["Nonnegative"])> & { readonly length: 4 })));
readonly "NineSlice": ({
/** Unset paints the centre. */
readonly "fill"?: (((Input_themeDefinitions["Adaptive14"])) | ((null)));
readonly "repeat"?: (((Input_themeDefinitions["Adaptive15"])) | ((null)));
readonly "slice": (Input_themeDefinitions["Adaptive13"]);
readonly "src": (Input_themeDefinitions["Adaptive12"]);
readonly "tint"?: (((Input_themeDefinitions["Adaptive14"])) | ((null)));
/** Unset draws at the slice's own widths, in points. */
readonly "width"?: (((Input_themeDefinitions["Adaptive13"])) | ((null)));
});
readonly "Nonnegative": (number);
readonly "OrnamentInput": (((string)) | ((Input_themeDefinitions["OrnamentSlots"])));
/** Per-piece names from the ornament dimension, not arbitrary object keys. */
readonly "OrnamentSlots": ({
readonly "button"?: ((string) | (null));
readonly "chip"?: ((string) | (null));
readonly "panel"?: ((string) | (null));
readonly "plate"?: ((string) | (null));
readonly "slot"?: ((string) | (null));
});
/** One style layer. Class resolution, CSS parsing, scheme-pair semantics,
slot names and referenced content are checked by the pack domain validator. */
readonly "PackInput": ({
/** Ignored author annotation, deliberately arbitrary JSON data. */
readonly "about"?: unknown;
readonly "contrast"?: (((Input_themeDefinitions["Theme"])) | ((null)));
/** Ordered font faces; family, weight and style belong to each face. */
readonly "fonts"?: (ReadonlyArray<(Input_themeDefinitions["ContentRef"])> | (number extends ((Input_themeDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "haptics"?: (Readonly<Record<string, (Input_themeDefinitions["HapticBinding"])>>);
readonly "iconSet"?: (((Input_themeDefinitions["ContentRef"])) | ((null)));
readonly "icons"?: (Readonly<Record<string, (Input_themeDefinitions["ContentRef"])>>);
/** Named nine-slice art; names are kebab-case. */
readonly "images"?: (Readonly<Record<string, (Input_themeDefinitions["Adaptive11"])>>);
readonly "scheme"?: (((Input_themeDefinitions["Scheme"])) | ((null)));
readonly "seeds"?: (((Input_themeDefinitions["Seeds"])) | ((null)));
/** An event maps to a sound; a Component.slot[.variant] maps to events. */
readonly "sounds"?: (Readonly<Record<string, (Input_themeDefinitions["SoundBinding"])>>);
readonly "theme"?: (((Input_themeDefinitions["Theme"])) | ((null)));
/** Component.slot[.variant] -> class string; empty declares the slot. */
readonly "variants"?: (Readonly<Record<string, (string)>>);
});
readonly "Palette": (((string)) | ((Readonly<Record<string, (Input_themeDefinitions["Adaptive"])>>)));
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
readonly "dark": (Input_themeDefinitions["NineSlice"]);
readonly "light": (Input_themeDefinitions["NineSlice"]);
});
readonly "Schemes12": ({
readonly "dark": (Input_themeDefinitions["ContentRef"]);
readonly "light": (Input_themeDefinitions["ContentRef"]);
});
readonly "Schemes13": ({
readonly "dark": (Input_themeDefinitions["Insets"]);
readonly "light": (Input_themeDefinitions["Insets"]);
});
readonly "Schemes14": ({
readonly "dark": (boolean);
readonly "light": (boolean);
});
readonly "Schemes15": ({
readonly "dark": (Input_themeDefinitions["Repeat"]);
readonly "light": (Input_themeDefinitions["Repeat"]);
});
readonly "Schemes2": ({
readonly "dark": (Readonly<Record<string, (Input_themeDefinitions["Adaptive3"])>>);
readonly "light": (Readonly<Record<string, (Input_themeDefinitions["Adaptive3"])>>);
});
readonly "Schemes3": ({
readonly "dark": (Input_themeDefinitions["Palette"]);
readonly "light": (Input_themeDefinitions["Palette"]);
});
readonly "Schemes4": ({
readonly "dark": (number);
readonly "light": (number);
});
readonly "Schemes5": ({
readonly "dark": (Readonly<Record<string, (Input_themeDefinitions["Adaptive4"])>>);
readonly "light": (Readonly<Record<string, (Input_themeDefinitions["Adaptive4"])>>);
});
readonly "Schemes6": ({
readonly "dark": (Readonly<Record<string, (Input_themeDefinitions["Adaptive7"])>>);
readonly "light": (Readonly<Record<string, (Input_themeDefinitions["Adaptive7"])>>);
});
readonly "Schemes7": ({
readonly "dark": (Input_themeDefinitions["FontSize"]);
readonly "light": (Input_themeDefinitions["FontSize"]);
});
readonly "Schemes8": ({
readonly "dark": (Readonly<Record<string, (Input_themeDefinitions["Adaptive"])>>);
readonly "light": (Readonly<Record<string, (Input_themeDefinitions["Adaptive"])>>);
});
readonly "Schemes9": ({
readonly "dark": (Readonly<Record<string, (Input_themeDefinitions["Adaptive10"])>>);
readonly "light": (Readonly<Record<string, (Input_themeDefinitions["Adaptive10"])>>);
});
readonly "Seeds": ({
readonly "accent"?: (((Input_themeDefinitions["Adaptive"])) | ((null)));
readonly "danger"?: (((Input_themeDefinitions["Adaptive"])) | ((null)));
readonly "hud"?: (((Input_themeDefinitions["Adaptive"])) | ((null)));
readonly "info"?: (((Input_themeDefinitions["Adaptive"])) | ((null)));
readonly "neutral"?: (((Input_themeDefinitions["Adaptive"])) | ((null)));
readonly "primary"?: (((Input_themeDefinitions["Adaptive"])) | ((null)));
readonly "success"?: (((Input_themeDefinitions["Adaptive"])) | ((null)));
readonly "team"?: (((Input_themeDefinitions["Adaptive"])) | ((null)));
readonly "warn"?: (((Input_themeDefinitions["Adaptive"])) | ((null)));
});
readonly "Sound": ({
/** Empty clips silence an underlying layer only when volume is zero. */
readonly "clips"?: (ReadonlyArray<(Input_themeDefinitions["ContentRef"])> | (number extends ((Input_themeDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Playback rate or [low, high]; unset is 1. */
readonly "pitch"?: (((Input_themeDefinitions["Range"])) | ((null)));
/** Gain or [low, high]; unset is 1. */
readonly "volume"?: (((Input_themeDefinitions["Range"])) | ((null)));
});
readonly "SoundBinding": ((((Input_themeDefinitions["Sound"])) & { readonly "back"?: never; readonly "cancel"?: never; readonly "close"?: never; readonly "confirm"?: never; readonly "countdown"?: never; readonly "error"?: never; readonly "hover"?: never; readonly "levelup"?: never; readonly "notify"?: never; readonly "open"?: never; readonly "press"?: never; readonly "purchase"?: never; readonly "reward"?: never; readonly "select"?: never; readonly "step"?: never; readonly "swipe"?: never; readonly "tab"?: never; readonly "tick"?: never; readonly "toggle"?: never; readonly "unlock"?: never; }) | (((Input_themeDefinitions["Events"])) & { readonly "clips"?: never; readonly "pitch"?: never; readonly "volume"?: never; }));
/** NSH Tailwind theme sections, as consumed by Theme::from_json after scheme
splitting. Numerical scales are numbers, not arbitrary CSS/JSON values. */
readonly "Theme": ({
readonly "aspect"?: (((Input_themeDefinitions["Adaptive5"])) | ((null)));
readonly "blur"?: (((Input_themeDefinitions["Adaptive5"])) | ((null)));
readonly "breakpoints"?: (((Input_themeDefinitions["Adaptive5"])) | ((null)));
readonly "colors"?: (((Input_themeDefinitions["Adaptive2"])) | ((null)));
readonly "container"?: (((Input_themeDefinitions["Adaptive5"])) | ((null)));
readonly "dropShadow"?: (((Input_themeDefinitions["Adaptive8"])) | ((null)));
readonly "duration"?: (((Input_themeDefinitions["Adaptive5"])) | ((null)));
/** Cubic Bezier control points [x1, y1, x2, y2]. */
readonly "ease"?: (((Input_themeDefinitions["Adaptive9"])) | ((null)));
readonly "fontFamily"?: (((Input_themeDefinitions["Adaptive8"])) | ((null)));
readonly "fontSize"?: (((Input_themeDefinitions["Adaptive6"])) | ((null)));
readonly "fontWeight"?: (((Input_themeDefinitions["Adaptive5"])) | ((null)));
readonly "insetShadow"?: (((Input_themeDefinitions["Adaptive8"])) | ((null)));
readonly "leading"?: (((Input_themeDefinitions["Adaptive5"])) | ((null)));
readonly "opacity"?: (((Input_themeDefinitions["Adaptive5"])) | ((null)));
readonly "perspective"?: (((Input_themeDefinitions["Adaptive5"])) | ((null)));
readonly "radius"?: (((Input_themeDefinitions["Adaptive5"])) | ((null)));
/** Root em in points. */
readonly "rem"?: (((Input_themeDefinitions["Adaptive4"])) | ((null)));
readonly "scale"?: (((Input_themeDefinitions["Adaptive5"])) | ((null)));
/** CSS box-shadow values. */
readonly "shadow"?: (((Input_themeDefinitions["Adaptive8"])) | ((null)));
/** Base spacing unit in points. */
readonly "spacing"?: (((Input_themeDefinitions["Adaptive4"])) | ((null)));
readonly "stroke"?: (((Input_themeDefinitions["Adaptive5"])) | ((null)));
/** CSS text-shadow values. */
readonly "textShadow"?: (((Input_themeDefinitions["Adaptive8"])) | ((null)));
readonly "tracking"?: (((Input_themeDefinitions["Adaptive5"])) | ((null)));
});
readonly "ThemeBase": (((string)) | ((Input_themeDefinitions["ContentRef"])));
readonly "ThemeColor": ({
/** Any independent literal CSS colour. */
readonly "accent"?: ((string) | (null));
/** Any independent literal CSS colour. */
readonly "danger"?: ((string) | (null));
/** Any independent literal CSS colour. */
readonly "neutral"?: ((string) | (null));
readonly "scheme"?: (((Input_themeDefinitions["ThemeScheme"])) | ((null)));
/** Any independent literal CSS colour; domain parsing rejects variables. */
readonly "source"?: ((string) | (null));
readonly "variant"?: ((string) | (null));
});
readonly "ThemeScheme": ("light" | "dark") & (string);
}
export type Input_tilemap = ({
readonly "height": (number);
readonly "layers": (ReadonlyArray<(Input_tilemapDefinitions["Layer"])> | (number extends ((Input_tilemapDefinitions["Layer"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "objects"?: (ReadonlyArray<(Input_tilemapDefinitions["Object"])> | (number extends ((Input_tilemapDefinitions["Object"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "orientation"?: (((Input_tilemapDefinitions["Orientation"])) | ((null)));
readonly "tileHeight": (number);
readonly "tileWidth": (number);
readonly "tilesets": (ReadonlyArray<(Input_tilemapDefinitions["ContentRef"])> | (number extends ((Input_tilemapDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** World units per tile width; height retains the authored pixel aspect ratio. */
readonly "unit"?: ((number) | (null));
readonly "width": (number);
});
export interface Input_tilemapDefinitions {
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Layer": ({
readonly "cellColliders"?: ((boolean) | (null));
/** Row-major, top row first: 0 empty; otherwise tile+1 with flip bits. */
readonly "cells": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "colliderLayers"?: ((number) | (null));
readonly "colliderMask"?: ((number) | (null));
readonly "collision"?: ((boolean) | (null));
readonly "lit"?: ((boolean) | (null));
readonly "name": (string);
readonly "offset"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "opacity"?: ((number) | (null));
readonly "parallax"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "repeatX"?: ((boolean) | (null));
readonly "repeatY"?: ((boolean) | (null));
readonly "terrain"?: ((string) | (null));
/** Index into the map's declared tilesets, checked by the domain. */
readonly "tileset": (number);
readonly "visible"?: ((boolean) | (null));
readonly "z"?: ((number) | (null));
});
readonly "Object": ({
readonly "height"?: ((number) | (null));
readonly "kind": (string);
readonly "name": (string);
readonly "properties"?: (Readonly<Record<string, (string)>>);
readonly "width"?: ((number) | (null));
/** Map-local world units, y up from the map's top-left. */
readonly "x": (number);
readonly "y": (number);
});
readonly "Orientation": ("orthogonal" | "isometric" | "staggered" | "hex" | "hexFlat") & (string);
}
export type Input_tileset = ({
readonly "columns": (number);
/** Per-tileset art conformance; omission uses the project's art contract. */
readonly "conform"?: (((Input_tilesetDefinitions["TilesetConform"])) | ((null)));
readonly "filter"?: (((Input_tilesetDefinitions["Filter"])) | ((null)));
readonly "margin"?: (number);
readonly "rows": (number);
readonly "spacing"?: (number);
readonly "terrains"?: (ReadonlyArray<(Input_tilesetDefinitions["Terrain"])> | (number extends ((Input_tilesetDefinitions["Terrain"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "texture"?: (((Input_tilesetDefinitions["ContentRef"])) | ((null)));
readonly "tileHeight": (number);
readonly "tileWidth": (number);
readonly "tiles"?: (Readonly<Record<string, ((Input_tilesetDefinitions["Tile"]))>>);
});
export interface Input_tilesetDefinitions {
readonly "Animation": ({
/** One strictly positive duration per frame; domain checks matching counts. */
readonly "durations": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "frames": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Collision": (((({
readonly "kind": ("full") & (string);
})) & { readonly "h"?: never; readonly "w"?: never; readonly "x"?: never; readonly "y"?: never; }) | (({
readonly "h": (number);
readonly "kind": ("rect") & (string);
readonly "w": (number);
readonly "x": (number);
readonly "y": (number);
})));
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Filter": ("nearest" | "linear") & (string);
readonly "Terrain": ({
readonly "default": (number);
readonly "edgeFilled"?: ((boolean) | (null));
readonly "mode": (Input_tilesetDefinitions["TerrainMode"]);
readonly "name": (string);
/** Neighbor mask to tile id. Domain checks reachable masks and tile bounds. */
readonly "rules": (Readonly<Record<string, ((number))>>);
});
readonly "TerrainMode": ("blob47" | "bitmask16") & (string);
readonly "Tile": ({
readonly "animation"?: (((Input_tilesetDefinitions["Animation"])) | ((null)));
readonly "collision"?: (((Input_tilesetDefinitions["Collision"])) | ((null)));
/** Closed polygons, top-left pixel coordinates, y down. */
readonly "polygons"?: (ReadonlyArray<(ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))> | (number extends ((ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "TilesetConform": (((({
readonly "kind": ("off") & (string);
})) & { readonly "size"?: never; }) | (({
readonly "kind": ("size") & (string);
readonly "size": (number);
})));
}
export type Input_timeline = ({
readonly "duration"?: ((number) | (null));
readonly "name"?: ((string) | (null));
readonly "paths"?: (Readonly<Record<string, (Input_timelineDefinitions["Path"])>>);
readonly "tracks": (ReadonlyArray<(Input_timelineDefinitions["Track"])> | (number extends ((Input_timelineDefinitions["Track"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
export interface Input_timelineDefinitions {
readonly "Animation": ({
readonly "blendIn"?: ((number) | (null));
readonly "clip": (string);
readonly "loop"?: ((boolean) | (null));
readonly "speed"?: ((number) | (null));
});
readonly "Audio": ({
readonly "bus"?: (((Input_timelineDefinitions["Bus"])) | ((null)));
readonly "playbackRate"?: ((number) | (null));
readonly "sound": (Input_timelineDefinitions["ContentRef"]);
readonly "volume"?: ((number) | (null));
});
readonly "Bus": ("master" | "music" | "sfx" | "ui") & (string);
readonly "Camera": ({
readonly "at"?: ((number) | (null));
readonly "focus"?: ((number) | (null));
readonly "focusOn"?: ((string) | (null));
readonly "focusRange"?: ((number) | (null));
readonly "fov"?: ((number) | (null));
readonly "position"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "roll"?: ((number) | (null));
readonly "target"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
});
readonly "Caption": ({
readonly "end"?: ((number) | (null));
readonly "position"?: (((Input_timelineDefinitions["CaptionPosition"])) | ((null)));
readonly "spans"?: (ReadonlyArray<(Input_timelineDefinitions["Span"])> | (number extends ((Input_timelineDefinitions["Span"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "speaker"?: ((string) | (null));
readonly "text": (string);
});
readonly "CaptionCues": ({
readonly "imports"?: (ReadonlyArray<(Input_timelineDefinitions["CaptionImport"])> | (number extends ((Input_timelineDefinitions["CaptionImport"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "keys"?: (ReadonlyArray<(Input_timelineDefinitions["Key6"])> | (number extends ((Input_timelineDefinitions["Key6"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "CaptionImport": ({
readonly "offset"?: ((number) | (null));
/** Explicit SRT, VTT or JSON cue file; cook captures it as a dependency. */
readonly "source": (string);
});
readonly "CaptionPosition": ("top" | "bottom") & (string);
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Direction": ("left" | "right" | "top" | "bottom") & (string);
readonly "Event": ({
readonly "action": (string);
readonly "retroactive"?: ((boolean) | (null));
});
readonly "FieldValue": (((number)) | ((boolean)) | ((string)) | ((ReadonlyArray<(number)> & { readonly length: 2 })) | ((ReadonlyArray<(number)> & { readonly length: 3 })) | ((ReadonlyArray<(number)> & { readonly length: 4 })));
readonly "Hotspot": ({
readonly "id": (string);
readonly "kind"?: (((Input_timelineDefinitions["HotspotKind"])) | ((null)));
readonly "label"?: ((string) | (null));
readonly "off"?: ((boolean) | (null));
readonly "rect"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "shape"?: (((Input_timelineDefinitions["HotspotShape"])) | ((null)));
});
readonly "HotspotKind": ("choice" | "inspect") & (string);
readonly "HotspotShape": ("rect" | "circle") & (string);
readonly "Key": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Camera"]);
});
readonly "Key10": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["FieldValue"]);
});
readonly "Key11": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Light"]);
});
readonly "Key12": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Reflected_og_cook::cook::source_schemas::media::timeline::PostEffects"]);
});
readonly "Key13": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (boolean);
});
readonly "Key14": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Reflected_og_cook::cook::source_schemas::media::timeline::Blendshapes"]);
});
readonly "Key15": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Material"]);
});
readonly "Key16": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Nested"]);
});
readonly "Key17": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Shot"]);
});
readonly "Key18": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Vfx"]);
});
readonly "Key19": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Video"]);
});
readonly "Key2": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Transform"]);
});
readonly "Key20": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Hotspot"]);
});
readonly "Key21": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Scene"]);
});
readonly "Key3": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Animation"]);
});
readonly "Key4": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Audio"]);
});
readonly "Key5": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Music"]);
});
readonly "Key6": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Caption"]);
});
readonly "Key7": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Event"]);
});
readonly "Key8": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Transition"]);
});
readonly "Key9": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (Input_timelineDefinitions["Marker"]);
});
readonly "Light": ((((Input_timelineDefinitions["Reflected_og_cook::cook::source_schemas::media::timeline::DirectionalLight"])) & { readonly "angle"?: never; readonly "castShadows"?: never; readonly "cookie"?: never; readonly "distance"?: never; readonly "falloff"?: never; readonly "height"?: never; readonly "innerAngle"?: never; readonly "kind"?: never; readonly "layers"?: never; readonly "outerAngle"?: never; readonly "penumbra"?: never; readonly "radius"?: never; readonly "sourceRadius"?: never; readonly "twoSided"?: never; readonly "width"?: never; }) | (((Input_timelineDefinitions["Reflected_og_cook::cook::source_schemas::media::timeline::PointLight"])) & { readonly "angle"?: never; readonly "cascadeBlend"?: never; readonly "castShadows"?: never; readonly "falloff"?: never; readonly "height"?: never; readonly "innerAngle"?: never; readonly "kind"?: never; readonly "layers"?: never; readonly "outerAngle"?: never; readonly "penumbra"?: never; readonly "shadowBias"?: never; readonly "shadowBox"?: never; readonly "shadowDistance"?: never; readonly "shadowFocus"?: never; readonly "shadowFocusDrop"?: never; readonly "shadowNormalBias"?: never; readonly "shadowSoftness"?: never; readonly "sourceRadius"?: never; readonly "twoSided"?: never; readonly "width"?: never; }) | (((Input_timelineDefinitions["Reflected_og_cook::cook::source_schemas::media::timeline::SpotLight"])) & { readonly "cascadeBlend"?: never; readonly "castShadows"?: never; readonly "falloff"?: never; readonly "height"?: never; readonly "innerAngle"?: never; readonly "kind"?: never; readonly "layers"?: never; readonly "outerAngle"?: never; readonly "shadowBias"?: never; readonly "shadowBox"?: never; readonly "shadowDistance"?: never; readonly "shadowFocus"?: never; readonly "shadowFocusDrop"?: never; readonly "shadowNormalBias"?: never; readonly "shadowSoftness"?: never; readonly "sourceRadius"?: never; readonly "twoSided"?: never; readonly "width"?: never; }) | (((Input_timelineDefinitions["Reflected_og_cook::cook::source_schemas::media::timeline::RectLight"])) & { readonly "angle"?: never; readonly "cascadeBlend"?: never; readonly "castShadows"?: never; readonly "cookie"?: never; readonly "distance"?: never; readonly "falloff"?: never; readonly "innerAngle"?: never; readonly "kind"?: never; readonly "layers"?: never; readonly "outerAngle"?: never; readonly "penumbra"?: never; readonly "radius"?: never; readonly "shadowBias"?: never; readonly "shadowBox"?: never; readonly "shadowDistance"?: never; readonly "shadowFocus"?: never; readonly "shadowFocusDrop"?: never; readonly "shadowNormalBias"?: never; readonly "shadowSoftness"?: never; readonly "sourceRadius"?: never; }) | (((Input_timelineDefinitions["Reflected_og_cook::cook::source_schemas::media::timeline::Light2D"])) & { readonly "angle"?: never; readonly "cascadeBlend"?: never; readonly "castShadow"?: never; readonly "cookie"?: never; readonly "mobility"?: never; readonly "penumbra"?: never; readonly "radius"?: never; readonly "shadowBias"?: never; readonly "shadowBox"?: never; readonly "shadowDistance"?: never; readonly "shadowFocus"?: never; readonly "shadowFocusDrop"?: never; readonly "shadowNormalBias"?: never; readonly "shadowSoftness"?: never; readonly "twoSided"?: never; readonly "width"?: never; }));
readonly "Marker": ({
readonly "name": (string);
});
readonly "Material": ({
readonly "mapOffset"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "mapRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "mapRotation"?: ((number) | (null));
/** Explicit shader uniforms, not an escape hatch for misspelled standard
fields. Matching the target shader's names/dimensions is a graph check. */
readonly "params"?: (({
readonly "alphaTest"?: never;
readonly "anisotropy"?: never;
readonly "anisotropyRotation"?: never;
readonly "aoMapIntensity"?: never;
readonly "attenuationColor"?: never;
readonly "attenuationDistance"?: never;
readonly "clearcoat"?: never;
readonly "clearcoatRoughness"?: never;
readonly "color"?: never;
readonly "dissolve"?: never;
readonly "dissolveEdgeColor"?: never;
readonly "dissolveEdgeWidth"?: never;
readonly "dissolveScale"?: never;
readonly "emissive"?: never;
readonly "emissiveIntensity"?: never;
readonly "envMapIntensity"?: never;
readonly "flash"?: never;
readonly "flashColor"?: never;
readonly "ior"?: never;
readonly "iridescence"?: never;
readonly "mapOffset"?: never;
readonly "mapRepeat"?: never;
readonly "mapRotation"?: never;
readonly "metalness"?: never;
readonly "normalScale"?: never;
readonly "opacity"?: never;
readonly "outlineColor"?: never;
readonly "outlineWidth"?: never;
readonly "rimColor"?: never;
readonly "rimIntensity"?: never;
readonly "rimPower"?: never;
readonly "roughness"?: never;
readonly "sheen"?: never;
readonly "sheenColor"?: never;
readonly "sheenRoughness"?: never;
readonly "thickness"?: never;
readonly "transmission"?: never;
readonly "transparent"?: never;
}) & Readonly<Record<string, (Input_timelineDefinitions["Uniform"])>>);
/** Only reflected fields actually driven by timeline presentation. */
readonly "standard"?: (((Input_timelineDefinitions["Reflected_og_cook::cook::source_schemas::media::timeline::StandardMaterial"])) | ((null)));
});
readonly "Music": ({
readonly "bus"?: (((Input_timelineDefinitions["Bus"])) | ((null)));
readonly "end"?: ((number) | (null));
readonly "fadeIn"?: ((number) | (null));
readonly "fadeOut"?: ((number) | (null));
readonly "loop"?: ((boolean) | (null));
readonly "sound": (Input_timelineDefinitions["ContentRef"]);
readonly "volume"?: ((number) | (null));
});
readonly "Nested": ({
readonly "loop"?: ((boolean) | (null));
readonly "play": (Input_timelineDefinitions["ContentRef"]);
readonly "speed"?: ((number) | (null));
});
readonly "Path": ({
readonly "closed"?: ((boolean) | (null));
readonly "kind"?: (((Input_timelineDefinitions["PathKind"])) | ((null)));
readonly "points": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "tension"?: ((number) | (null));
});
readonly "PathKind": ("catmullRom" | "bezier" | "linear") & (string);
readonly "Reflected_og_cook::cook::source_schemas::media::timeline::Blendshapes": ({
/** Weights of targets 8 to 63, as three.js morphTargetInfluences past the first eight (w0...w7): influences[k] drives target 8 + k; past the list's length 0. A face rig's 52 ARKit shapes fit. Clips write it with `source` clips, as they write w0...w7. */
readonly "influences"?: (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Weight of target 0. */
readonly "w0"?: (number);
/** Weight of target 1. */
readonly "w1"?: (number);
/** Weight of target 2. */
readonly "w2"?: (number);
/** Weight of target 3. */
readonly "w3"?: (number);
/** Weight of target 4. */
readonly "w4"?: (number);
/** Weight of target 5. */
readonly "w5"?: (number);
/** Weight of target 6. */
readonly "w6"?: (number);
/** Weight of target 7. */
readonly "w7"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::media::timeline::DirectionalLight": ({
/** The fraction of each shadow cascade's depth over which it fades into the next, hiding the seam where the shadow's resolution changes. 0 is none; 0.1 to 0.2 is typical (those pixels read two cascades). */
readonly "cascadeBlend"?: (number);
/** Cast shadows through the cascades (RenderSettings.shadowCascades), as three.js Light.castShadow. */
readonly "castShadow"?: (boolean);
/** The light's linear colour. */
readonly "color"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** The radiance multiplier, as three.js DirectionalLight.intensity. */
readonly "intensity"?: (number);
/** How the light may change while the game runs: what og bake lightmaps bakes of it and what is drawn every frame (stationary, the default; movable; static). See `render §4`. */
readonly "mobility"?: ("movable" | "stationary" | "static") & (string);
/** With shadowBox: the depth bias toward the light (three.js shadow.bias times its camera's depth range). 0 is one texel. Leaf cards need a few centimetres or they shadow each other dark. */
readonly "shadowBias"?: (number);
/** One shadow map over a fixed box instead of the cascades: its half size across the light. 0 keeps the cascades. Everything outside the box is lit; the casters are drawn once, not once per cascade (three.js games that fit the light's orthographic shadow camera around the player draw this way). */
readonly "shadowBox"?: (number);
/** How far from the camera shadows reach; they fade over the last tenth. Smaller is sharper. */
readonly "shadowDistance"?: (number);
/** With shadowBox: how far ahead of the camera (along its level forward) the box is centred. */
readonly "shadowFocus"?: (number);
/** With shadowBox: how far below the camera the box's centre sits (the ground under a first-person eye). */
readonly "shadowFocusDrop"?: (number);
/** With shadowBox: how far a receiver is pushed along its normal before it reads the map (three.js shadow.normalBias). 0 is one texel. */
readonly "shadowNormalBias"?: (number);
/** Soft shadows (PCSS): the penumbra's width per metre between caster and receiver, the tangent of the sun's apparent radius (the real sun's is 0.005). 0 keeps the hard one-texel edge; 0.01 to 0.05 is a soft sun, sharp at contact and blurring with distance. */
readonly "shadowSoftness"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::media::timeline::Light2D": ({
/** Occluders between the light and a pixel shade it (point and spot lights). */
readonly "castShadows"?: (boolean);
/** The light's linear colour. */
readonly "color"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Where the light reaches zero, in world units. */
readonly "distance"?: (number);
/** The exponent of the fade. */
readonly "falloff"?: (number);
/** How far above the 2D plane the light is, in world units: the slant normal-mapped sprites see it at (0 grazes the surface). */
readonly "height"?: (number);
/** A spot's half-angle at full intensity. */
readonly "innerAngle"?: (number);
/** The brightness at the centre. */
readonly "intensity"?: (number);
/** Point, spot or global. */
readonly "kind"?: ("point" | "spot" | "global") & (string);
/** The light layers it lights: a bit per layer, matched against each sprite's, rig's or tilemap's og.LightMask2D (layer 0, bit 1, without one). */
readonly "layers"?: (number);
/** A spot's half-angle where it reaches zero (at least innerAngle). */
readonly "outerAngle"?: (number);
/** The light's own size: larger gives softer shadows. */
readonly "sourceRadius"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::media::timeline::PointLight": ({
/** Cast shadows: six cube faces or two paraboloid hemispheres (RenderOptions.pointShadows). */
readonly "castShadow"?: (boolean);
/** The light's linear colour. */
readonly "color"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** A light cookie projected all around the light: six square faces stacked top to bottom as +X, -X, +Y, -Y, +Z, -Z, six times as tall as wide (a KTX2 cube's face order, a cube map's orientation, world axes). The light's colour is multiplied by the texel its direction meets. 0 is none. See `render §3`. */
readonly "cookie"?: ({
readonly "$ogRef": (string);
});
/** Where the light reaches zero, as three.js PointLight.distance (never infinite here). */
readonly "distance"?: (number);
/** Luminous intensity: the radiance at one metre for a zero-radius source, before the distance fade. */
readonly "intensity"?: (number);
/** How the light may change while the game runs: what og bake lightmaps bakes of it and what is drawn every frame (stationary, the default; movable; static). See `render §4`. */
readonly "mobility"?: ("movable" | "stationary" | "static") & (string);
/** The spherical source radius: softens near-source falloff and widens specular highlights. Zero keeps a point source; independent of distance and shadow softness. */
readonly "radius"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::media::timeline::PostEffects": ({
/** Screen-space ambient occlusion: darkens the ambient light in creases and contact areas. */
readonly "ao"?: (boolean);
/** How much the occlusion also darkens direct and local light (0 only the ambient light, 1 a fully occluded pixel keeps 35 % of it). Above 0 the AO sees and darkens water surfaces too; at 0 water is outside it. */
readonly "aoDirect"?: (number);
/** 0 none, 1 full; above 1 exaggerates. */
readonly "aoIntensity"?: (number);
/** The algorithm. */
readonly "aoMethod"?: ("auto" | "gtao" | "ssao" | "screen") & (string);
/** The world-space radius searched for occluders. */
readonly "aoRadius"?: (number);
/** Above 0 the radius grows with the distance, min(aoRadius, aoRadiusScale × distance), a fixed share of the screen up close as three.js SSAO passes size theirs; 0 keeps aoRadius everywhere. See `render §5`. */
readonly "aoRadiusScale"?: (number);
/** Half (the default) searches and blurs at half resolution with a normal- and depth-aware upsample; full searches every pixel. QualityPreset.high sets RenderSettings.aoResolution to full. */
readonly "aoResolution"?: ("full" | "half") & (string);
/** The colour balance of the highlights (luminance above 0.5, fading in from it), as balanceShadows. */
readonly "balanceHighlights"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The colour balance of the mid-tones (what the shadows and highlights leave), as balanceShadows. */
readonly "balanceMidtones"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The colour balance of the shadows (luminance below 0.5, fading out towards it): x shifts from cyan to red, y from magenta to green, z from yellow to blue, each -1 to 1 added to its channel; zero is off. */
readonly "balanceShadows"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** Added to every channel of the display colour; 0 is off. */
readonly "brightness"?: (number);
/** Red and blue split towards the edges, in pixels at the corners at 720p; 0 is off. */
readonly "chromaticAberration"?: (number);
/** Contrast around mid grey: positive spreads the tones by 1 / (1 - contrast), negative pulls them in by 1 + contrast (-1 is flat grey); 0 is off. */
readonly "contrast"?: (number);
/** Custom post passes in order (og.load.post), at most RenderCapabilities.maxCustomPosts (8), each over the one before, after depth of field and motion blur, before exposure and bloom, reading depth and normals (the prepass). A 0 is skipped, as is one failing to compile (og.render.postErrors). See `render §5`. */
readonly "custom"?: (ReadonlyArray<({
readonly "$ogRef": (string);
})> | (number extends (({
readonly "$ogRef": (string);
})) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Bokeh depth of field: pixels away from the focus distance blur into discs. */
readonly "depthOfField"?: (boolean);
/** How quickly blur grows outside the band: full blur focusRange times (1 + falloff) away. */
readonly "dofFalloff"?: (number);
/** The distance from the camera that is sharp. */
readonly "dofFocusDistance"?: (number);
/** The half-width of the sharp band around it. */
readonly "dofFocusRange"?: (number);
/** The largest blur radius, as a fraction of the image height (0.012 is about 9 px at 720p). */
readonly "dofMaxBlur"?: (number);
/** Taps per pixel. */
readonly "dofQuality"?: ("low" | "high") & (string);
/** Tilt-shift: above 0 the sharp band is a strip across the image's middle, this fraction of its height either side of the centre line, at any camera distance; blur grows above it as far and below it as near, full at dofTiltShift times (1 + dofFalloff) past it. 0 focuses by depth. See `looks §19`. */
readonly "dofTiltShift"?: (number);
/** Luminance-weighted noise amplitude in display units, a new pattern every tick (0.04 is subtle); 0 is off. */
readonly "filmGrain"?: (number);
/** The grain size in pixels. */
readonly "filmGrainSize"?: (number);
/** A full-screen flash: the display colour mixed towards flashColor by this much (animate it per tick); 0 is off. */
readonly "flash"?: (number);
/** The flash's linear display colour. */
readonly "flashColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Order-independent 2D glow: sprites with SpriteDesc.glow (and 2D particles with a glow) are blurred and added back, with or without bloom. */
readonly "glow"?: (boolean);
/** How much of the blurred glow is added back. */
readonly "glowIntensity"?: (number);
/** Screen-space light shafts from the DirectionalLight (the sun must be in front of the camera). The sun casts them with its own light, as a source of a fixed angular size whatever size its disc is drawn, dimmed by the clouds over it; the bright sky around it adds its own. */
readonly "godRays"?: (boolean);
/** How fast the rays fade along their length. */
readonly "godRaysDecay"?: (number);
/** How far towards the sun each ray samples (a fraction of the distance). */
readonly "godRaysDensity"?: (number);
/** How bright the shafts are. */
readonly "godRaysIntensity"?: (number);
/** Taps per ray. */
readonly "godRaysSamples"?: (number);
/** Brightness (linear, before exposure) below which the sky and the sun carry no rays: only what is brighter casts shafts. */
readonly "godRaysThreshold"?: (number);
/** A linear tint on the light the rays carry. */
readonly "godRaysTint"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** How far each colour goes towards its luminance (grey); 0 is off. */
readonly "grayscale"?: (number);
/** Turns every hue about the grey axis (red towards green at 120); 0 is off. */
readonly "hue"?: (number);
/** How far each colour goes towards its complement; 0 is off. */
readonly "invert"?: (number);
/** Barrel (positive: lines bow out) or pincushion (negative: in) distortion; 0 is off. Radius in pixels at any aspect; a barrel keeps its farthest corner on the image. After the depth-reading Stylise passes, before palette quantisation; the UI is not bent, pointer raycasts see the undistorted frame. See `render §5`. */
readonly "lensDistortion"?: (number);
/** The lens's centre, in fractions of the frame from the top left. */
readonly "lensDistortionCenter"?: (ReadonlyArray<(number)> & { readonly length: 2 });
/** Colour grading by a lookup table from og.load.lut; 0 is none (and so is a LUT whose streaming group is not resident). */
readonly "lut"?: ({
readonly "$ogRef": (string);
});
/** 0 leaves the image as tone mapped, 1 applies the table fully. */
readonly "lutIntensity"?: (number);
/** Per-object and camera motion blur along the screen-space velocity. */
readonly "motionBlur"?: (boolean);
/** The longest blur in pixels. */
readonly "motionBlurMaxPixels"?: (number);
/** Taps along the velocity. */
readonly "motionBlurSamples"?: (number);
/** The shutter as a fraction of the frame (0.5 is a 180 degree shutter). */
readonly "motionBlurShutter"?: (number);
/** Outlines every opaque surface from depth and normal edges, creases between the faces of one object included (materials with outlineWidth get their own hull outline instead). */
readonly "outline"?: (boolean);
/** The line colour, linear. */
readonly "outlineColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** The relative depth step that counts as a silhouette (0.04 is 4 %). */
readonly "outlineDepthThreshold"?: (number);
/** The normal change that counts as a crease (1 - cos of the angle; 0.4 is about 53 degrees). */
readonly "outlineNormalThreshold"?: (number);
/** Line width in pixels, drawn inside the silhouette. */
readonly "outlineWidth"?: (number);
/** The colour balance keeps each pixel's luminance, so it moves only the hue (Photoshop's Preserve Luminosity). */
readonly "preserveLuminosity"?: (boolean);
/** -1 grey, 0 unchanged, 1 twice the colour (the chroma around the luminance scaled by 1 + saturation). */
readonly "saturation"?: (number);
/** A zoom, spin or directional blur of the whole frame (RedGPU's zoom, radial and directional blurs), weighted to the middle of its reach: one pass after motion blur and before bloom and the tone curve, so highlights streak and glow; the UI is not blurred. Off at none or at screenBlurStrength 0. */
readonly "screenBlur"?: ("none" | "zoom" | "spin" | "directional") & (string);
/** Directional's direction, counter-clockwise from the right (90 is vertical). */
readonly "screenBlurAngle"?: (number);
/** Zoom's and spin's centre, in fractions of the frame from the top left. */
readonly "screenBlurCenter"?: (ReadonlyArray<(number)> & { readonly length: 2 });
/** Taps per pixel for spin and directional. Zoom exceeds this quality floor with three eight-tap stages (512 effective positions), or direct adaptive sampling when strength >= 1 crosses the centre. Both use stable per-pixel jitter. */
readonly "screenBlurSamples"?: (number);
/** How far it reaches each way: for zoom the fraction of the way to the centre, for spin radians, for directional pixels at 720 lines; 0 is off. */
readonly "screenBlurStrength"?: (number);
/** How fast the shake moves. */
readonly "shakeFrequency"?: (number);
/** The largest offset at trauma 1, as a fraction of the view height. */
readonly "shakeMaxOffset"?: (number);
/** The largest roll at trauma 1. */
readonly "shakeMaxRoll"?: (number);
/** The shake's noise seed. */
readonly "shakeSeed"?: (number);
/** Camera shake held at a fixed level (decay it yourself): 0 none, 1 the most (the amplitude is trauma squared). Prefer og.camera.shake, which decays by itself; the frame shakes by the larger of the two. Smooth value noise by the tick, so a replay shakes the same. Visual only. */
readonly "shakeTrauma"?: (number);
/** Contrast-adaptive sharpening (CAS) of the final image, as a three.js CAS pass after the output pass: each pixel pushed away from its four neighbours by a weight that falls where the neighbourhood is already near black or white. 0 is the mildest, 1 the strongest; below 0 is off. */
readonly "sharpen"?: (number);
/** Stochastic, denoised screen-space reflections (after FidelityFX SSSR and Frostbite): a GGX-lobe ray a pixel up to ssrMaxRoughness through a depth pyramid, reflecting the previous frame's lit image; on a miss, the probes', probe GI's and environment's. Opaque, perspective 3D; turns on the prepass. See `render §5`. */
readonly "ssr"?: (boolean);
/** The fraction of the screen's edge over which a reflection fades into the probes' (the last fifth of ssrMaxDistance fades too). */
readonly "ssrFade"?: (number);
/** How much of the reflection replaces the probes' and the environment's where it resolved; 1 is physical (weighted by the surface's Fresnel and lobe as the probes' are). */
readonly "ssrIntensity"?: (number);
/** How far a reflected ray marches, in world units; the last fifth fades into the probes'. The march's cost is set by ssrSteps, not by the distance. */
readonly "ssrMaxDistance"?: (number);
/** The roughest surface traced: rougher ones keep the probes' reflection, and the last quarter below it fades into them. See `render §5`. */
readonly "ssrMaxRoughness"?: (number);
/** Half (the default): one ray per 2x2 block, a different pixel of the block each frame, denoised on the blocks and upsampled to the pixels by their depth, normal and roughness. Full: one ray per pixel, denoised at full resolution. See `render §5`. */
readonly "ssrResolution"?: ("full" | "half") & (string);
/** The hierarchical march's iterations along each ray, each crossing a cell of the min-max depth pyramid's current level; a ray out of steps keeps the probes' reflection. Only the rays that need the steps pay for them. See `render §5`. */
readonly "ssrSteps"?: (number);
/** How far behind the depth buffer a surface is taken to extend: a ray passing behind a surface by less than this hits it. */
readonly "ssrThickness"?: (number);
/** White balance: towards warm orange (positive) or cool blue (negative), Unity's Temperature divided by 100 (a von Kries scaling in LMS from a white point moved along the daylight locus); 0 is off. See `render §5`. */
readonly "temperature"?: (number);
/** Above 0, each pixel white where its luminance reaches this level and black below it (the table's 32 steps soften the edge over about 1/31 of the range); 0 is off. */
readonly "threshold"?: (number);
/** White balance: towards magenta (positive) or green (negative), as Lightroom's and Unity's Tint divided by 100; 0 is off. */
readonly "tint"?: (number);
/** Saturation that spares colours already saturated and skin tones: positive livens dull colours, negative mutes them; 0 is off. */
readonly "vibrance"?: (number);
/** Darkened (or tinted) edges of the frame. */
readonly "vignette"?: (boolean);
/** The linear colour the edges move towards. */
readonly "vignetteColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** 0 none, 1 black corners. */
readonly "vignetteIntensity"?: (number);
/** Where darkening starts, as a fraction of the half-diagonal. */
readonly "vignetteRadius"?: (number);
/** The width of the fade. */
readonly "vignetteSmoothness"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::media::timeline::RectLight": ({
/** Cast a soft shadow: a shadow map from the rectangle's centre (two when twoSided, counted in the shadowed lights' budget) read with percentage-closer soft shadows, sharp at contact and widening with the rectangle's size away from it. See `render §3`. */
readonly "castShadow"?: (boolean);
/** The light's linear colour. */
readonly "color"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Its size along the entity's local y, as RectAreaLight.height. */
readonly "height"?: (number);
/** The emitted radiance (luminance), as three.js RectAreaLight.intensity. */
readonly "intensity"?: (number);
/** How the light may change while the game runs: what og bake lightmaps bakes of it and what is drawn every frame (stationary, the default; movable; static). See `render §4`. */
readonly "mobility"?: ("movable" | "stationary" | "static") & (string);
/** Emit from the back face as well (a hanging panel lit both ways). */
readonly "twoSided"?: (boolean);
/** Its size along the entity's local x, as RectAreaLight.width. */
readonly "width"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::media::timeline::SpotLight": ({
/** The half-angle of the outer cone, as three.js SpotLight.angle. */
readonly "angle"?: (number);
/** Cast a shadow map. */
readonly "castShadow"?: (boolean);
/** The light's linear colour. */
readonly "color"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** A light cookie, as three.js SpotLight.map: the texture projected along the cone (its centre on the axis, its edges at the outer angle, its top towards world up), multiplying the light's colour. 0 is none. A shadowed spot keeps its shadow over the pattern. See `render §3`. */
readonly "cookie"?: ({
readonly "$ogRef": (string);
});
/** Where the light reaches zero. */
readonly "distance"?: (number);
/** Luminous intensity on the axis: the radiance at one metre for a zero-radius source, before the distance fade. */
readonly "intensity"?: (number);
/** How the light may change while the game runs: what og bake lightmaps bakes of it and what is drawn every frame (stationary, the default; movable; static). See `render §4`. */
readonly "mobility"?: ("movable" | "stationary" | "static") & (string);
/** The fraction of the cone over which the light fades out, as three.js SpotLight.penumbra. */
readonly "penumbra"?: (number);
/** The spherical source radius: softens near-source falloff and widens specular highlights. Zero keeps a point source; independent of distance and shadow softness. */
readonly "radius"?: (number);
});
/** A physically based metallic-roughness material with three.js
`MeshStandardMaterial` parameter names, reflected: its schema
is what the inspector, Tune and the probe edit,
and its fields match `render.StandardMaterialDesc` in the IDL.

The `MeshPhysicalMaterial` fields (clearcoat maps, sheen, transmission,
iridescence, anisotropy, `ior`) draw through the physical lit pipelines
([`StandardMaterial::is_physical`]); a material without them draws
exactly as a `MeshStandardMaterial`.

Texture fields are slots: `0` is none, otherwise
[`TextureHandle::slot`] (the texture's index plus one). Boolean fields
are `0` or `1`. Colours are linear. The default is three.js's: white,
metalness 0, roughness 1, opaque, front side. */
readonly "Reflected_og_cook::cook::source_schemas::media::timeline::StandardMaterial": ({
/** Pixels whose alpha is below this are discarded (0: none). */
readonly "alphaTest"?: (number);
/** Stretches the specular highlight along a direction of the surface
(three.js `anisotropy`, glTF `KHR_materials_anisotropy`): brushed
metal, hair, satin. 0: none, 1: fully stretched. */
readonly "anisotropy"?: (number);
/** The stretch's direction: radians from the tangent (+u) towards the bitangent. */
readonly "anisotropyRotation"?: (number);
/**  */
readonly "aoMapIntensity"?: (number);
/** The colour white light becomes after `attenuationDistance` in the volume (linear RGB). */
readonly "attenuationColor"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The distance light travels in the volume before it is tinted to
`attenuationColor` (world units). 0: no absorption (three.js's Infinity). */
readonly "attenuationDistance"?: (number);
/** A clear lacquer over the surface (three.js `MeshPhysicalMaterial`
`clearcoat`): a second specular layer, 4 % reflective, on the
surface's normal before the normal map. 0: none. */
readonly "clearcoat"?: (number);
/** How rough the coat is. */
readonly "clearcoatRoughness"?: (number);
/** The base colour (linear RGB), multiplied with `map` and vertex colours. */
readonly "color"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The dissolved fraction: 0 whole, 1 gone. Pixels whose noise is below
it are discarded (also from shadows and the prepass); the band just
above glows with `dissolveEdgeColor`. */
readonly "dissolve"?: (number);
/** The glowing edge (linear RGB, usually above 1 so it blooms). */
readonly "dissolveEdgeColor"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** Width of the glowing band in noise units. */
readonly "dissolveEdgeWidth"?: (number);
/** Frequency of the built-in noise (cells per object-space unit). */
readonly "dissolveScale"?: (number);
/** Emitted light (linear RGB), multiplied with `emissiveMap`. */
readonly "emissive"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** A multiplier on the emitted light. */
readonly "emissiveIntensity"?: (number);
/** A multiplier on the image-based (environment) light. */
readonly "envMapIntensity"?: (number);
/** How far the final colour is mixed towards `flashColor` (0 none, 1 flat). */
readonly "flash"?: (number);
/** The colour a hit flash mixes towards (linear RGB, may exceed 1). */
readonly "flashColor"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The index of refraction (glTF `KHR_materials_ior`): the dielectric
reflectance ((ior - 1) / (ior + 1))², 4 % at the default 1.5, and
how far transmission refracts. */
readonly "ior"?: (number);
/** A thin-film interference layer (three.js `iridescence`, glTF
`KHR_materials_iridescence`): soap bubbles, oil, beetle shells. 0: none. */
readonly "iridescence"?: (number);
/** How metallic the surface is. */
readonly "metalness"?: (number);
/** How strongly the normal map bends the normal, per tangent axis. */
readonly "normalScale"?: (ReadonlyArray<(number)> & { readonly length: 2 });
/** The opacity; below 1 only shows with `transparent`. */
readonly "opacity"?: (number);
/** This object's outline colour (linear RGB) in the outline pass. */
readonly "outlineColor"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** This object's outline width in pixels (0: the scene's
`PostEffects::outline`, if any). Turns the outline pass on. */
readonly "outlineWidth"?: (number);
/** Rim (fresnel) light added at grazing angles, unshadowed (linear RGB). */
readonly "rimColor"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** 0: no rim light. */
readonly "rimIntensity"?: (number);
/** How tight the rim is (higher: thinner). */
readonly "rimPower"?: (number);
/** How rough the surface is. */
readonly "roughness"?: (number);
/** A velvet-like retroreflective layer (three.js `sheen`, glTF
`KHR_materials_sheen`): its strength times `sheenColor`. 0: none. */
readonly "sheen"?: (number);
/** The sheen's colour (linear RGB), multiplied with `sheenColorMap`. */
readonly "sheenColor"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** How rough the sheen is: low is a thin rim, high spreads it over the surface. */
readonly "sheenRoughness"?: (number);
/** The volume's thickness under the surface in object-space units
(glTF `KHR_materials_volume`): how far refraction bends the view
and how much `attenuationColor` absorbs. 0: a thin wall. */
readonly "thickness"?: (number);
/** How much light passes through the surface (three.js `transmission`,
glTF `KHR_materials_transmission`): glass, water, thin plastic. The
opaque scene behind it is refracted, blurred by `roughness`. 0: none. */
readonly "transmission"?: (number);
/** 1: blended over the opaque scene, sorted back to front, no depth write. */
readonly "transparent"?: (number);
});
readonly "Scene": ({
/** Null clears the scene at this key. */
readonly "scene"?: (((Input_timelineDefinitions["ContentRef"])) | ((null)));
});
readonly "Shot": ({
readonly "blend"?: ((number) | (null));
readonly "camera"?: ((string) | (null));
});
readonly "Span": ({
readonly "color"?: ((string) | (null));
readonly "length": (number);
readonly "name": (string);
readonly "position": (number);
});
readonly "Track": (((({
readonly "blendIn"?: ((number) | (null));
readonly "blendOut"?: ((number) | (null));
readonly "follow"?: ((string) | (null));
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key"])> | (number extends ((Input_timelineDefinitions["Key"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("camera") & (string);
readonly "rail"?: ((string) | (null));
})) & { readonly "entity"?: never; readonly "field"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; }) | ((({
readonly "entity": (string);
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key2"])> | (number extends ((Input_timelineDefinitions["Key2"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("transform") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "entity": (string);
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key3"])> | (number extends ((Input_timelineDefinitions["Key3"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("animation") & (string);
readonly "model"?: (((Input_timelineDefinitions["ContentRef"])) | ((null)));
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key4"])> | (number extends ((Input_timelineDefinitions["Key4"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("audio") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key5"])> | (number extends ((Input_timelineDefinitions["Key5"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("music") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "imports"?: (ReadonlyArray<(Input_timelineDefinitions["CaptionImport"])> | (number extends ((Input_timelineDefinitions["CaptionImport"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "keys"?: (ReadonlyArray<(Input_timelineDefinitions["Key6"])> | (number extends ((Input_timelineDefinitions["Key6"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("caption") & (string);
readonly "locales"?: ((Readonly<Record<string, (Input_timelineDefinitions["CaptionCues"])>>) | (null));
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key7"])> | (number extends ((Input_timelineDefinitions["Key7"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("event") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key8"])> | (number extends ((Input_timelineDefinitions["Key8"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("transition") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key9"])> | (number extends ((Input_timelineDefinitions["Key9"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("marker") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "entity"?: ((string) | (null));
readonly "field": (string);
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key10"])> | (number extends ((Input_timelineDefinitions["Key10"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("property") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "entity": (string);
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key11"])> | (number extends ((Input_timelineDefinitions["Key11"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("light") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key12"])> | (number extends ((Input_timelineDefinitions["Key12"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("post") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "entity": (string);
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key13"])> | (number extends ((Input_timelineDefinitions["Key13"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("visibility") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "entity": (string);
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key14"])> | (number extends ((Input_timelineDefinitions["Key14"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("blendshape") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "entity": (string);
readonly "index"?: ((number) | (null));
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key15"])> | (number extends ((Input_timelineDefinitions["Key15"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("material") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key16"])> | (number extends ((Input_timelineDefinitions["Key16"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("timeline") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key17"])> | (number extends ((Input_timelineDefinitions["Key17"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("shot") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key18"])> | (number extends ((Input_timelineDefinitions["Key18"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("vfx") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "entity"?: ((string) | (null));
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key19"])> | (number extends ((Input_timelineDefinitions["Key19"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("video") & (string);
readonly "layer"?: (((Input_timelineDefinitions["VideoLayer"])) | ((null)));
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key20"])> | (number extends ((Input_timelineDefinitions["Key20"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("hotspot") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(Input_timelineDefinitions["Key21"])> | (number extends ((Input_timelineDefinitions["Key21"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("scene") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }));
readonly "Transform": ({
readonly "position"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "quaternion"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
/** Euler XYZ degrees; exclusive with quaternion (domain check). */
readonly "rotation"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "scale"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
});
readonly "Transition": ({
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "duration"?: ((number) | (null));
readonly "from"?: (((Input_timelineDefinitions["Direction"])) | ((null)));
readonly "kind"?: (((Input_timelineDefinitions["TransitionKind"])) | ((null)));
readonly "mode"?: (((Input_timelineDefinitions["TransitionMode"])) | ((null)));
});
readonly "TransitionKind": ("cut" | "fade" | "wipe" | "dissolve" | "iris" | "slide") & (string);
readonly "TransitionMode": ("through" | "in" | "out") & (string);
readonly "Uniform": (((number)) | ((ReadonlyArray<(number)> & { readonly length: 2 })) | ((ReadonlyArray<(number)> & { readonly length: 3 })) | ((ReadonlyArray<(number)> & { readonly length: 4 })));
readonly "Vfx": ({
readonly "at"?: ((string) | (null));
readonly "effect": (Input_timelineDefinitions["ContentRef"]);
readonly "position"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "scale"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
});
readonly "Video": ({
readonly "from"?: ((number) | (null));
readonly "speed"?: ((number) | (null));
readonly "video": (Input_timelineDefinitions["ContentRef"]);
});
readonly "VideoLayer": ("over" | "under") & (string);
}
export type Input_ui_material = ({
/** WGSL defining fn material(m: MaterialInput, p: Params) -> vec4<f32>. */
readonly "code": (string);
readonly "params"?: (ReadonlyArray<(Input_ui_materialDefinitions["Param"])> | (number extends ((Input_ui_materialDefinitions["Param"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
export interface Input_ui_materialDefinitions {
readonly "Param": ((({
readonly "default"?: ((number) | (null));
readonly "kind": ("float") & (string);
readonly "name": (string);
})) | (({
readonly "default"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "kind": ("vec2") & (string);
readonly "name": (string);
})) | (({
readonly "default"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "kind": ("vec3") & (string);
readonly "name": (string);
})) | (({
readonly "default"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "kind": ("vec4") & (string);
readonly "name": (string);
})) | (({
readonly "default"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "kind": ("color") & (string);
readonly "name": (string);
})));
}
/** Exactly one SVG source. SVG syntax/element support belongs to the vector
domain validator. No embedded OGT or untyped configuration is accepted. */
export type Input_vector = (((({
/** Explicit canonical SVG path relative to the owning module
(e.g. assets/ui/emblem.svg), without sibling/assets fallback. */
readonly "source": (string);
/** Positive flattening error in SVG view-box units; omitted uses the
longer view-box side divided by 2000. */
readonly "tolerance"?: ((number) | (null));
})) & { readonly "svg"?: never; }) | ((({
readonly "svg": (string);
/** Positive flattening error in SVG view-box units. */
readonly "tolerance"?: ((number) | (null));
})) & { readonly "source"?: never; }));
/** A composed effect timeline. Element end times, parameter identities and
duplicate event names are checked by the domain validator. */
export type Input_vfx = ({
readonly "duration": (number);
readonly "elements": (ReadonlyArray<(Input_vfxDefinitions["VfxElement"])> | (number extends ((Input_vfxDefinitions["VfxElement"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "events"?: ((ReadonlyArray<(Input_vfxDefinitions["VfxEvent"])> | (number extends ((Input_vfxDefinitions["VfxEvent"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "looping"?: ((boolean) | (null));
readonly "params"?: ((ReadonlyArray<(Input_vfxDefinitions["VfxParam"])> | (number extends ((Input_vfxDefinitions["VfxParam"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
});
export interface Input_vfxDefinitions {
readonly "Blend": ("alpha" | "additive" | "multiply" | "subtractive") & (string);
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
/** A scalar constant, per-instance range or ordered lifetime curve. */
readonly "EffectValue": (((number)) | ((Input_vfxDefinitions["ValueInput"])));
readonly "RingInput": ({
readonly "band"?: ((number) | (null));
readonly "colors"?: ((ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 4 })> & { readonly length: 3 }) | (null));
readonly "height"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "inner"?: ((number) | (null));
readonly "length"?: ((number) | (null));
readonly "segments"?: ((number) | (null));
readonly "start"?: ((number) | (null));
});
readonly "SpriteBlend": ("alpha" | "additive") & (string);
readonly "ValueInput": (((({
readonly "kind": ("random") & (string);
readonly "max": (number);
readonly "min": (number);
})) & { readonly "keys"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("curve") & (string);
})) & { readonly "max"?: never; readonly "min"?: never; }));
readonly "VfxElement": ({
readonly "acceleration"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "amount"?: ((string) | (null));
readonly "at"?: ((number) | (null));
readonly "life"?: ((number) | (null));
readonly "node": (Input_vfxDefinitions["VfxNode"]);
readonly "offset"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "rotate"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
/** End time instead of life; both together are a domain error. */
readonly "until"?: ((number) | (null));
readonly "velocity"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "when"?: ((string) | (null));
});
readonly "VfxEvent": ({
/** At most 63 UTF-8 bytes (domain validation, not character count). */
readonly "name": (string);
readonly "time": (number);
});
readonly "VfxNode": (((({
readonly "asset": (Input_vfxDefinitions["ContentRef"]);
readonly "kind": ("emitter") & (string);
readonly "seed"?: ((number) | (null));
})) & { readonly "angleFade"?: never; readonly "blend"?: never; readonly "color"?: never; readonly "distance"?: never; readonly "effect"?: never; readonly "emissive"?: never; readonly "faceCamera"?: never; readonly "fade"?: never; readonly "intensity"?: never; readonly "jitter"?: never; readonly "lifetime"?: never; readonly "metallic"?: never; readonly "normal"?: never; readonly "normalScale"?: never; readonly "orbit"?: never; readonly "pitch"?: never; readonly "rise"?: never; readonly "roughness"?: never; readonly "scale"?: never; readonly "segments"?: never; readonly "shape"?: never; readonly "size"?: never; readonly "spin"?: never; readonly "stretch"?: never; readonly "texture"?: never; readonly "to"?: never; readonly "trauma"?: never; readonly "turn"?: never; readonly "uv"?: never; readonly "volume"?: never; readonly "wet"?: never; readonly "width"?: never; }) | ((({
readonly "blend"?: (((Input_vfxDefinitions["Blend"])) | ((null)));
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "effect": (Input_vfxDefinitions["ContentRef"]);
readonly "faceCamera"?: ((boolean) | (null));
readonly "kind": ("mesh") & (string);
readonly "scale"?: (((Input_vfxDefinitions["EffectValue"])) | ((null)));
readonly "shape": (Input_vfxDefinitions["VfxShape"]);
readonly "spin"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "stretch"?: ((string) | (null));
})) & { readonly "angleFade"?: never; readonly "asset"?: never; readonly "distance"?: never; readonly "emissive"?: never; readonly "fade"?: never; readonly "intensity"?: never; readonly "jitter"?: never; readonly "lifetime"?: never; readonly "metallic"?: never; readonly "normal"?: never; readonly "normalScale"?: never; readonly "orbit"?: never; readonly "pitch"?: never; readonly "rise"?: never; readonly "roughness"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "size"?: never; readonly "texture"?: never; readonly "to"?: never; readonly "trauma"?: never; readonly "turn"?: never; readonly "uv"?: never; readonly "volume"?: never; readonly "wet"?: never; readonly "width"?: never; }) | ((({
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "effect"?: (((Input_vfxDefinitions["ContentRef"])) | ((null)));
readonly "kind": ("trail") & (string);
readonly "lifetime"?: ((number) | (null));
/** Radius in metres and turns per second about local Y. */
readonly "orbit"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "rise"?: ((number) | (null));
readonly "width"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
})) & { readonly "angleFade"?: never; readonly "asset"?: never; readonly "blend"?: never; readonly "distance"?: never; readonly "emissive"?: never; readonly "faceCamera"?: never; readonly "fade"?: never; readonly "intensity"?: never; readonly "jitter"?: never; readonly "metallic"?: never; readonly "normal"?: never; readonly "normalScale"?: never; readonly "pitch"?: never; readonly "roughness"?: never; readonly "scale"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "shape"?: never; readonly "size"?: never; readonly "spin"?: never; readonly "stretch"?: never; readonly "texture"?: never; readonly "to"?: never; readonly "trauma"?: never; readonly "turn"?: never; readonly "uv"?: never; readonly "volume"?: never; readonly "wet"?: never; }) | ((({
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "effect"?: (((Input_vfxDefinitions["ContentRef"])) | ((null)));
readonly "jitter"?: ((number) | (null));
readonly "kind": ("beam") & (string);
readonly "segments"?: ((number) | (null));
readonly "stretch"?: ((string) | (null));
readonly "to"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "width"?: ((number) | (null));
})) & { readonly "angleFade"?: never; readonly "asset"?: never; readonly "blend"?: never; readonly "distance"?: never; readonly "emissive"?: never; readonly "faceCamera"?: never; readonly "fade"?: never; readonly "intensity"?: never; readonly "lifetime"?: never; readonly "metallic"?: never; readonly "normal"?: never; readonly "normalScale"?: never; readonly "orbit"?: never; readonly "pitch"?: never; readonly "rise"?: never; readonly "roughness"?: never; readonly "scale"?: never; readonly "seed"?: never; readonly "shape"?: never; readonly "size"?: never; readonly "spin"?: never; readonly "texture"?: never; readonly "trauma"?: never; readonly "turn"?: never; readonly "uv"?: never; readonly "volume"?: never; readonly "wet"?: never; }) | ((({
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "distance"?: ((number) | (null));
readonly "intensity"?: (((Input_vfxDefinitions["EffectValue"])) | ((null)));
readonly "kind": ("light") & (string);
})) & { readonly "angleFade"?: never; readonly "asset"?: never; readonly "blend"?: never; readonly "effect"?: never; readonly "emissive"?: never; readonly "faceCamera"?: never; readonly "fade"?: never; readonly "jitter"?: never; readonly "lifetime"?: never; readonly "metallic"?: never; readonly "normal"?: never; readonly "normalScale"?: never; readonly "orbit"?: never; readonly "pitch"?: never; readonly "rise"?: never; readonly "roughness"?: never; readonly "scale"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "shape"?: never; readonly "size"?: never; readonly "spin"?: never; readonly "stretch"?: never; readonly "texture"?: never; readonly "to"?: never; readonly "trauma"?: never; readonly "turn"?: never; readonly "uv"?: never; readonly "volume"?: never; readonly "wet"?: never; readonly "width"?: never; }) | ((({
readonly "angleFade"?: ((number) | (null));
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
/** Linear emission; HDR values above one are allowed. */
readonly "emissive"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "fade"?: ((number) | (null));
readonly "kind": ("decal") & (string);
/** Omitted preserves the receiving surface's metallic value. */
readonly "metallic"?: ((number) | (null));
/** Tangent-space glTF normal map: +X along U, +Y toward image top.
Shares the colour texture's UV region; omitted leaves normals alone. */
readonly "normal"?: (((Input_vfxDefinitions["ContentRef"])) | ((null)));
readonly "normalScale"?: ((number) | (null));
/** Omitted preserves the receiving surface's roughness. */
readonly "roughness"?: ((number) | (null));
readonly "size"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
/** Omitted means a solid-colour decal. */
readonly "texture"?: (((Input_vfxDefinitions["ContentRef"])) | ((null)));
/** Seeded per-instance rotation interval about the projection axis. */
readonly "turn"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** Region of this decal's texture and normal, not a global atlas. */
readonly "uv"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "wet"?: ((boolean) | (null));
})) & { readonly "asset"?: never; readonly "blend"?: never; readonly "distance"?: never; readonly "effect"?: never; readonly "faceCamera"?: never; readonly "intensity"?: never; readonly "jitter"?: never; readonly "lifetime"?: never; readonly "orbit"?: never; readonly "pitch"?: never; readonly "rise"?: never; readonly "scale"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "shape"?: never; readonly "spin"?: never; readonly "stretch"?: never; readonly "to"?: never; readonly "trauma"?: never; readonly "volume"?: never; readonly "width"?: never; }) | ((({
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "kind": ("glow") & (string);
readonly "size"?: (((Input_vfxDefinitions["EffectValue"])) | ((null)));
readonly "stretch"?: ((string) | (null));
})) & { readonly "angleFade"?: never; readonly "asset"?: never; readonly "blend"?: never; readonly "distance"?: never; readonly "effect"?: never; readonly "emissive"?: never; readonly "faceCamera"?: never; readonly "fade"?: never; readonly "intensity"?: never; readonly "jitter"?: never; readonly "lifetime"?: never; readonly "metallic"?: never; readonly "normal"?: never; readonly "normalScale"?: never; readonly "orbit"?: never; readonly "pitch"?: never; readonly "rise"?: never; readonly "roughness"?: never; readonly "scale"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "shape"?: never; readonly "spin"?: never; readonly "texture"?: never; readonly "to"?: never; readonly "trauma"?: never; readonly "turn"?: never; readonly "uv"?: never; readonly "volume"?: never; readonly "wet"?: never; readonly "width"?: never; }) | ((({
readonly "kind": ("shake") & (string);
readonly "trauma"?: ((number) | (null));
})) & { readonly "angleFade"?: never; readonly "asset"?: never; readonly "blend"?: never; readonly "color"?: never; readonly "distance"?: never; readonly "effect"?: never; readonly "emissive"?: never; readonly "faceCamera"?: never; readonly "fade"?: never; readonly "intensity"?: never; readonly "jitter"?: never; readonly "lifetime"?: never; readonly "metallic"?: never; readonly "normal"?: never; readonly "normalScale"?: never; readonly "orbit"?: never; readonly "pitch"?: never; readonly "rise"?: never; readonly "roughness"?: never; readonly "scale"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "shape"?: never; readonly "size"?: never; readonly "spin"?: never; readonly "stretch"?: never; readonly "texture"?: never; readonly "to"?: never; readonly "turn"?: never; readonly "uv"?: never; readonly "volume"?: never; readonly "wet"?: never; readonly "width"?: never; }) | ((({
readonly "asset": (Input_vfxDefinitions["ContentRef"]);
readonly "kind": ("sound") & (string);
/** Playback-rate interval; 1 is the recording's original pitch. */
readonly "pitch"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "volume"?: ((number) | (null));
})) & { readonly "angleFade"?: never; readonly "blend"?: never; readonly "color"?: never; readonly "distance"?: never; readonly "effect"?: never; readonly "emissive"?: never; readonly "faceCamera"?: never; readonly "fade"?: never; readonly "intensity"?: never; readonly "jitter"?: never; readonly "lifetime"?: never; readonly "metallic"?: never; readonly "normal"?: never; readonly "normalScale"?: never; readonly "orbit"?: never; readonly "rise"?: never; readonly "roughness"?: never; readonly "scale"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "shape"?: never; readonly "size"?: never; readonly "spin"?: never; readonly "stretch"?: never; readonly "texture"?: never; readonly "to"?: never; readonly "trauma"?: never; readonly "turn"?: never; readonly "uv"?: never; readonly "wet"?: never; readonly "width"?: never; }) | ((({
readonly "asset": (Input_vfxDefinitions["ContentRef"]);
readonly "blend"?: (((Input_vfxDefinitions["SpriteBlend"])) | ((null)));
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "kind": ("flipbook") & (string);
readonly "size"?: ((number) | (null));
})) & { readonly "angleFade"?: never; readonly "distance"?: never; readonly "effect"?: never; readonly "emissive"?: never; readonly "faceCamera"?: never; readonly "fade"?: never; readonly "intensity"?: never; readonly "jitter"?: never; readonly "lifetime"?: never; readonly "metallic"?: never; readonly "normal"?: never; readonly "normalScale"?: never; readonly "orbit"?: never; readonly "pitch"?: never; readonly "rise"?: never; readonly "roughness"?: never; readonly "scale"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "shape"?: never; readonly "spin"?: never; readonly "stretch"?: never; readonly "texture"?: never; readonly "to"?: never; readonly "trauma"?: never; readonly "turn"?: never; readonly "uv"?: never; readonly "volume"?: never; readonly "wet"?: never; readonly "width"?: never; }));
readonly "VfxParam": ({
readonly "default": (number);
readonly "name": (string);
});
readonly "VfxShape": (((({
readonly "kind": ("sphere") & (string);
})) & { readonly "asset"?: never; readonly "ring"?: never; }) | ((({
readonly "kind": ("quad") & (string);
})) & { readonly "asset"?: never; readonly "ring"?: never; }) | ((({
readonly "kind": ("box") & (string);
})) & { readonly "asset"?: never; readonly "ring"?: never; }) | ((({
readonly "kind": ("cylinder") & (string);
})) & { readonly "asset"?: never; readonly "ring"?: never; }) | ((({
readonly "kind": ("ring") & (string);
readonly "ring"?: (((Input_vfxDefinitions["RingInput"])) | ((null)));
})) & { readonly "asset"?: never; }) | ((({
readonly "asset": (Input_vfxDefinitions["ContentRef"]);
readonly "kind": ("model") & (string);
})) & { readonly "ring"?: never; }));
}
export type Input_vfx2d = ({
readonly "duration"?: ((number) | (null));
readonly "events"?: (ReadonlyArray<(Input_vfx2dDefinitions["Event"])> | (number extends ((Input_vfx2dDefinitions["Event"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "impact"?: ((number) | (null));
readonly "loop"?: ((boolean) | (null));
readonly "recipe": (Input_vfx2dDefinitions["Recipe"]);
});
export interface Input_vfx2dDefinitions {
readonly "Blend": ("over" | "under") & (string);
readonly "BoltMode": ("radial" | "strike" | "line") & (string);
readonly "Core": ("dark" | "light") & (string);
readonly "Event": ({
readonly "name": (string);
readonly "time": (number);
});
readonly "Layer": (((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "cool"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "drift"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "erode"?: ((number) | (null));
readonly "erodeStart"?: ((number) | (null));
readonly "evolve"?: ((number) | (null));
readonly "freq"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "grow"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hollow"?: ((number) | (null));
readonly "hue"?: ((number) | (null));
readonly "r0"?: ((number) | (null));
readonly "r1"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "rough"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "type": ("burst") & (string);
readonly "wobble"?: ((number) | (null));
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "breakup"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "fade"?: ((number) | (null));
readonly "freq"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "inner"?: ((number) | (null));
readonly "r0"?: ((number) | (null));
readonly "r1"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "th"?: ((number) | (null));
readonly "thEnd"?: ((number) | (null));
readonly "type": ("ring") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "flick"?: ((number) | (null));
readonly "freq"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hgt"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hollow"?: ((number) | (null));
readonly "hue"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "rough"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "speed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "taper"?: ((number) | (null));
readonly "type": ("flame") & (string);
readonly "w"?: ((number) | (null));
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flicker"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hold"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "cap"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "freq"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "grow"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "len"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "speed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "th"?: ((number) | (null));
readonly "type": ("beam") & (string);
readonly "wob"?: ((number) | (null));
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "a0"?: ((number) | (null));
readonly "a1"?: ((number) | (null));
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "erode"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "r"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
readonly "sweep"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "th"?: ((number) | (null));
readonly "trail"?: ((number) | (null));
readonly "type": ("slash") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "freq"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "grow"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "pulse"?: ((number) | (null));
readonly "pulseRate"?: ((number) | (null));
readonly "r"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "rim"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "spin"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "type": ("orb") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "freq"?: ((number) | (null));
readonly "from"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "len"?: ((number) | (null));
readonly "r"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "rough"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "speed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "type": ("comet") & (string);
readonly "until"?: ((number) | (null));
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "arms"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "core"?: (((Input_vfx2dDefinitions["Core"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dir"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "freq"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "r"?: ((number) | (null));
readonly "rIn"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "spin"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "twist"?: ((number) | (null));
readonly "type": ("vortex") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "freq"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "grow"?: ((number) | (null));
readonly "hgt"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "rough"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "speed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "type": ("pillar") & (string);
readonly "w"?: ((number) | (null));
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "count"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "inner"?: ((number) | (null));
readonly "jitter"?: ((number) | (null));
readonly "r"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "spin"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "twinkle"?: ((number) | (null));
readonly "type": ("rays") & (string);
readonly "width"?: ((number) | (null));
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "jag"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "draw"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "fill"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "r"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "rings"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "runes"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "spin"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
readonly "star"?: ((number) | (null));
readonly "step"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "ticks"?: ((number) | (null));
readonly "type": ("glyph") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "cone"?: ((number) | (null));
readonly "count"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "decay"?: ((number) | (null));
readonly "dir"?: ((number) | (null));
readonly "dissolve"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "drift"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "emit"?: ((number) | (null));
readonly "flat"?: ((number) | (null));
readonly "freq"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "life"?: (((Input_vfx2dDefinitions["Pair"])) | ((null)));
readonly "r0"?: ((number) | (null));
readonly "r1"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "rise"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "speed"?: (((Input_vfx2dDefinitions["Pair"])) | ((null)));
readonly "spread"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "type": ("puffs") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "curve"?: never; readonly "depth"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "spin"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "area"?: ((number) | (null));
readonly "areaX"?: ((number) | (null));
readonly "areaY"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "both"?: ((boolean) | (null));
readonly "count"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "dir"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "drag"?: ((number) | (null));
readonly "driftX"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "emit"?: ((number) | (null));
readonly "fadeIn"?: ((number) | (null));
readonly "fallScale"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "gravity"?: ((number) | (null));
readonly "ground"?: ((number) | (null));
readonly "heat"?: (((Input_vfx2dDefinitions["Pair"])) | ((null)));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "implodeR"?: ((number) | (null));
readonly "jitter"?: ((number) | (null));
readonly "life"?: (((Input_vfx2dDefinitions["Pair"])) | ((null)));
readonly "mode"?: (((Input_vfx2dDefinitions["ParticleMode"])) | ((null)));
readonly "orbitR"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "shape"?: (((Input_vfx2dDefinitions["ParticleShape"])) | ((null)));
readonly "size"?: (((Input_vfx2dDefinitions["Pair"])) | ((null)));
readonly "spanX"?: ((number) | (null));
readonly "spanY"?: ((number) | (null));
readonly "speed"?: (((Input_vfx2dDefinitions["Pair"])) | ((null)));
readonly "spread"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
readonly "swirl"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "turns"?: ((number) | (null));
readonly "twinkle"?: ((number) | (null));
readonly "type": ("particles") & (string);
readonly "wiggle"?: ((number) | (null));
readonly "wiggleRate"?: ((number) | (null));
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "arms"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dissolve"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "dur"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeStart"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "grow"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "spin"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "branch"?: ((number) | (null));
readonly "count"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "depth"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "fadeStart"?: ((number) | (null));
readonly "flicker"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hold"?: ((number) | (null));
readonly "hue"?: ((number) | (null));
readonly "jag"?: ((number) | (null));
readonly "len"?: ((number) | (null));
readonly "mode"?: (((Input_vfx2dDefinitions["BoltMode"])) | ((null)));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "spread"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "thick"?: ((number) | (null));
readonly "type": ("bolt") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "x0"?: ((number) | (null));
readonly "x1"?: ((number) | (null));
readonly "y"?: ((number) | (null));
readonly "y0"?: ((number) | (null));
readonly "y1"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "life"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "count"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "fadeStart"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "grow"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "len"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "type": ("cracks") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "count"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "fan"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "grow"?: ((number) | (null));
readonly "hgt"?: (((Input_vfx2dDefinitions["Pair"])) | ((null)));
readonly "hidden"?: ((boolean) | (null));
readonly "hold"?: ((number) | (null));
readonly "hue"?: ((number) | (null));
readonly "lean"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "spread"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "type": ("spikes") & (string);
readonly "width"?: (((Input_vfx2dDefinitions["Pair"])) | ((null)));
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hollow"?: ((number) | (null));
readonly "hue"?: ((number) | (null));
readonly "inner"?: ((number) | (null));
readonly "jitter"?: ((number) | (null));
readonly "points"?: ((number) | (null));
readonly "pop"?: ((number) | (null));
readonly "r"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "sharp"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "type": ("star") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "implodeR"?: never; readonly "jag"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "count"?: ((number) | (null));
readonly "curve"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "dir"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "gap"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "len"?: ((number) | (null));
readonly "mode"?: (((Input_vfx2dDefinitions["StreakMode"])) | ((null)));
readonly "r"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "spanY"?: ((number) | (null));
readonly "speed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
readonly "stagger"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "th"?: ((number) | (null));
readonly "turns"?: ((number) | (null));
readonly "type": ("streaks") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "life"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "cell"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "fill"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hex"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "r"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "shimmer"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "th"?: ((number) | (null));
readonly "type": ("bubble") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "dur"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "r"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "type": ("flash") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "fadeStart"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "pop"?: ((boolean) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "rise"?: ((number) | (null));
readonly "scale"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "text"?: ((string) | (null));
readonly "type": ("text") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((Input_vfx2dDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "freq"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hgt"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "ramp"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "rough"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "spin"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
readonly "sway"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "twist"?: ((number) | (null));
readonly "type": ("tornado") & (string);
readonly "w0"?: ((number) | (null));
readonly "w1"?: ((number) | (null));
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "until"?: never; readonly "w"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }));
readonly "Outline": ("none" | "outside" | "outside8" | "inside") & (string);
readonly "Pair": (((number)) | ((ReadonlyArray<(number)> & { readonly length: 2 })));
readonly "ParticleMode": ("burst" | "spray" | "rise" | "fall" | "ambient" | "orbit" | "implode") & (string);
readonly "ParticleShape": ("dot" | "square" | "plus" | "cross" | "star" | "diamond" | "ring" | "bubble" | "streak" | "drop" | "shard" | "leaf" | "petal" | "chevron" | "coin" | "chunk" | "circle" | "heart") & (string);
readonly "Ramp": (((string)) | ((ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))));
readonly "Recipe": ({
readonly "cleanup"?: ((boolean) | (null));
readonly "dither"?: ((number) | (null));
readonly "fps"?: ((number) | (null));
readonly "frames"?: ((number) | (null));
readonly "h"?: ((number) | (null));
readonly "hue"?: ((number) | (null));
readonly "impact"?: ((number) | (null));
readonly "innerLine"?: ((number) | (null));
readonly "layers": (ReadonlyArray<(Input_vfx2dDefinitions["Layer"])> | (number extends ((Input_vfx2dDefinitions["Layer"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "line"?: ((number) | (null));
readonly "loop"?: ((boolean) | (null));
readonly "name"?: ((string) | (null));
readonly "outline"?: (((Input_vfx2dDefinitions["Outline"])) | ((null)));
readonly "outlineColor"?: ((string) | (null));
readonly "palette"?: (((Input_vfx2dDefinitions["Ramp"])) | ((null)));
readonly "pivot"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "ramps"?: (Readonly<Record<string, (Input_vfx2dDefinitions["Ramp"])>>);
readonly "sat"?: ((number) | (null));
readonly "scale"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "supersample"?: ((number) | (null));
readonly "w"?: ((number) | (null));
});
readonly "StreakMode": ("claw" | "speed" | "flow") & (string);
}
export type Input_video = ({
readonly "aiGenerated"?: ((boolean) | (null));
readonly "alpha"?: ((boolean) | (null));
/** Keep the source audio when present; omitted true. */
readonly "audio"?: ((boolean) | (null));
/** Alternate audio paths under this module's assets/ or sources/.
Locale tags are unique ignoring case; ASCII letters, digits, '-' and '_'. */
readonly "audioLocale"?: (Readonly<Record<string, (string)>>);
readonly "chunk"?: ((number) | (null));
readonly "crf"?: ((number) | (null));
readonly "crf720"?: ((number) | (null));
readonly "fps"?: ((number) | (null));
readonly "keyframe"?: ((number) | (null));
/** Frame processing; omitted color, pixel keeps nearest/lossless encoding. */
readonly "kind"?: (((Input_videoDefinitions["FrameKind"])) | ((null)));
readonly "loudness"?: ((number) | (null));
readonly "lowRendition"?: ((boolean) | (null));
/** Seconds retained from the beginning; omitted: 8 for a clip, whole source for a stream. */
readonly "maxSeconds"?: ((number) | (null));
readonly "placeholder"?: ((boolean) | (null));
/** Zero keeps the full resolution; otherwise 64..4096 pixels. */
readonly "posterSize"?: (((0)) | ((number)) | ((null))) & ((number) | (null));
/** Browser codecs beside AV1; omitted HEVC when available, [] disables them.
All rendition controls require stream: true. */
readonly "renditions"?: ((ReadonlyArray<(Input_videoDefinitions["Rendition"])> | (number extends ((Input_videoDefinitions["Rendition"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Width, height in pixels. */
readonly "size"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** Canonical .mp4, .webm or .mov path under the owning module's assets/
or sources/, without declaration-relative or sibling fallback. */
readonly "source": (string);
readonly "stream"?: ((boolean) | (null));
});
export interface Input_videoDefinitions {
readonly "FrameKind": ("color" | "pixel") & (string);
readonly "Rendition": ("hevc" | "h264") & (string);
}
export interface FileInputs {
readonly "animgraph": Input_animgraph;
readonly "behaviour": Input_behaviour;
readonly "dialogue": Input_dialogue;
readonly "district": Input_district;
readonly "effect": Input_effect;
readonly "effect-texture": Input_effect_texture;
readonly "environment": Input_environment;
readonly "field": Input_field;
readonly "flipbook": Input_flipbook;
readonly "font-extension": Input_font_extension;
readonly "grammar": Input_grammar;
readonly "icons": Input_icons;
readonly "import": Input_import;
readonly "lightmap": Input_lightmap;
readonly "look": Input_look;
readonly "materials": Input_materials;
readonly "model": Input_model;
readonly "model-program": Input_model_program;
readonly "motion-action": Input_motion_action;
readonly "navmesh": Input_navmesh;
readonly "pack": Input_pack;
readonly "particles": Input_particles;
readonly "post": Input_post;
readonly "quest": Input_quest;
readonly "rig": Input_rig;
readonly "rig2d": Input_rig2d;
readonly "scatter": Input_scatter;
readonly "sfx": Input_sfx;
readonly "shader": Input_shader;
readonly "sound": Input_sound;
readonly "strings": Input_strings;
readonly "terrain": Input_terrain;
readonly "texture-array": Input_texture_array;
readonly "theme": Input_theme;
readonly "tilemap": Input_tilemap;
readonly "tileset": Input_tileset;
readonly "timeline": Input_timeline;
readonly "ui-material": Input_ui_material;
readonly "vector": Input_vector;
readonly "vfx": Input_vfx;
readonly "vfx2d": Input_vfx2d;
readonly "video": Input_video;
}
