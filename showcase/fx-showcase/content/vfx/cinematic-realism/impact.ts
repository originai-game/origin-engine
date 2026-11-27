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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/cinematic-realism/emitters/impact-sparks")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.05000000074505806,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/impact-dust")
      }
    },
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
        "asset": ref("base/vfx/cinematic-realism/emitters/grit")
      }
    },
    {
      "at": 0,
      "life": 0.1599999964237213,
      "offset": [
        0,
        0.11999999731779099,
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
        "kind": "glow",
        "color": [
          1.600000023841858,
          0.8500000238418579,
          0.25,
          0.6499999761581421
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.18000000715255737
            ],
            [
              0.20000000298023224,
              0.36000001430511475
            ],
            [
              1,
              0
            ]
          ]
        }
      }
    },
    {
      "at": 0,
      "life": 0.11999999731779099,
      "offset": [
        0,
        0.4000000059604645,
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
        "kind": "light",
        "color": [
          1,
          0.6200000047683716,
          0.2800000011920929
        ],
        "distance": 1.7999999523162842,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.18000000715255737
            ],
            [
              1,
              0
            ]
          ]
        }
      }
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
        "kind": "shake",
        "trauma": 0.18000000715255737
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
        "normal": ref("base/vfx/cinematic-realism/textures/dent-normal"),
        "normalScale": 1.5,
        "roughness": 0.5,
        "size": [
          0.14000000059604645,
          0.10000000149011612,
          0.14000000059604645
        ],
        "texture": ref("base/vfx/cinematic-realism/textures/dent"),
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
          1,
          1.25
        ],
        "volume": 0.8999999761581421,
        "asset": ref("base/vfx/cinematic-realism/audio/tick")
      }
    }
  ]
});
