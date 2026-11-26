// Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate.
use og_api::wire::{self, ToJson};
use serde_json::{Value, json};

pub const FUNCTIONS: &[&str] = &[
    "noise",
    "noiseGrad",
    "curlNoise",
    "worley",
    "noiseGrid",
    "poissonDisk",
    "bsp",
    "wfc",
    "heightmap",
    "plant",
    "fracture",
];

pub const MODULE_SOURCE: &str = "\n        const invoke = globalThis.__procgen;\n        const shapes = {\"noiseGrid\":{\"value\":\"Float32Array\"},\"bsp\":{\"value\":{\"cells\":\"Uint8Array\"}},\"wfc\":{\"value\":{\"tiles\":\"Uint32Array\",\"rotations\":\"Uint8Array\"}},\"heightmap\":{\"value\":{\"heights\":\"Float32Array\",\"control\":\"Uint8Array\"}},\"plant\":{\"value\":{\"wood\":{\"positions\":\"Float32Array\",\"normals\":\"Float32Array\",\"uvs\":\"Float32Array\",\"colors\":\"Float32Array\",\"indices\":\"Uint32Array\",\"windPivot\":\"Float32Array\",\"windLeaf\":\"Float32Array\"},\"leaves\":{\"positions\":\"Float32Array\",\"normals\":\"Float32Array\",\"uvs\":\"Float32Array\",\"colors\":\"Float32Array\",\"indices\":\"Uint32Array\",\"windPivot\":\"Float32Array\",\"windLeaf\":\"Float32Array\"}}},\"fracture\":{\"value\":{\"pieces\":[{\"positions\":\"Float32Array\",\"normals\":\"Float32Array\",\"uvs\":\"Float32Array\",\"indices\":\"Uint32Array\"}]}}};\n        function typed(value, shape) {\n            if (value == null || shape == null) return value;\n            if (typeof shape === 'string') return new globalThis[shape](value);\n            if (Array.isArray(shape)) return value.map(v => typed(v, shape[0]));\n            for (const key of Object.keys(shape)) value[key] = typed(value[key], shape[key]);\n            return value;\n        }\n        export const procgen = Object.freeze(Object.fromEntries([\"noise\",\"noiseGrad\",\"curlNoise\",\"worley\",\"noiseGrid\",\"poissonDisk\",\"bsp\",\"wfc\",\"heightmap\",\"plant\",\"fracture\"].map(name => [name, (...args) =>\n            typed(JSON.parse(invoke(name, JSON.stringify(args, (_, v) => {\n                if (typeof v === 'number' && !Number.isFinite(v)) throw new Error('non-finite generator argument');\n                return ArrayBuffer.isView(v) ? Array.from(v) : v;\n            }))), shapes[name])\n        ])));\n    ";

pub fn invoke(name: &str, args: &[Value]) -> Result<Value, String> {
    Ok(match name {
        "noise" => {
            let a0 = args.first().unwrap_or(&Value::Null).to_string();
            let a1 = wire::value::<f64>(args.get(1).ok_or("og.procgen.noise.x: required argument")?, "x")?;
            let a2 = wire::value::<f64>(args.get(2).ok_or("og.procgen.noise.y: required argument")?, "y")?;
            let a3 = wire::value::<f64>(args.get(3).ok_or("og.procgen.noise.z: required argument")?, "z")?;
            crate::og_procgen_noise(a0, a1, a2, a3).to_json()
        }
        "noiseGrad" => {
            let a0 = args.first().unwrap_or(&Value::Null).to_string();
            let a1 = wire::value::<f64>(args.get(1).ok_or("og.procgen.noiseGrad.x: required argument")?, "x")?;
            let a2 = wire::value::<f64>(args.get(2).ok_or("og.procgen.noiseGrad.y: required argument")?, "y")?;
            let a3 = wire::value::<f64>(args.get(3).ok_or("og.procgen.noiseGrad.z: required argument")?, "z")?;
            crate::og_procgen_noise_grad(a0, a1, a2, a3).to_json()
        }
        "curlNoise" => {
            let a0 = args.first().unwrap_or(&Value::Null).to_string();
            let a1 = wire::value::<f64>(args.get(1).ok_or("og.procgen.curlNoise.x: required argument")?, "x")?;
            let a2 = wire::value::<f64>(args.get(2).ok_or("og.procgen.curlNoise.y: required argument")?, "y")?;
            let a3 = wire::value::<f64>(args.get(3).ok_or("og.procgen.curlNoise.z: required argument")?, "z")?;
            crate::og_procgen_curl_noise(a0, a1, a2, a3).to_json()
        }
        "worley" => {
            let a0 = wire::value::<u32>(args.first().ok_or("og.procgen.worley.seed: required argument")?, "seed")?;
            let a1 = wire::value::<f64>(args.get(1).ok_or("og.procgen.worley.x: required argument")?, "x")?;
            let a2 = wire::value::<f64>(args.get(2).ok_or("og.procgen.worley.y: required argument")?, "y")?;
            let a3 = wire::value::<f64>(args.get(3).ok_or("og.procgen.worley.z: required argument")?, "z")?;
            let a4 = wire::value::<bool>(args.get(4).ok_or("og.procgen.worley.three_d: required argument")?, "three_d")?;
            crate::og_procgen_worley(a0, a1, a2, a3, a4).to_json()
        }
        "noiseGrid" => {
            let a0 = args.first().map(Value::to_string);
            match crate::og_procgen_noise_grid(a0) {
                Ok(value) => json!({"ok": true, "value": value.to_json()}),
                Err(reason) => json!({"ok": false, "reason": reason}),
            }
        }
        "poissonDisk" => {
            let a0 = args.first().map(Value::to_string);
            match crate::og_procgen_poisson_disk(a0) {
                Ok(value) => json!({"ok": true, "value": value.to_json()}),
                Err(reason) => json!({"ok": false, "reason": reason}),
            }
        }
        "bsp" => {
            let a0 = args.first().map(Value::to_string);
            match crate::og_procgen_bsp(a0) {
                Ok(value) => json!({"ok": true, "value": value.to_json()}),
                Err(reason) => json!({"ok": false, "reason": reason}),
            }
        }
        "wfc" => {
            let a0 = args.first().map(Value::to_string);
            match crate::og_procgen_wfc(a0) {
                Ok(value) => json!({"ok": true, "value": value.to_json()}),
                Err(reason) => json!({"ok": false, "reason": reason}),
            }
        }
        "heightmap" => {
            let a0 = args.first().map(|v| v.to_string().into_bytes());
            match crate::og_procgen_heightmap(a0) {
                Ok(value) => json!({"ok": true, "value": value.to_json()}),
                Err(reason) => json!({"ok": false, "reason": reason}),
            }
        }
        "plant" => {
            let a0 = args.first().map(Value::to_string);
            match crate::og_procgen_plant(a0) {
                Ok(value) => json!({"ok": true, "value": value.to_json()}),
                Err(reason) => json!({"ok": false, "reason": reason}),
            }
        }
        "fracture" => {
            let a0 = args.first().map(|v| v.to_string().into_bytes());
            match crate::og_procgen_fracture(a0) {
                Ok(value) => json!({"ok": true, "value": value.to_json()}),
                Err(reason) => json!({"ok": false, "reason": reason}),
            }
        }
        _ => return Err(format!("unknown pure generator: {name}")),
    })
}
