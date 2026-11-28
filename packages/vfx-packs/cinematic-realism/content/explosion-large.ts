import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 18,
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
        3,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/fireball-large")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        4,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/smoke-column-large")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.30000001192092896,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/dust-ring-large")
      }
    },
    {
      "at": 0,
      "life": 0,
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/cinematic-realism/emitters/debris-large")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        1,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/sparks")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        3.5,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/heat-large")
      }
    },
    {
      "at": 0,
      "life": 0.25,
      "offset": [
        0,
        2.5,
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
          2.200000047683716,
          1.100000023841858,
          0.4000000059604645,
          0.699999988079071
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              4
            ],
            [
              0.30000001192092896,
              9
            ],
            [
              1,
              6
            ]
          ]
        }
      }
    },
    {
      "at": 0,
      "life": 1,
      "offset": [
        0,
        3,
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
          0.5,
          0.18000000715255737
        ],
        "distance": 25,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              30
            ],
            [
              0.15000000596046448,
              18
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
        "trauma": 0.800000011920929
      }
    },
    {
      "at": 0.03999999910593033,
      "life": 17.959999084472656,
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
          18,
          3,
          18
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
      },
      "when": "mark"
    },
    {
      "at": 0.11999999731779099,
      "life": 8,
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
        "fade": 7,
        "normalScale": 1,
        "size": [
          8,
          3,
          8
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
          0.949999988079071,
          1.0499999523162842
        ],
        "volume": 1,
        "asset": ref("base/vfx/cinematic-realism/audio/blast-large")
      }
    }
  ]
});
