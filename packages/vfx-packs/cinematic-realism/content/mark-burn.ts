import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 30,
  "looping": false,
  "events": [],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 30,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "decal",
        "angleFade": 80,
        "color": [
          1,
          1,
          1,
          1
        ],
        "emissive": [
          0,
          0,
          0
        ],
        "fade": 5,
        "normal": ref("base/vfx/cinematic-realism/textures/scorch-normal"),
        "normalScale": 1,
        "roughness": 0.9599999785423279,
        "size": [
          1.600000023841858,
          0.6000000238418579,
          1.600000023841858
        ],
        "texture": ref("base/vfx/cinematic-realism/textures/scorch"),
        "turn": [
          0,
          360
        ],
        "uv": [
          0,
          0,
          1,
          1
        ],
        "wet": false
      }
    },
    {
      "at": 0,
      "life": 5,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "decal",
        "angleFade": 80,
        "color": [
          0.05999999865889549,
          0.05000000074505806,
          0.04500000178813934,
          1
        ],
        "emissive": [
          3,
          1.7000000476837158,
          0.8999999761581421
        ],
        "fade": 4.5,
        "normalScale": 1,
        "size": [
          0.800000011920929,
          0.6000000238418579,
          0.800000011920929
        ],
        "texture": ref("base/vfx/cinematic-realism/textures/embers"),
        "turn": [
          0,
          360
        ],
        "uv": [
          0,
          0,
          1,
          1
        ],
        "wet": false
      }
    }
  ]
});
