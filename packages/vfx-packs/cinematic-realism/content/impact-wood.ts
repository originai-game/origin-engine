import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 10,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0
    }
  ],
  "params": [
    {
      "default": 1,
      "name": "mark"
    }
  ],
  "elements": [
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.029999999329447746,
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/cinematic-realism/emitters/splinters")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.03999999910593033,
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/cinematic-realism/emitters/wood-dust")
      }
    },
    {
      "at": 0,
      "life": 10,
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
        "angleFade": 75,
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
        "fade": 3,
        "normal": ref("base/vfx/cinematic-realism/textures/gouge-normal"),
        "normalScale": 2,
        "roughness": 0.8500000238418579,
        "size": [
          0.30000001192092896,
          0.20000000298023224,
          0.30000001192092896
        ],
        "texture": ref("base/vfx/cinematic-realism/textures/gouge"),
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
      },
      "when": "mark"
    },
    {
      "at": 0,
      "life": 0,
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
        "kind": "sound",
        "pitch": [
          1.100000023841858,
          1.399999976158142
        ],
        "volume": 0.6000000238418579,
        "asset": ref("base/vfx/cinematic-realism/audio/crack")
      }
    }
  ]
});
