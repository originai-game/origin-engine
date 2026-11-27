import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 1.600000023841858,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0.05000000074505806
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 0.20000000298023224,
      "offset": [
        0,
        0.3499999940395355,
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
          2.4000000953674316,
          1.399999976158142,
          0.6000000238418579,
          1
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.6000000238418579
            ],
            [
              1,
              1.399999976158142
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
        "asset": ref("base/fx/sparks")
      }
    },
    {
      "at": 0.019999999552965164,
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
        "asset": ref("base/fx/embers")
      }
    },
    {
      "at": 0.05000000074505806,
      "life": 0,
      "offset": [
        0,
        0.10000000149011612,
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
        "asset": ref("base/fx/smoke")
      }
    },
    {
      "at": 0,
      "life": 0.44999998807907104,
      "offset": [
        0,
        0.05999999865889549,
        0
      ],
      "rotate": [
        -90,
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
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/fx/shock"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.4000000059604645
            ],
            [
              1,
              3
            ]
          ]
        },
        "shape": {
          "kind": "ring"
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0,
      "life": 0.5,
      "offset": [
        0,
        0.6000000238418579,
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
          0.6000000238418579,
          0.25
        ],
        "distance": 7,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              25
            ],
            [
              0.30000001192092896,
              8
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
      "at": 0.05000000074505806,
      "life": 1.5499999523162842,
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
        "fade": 0.6000000238418579,
        "normalScale": 1,
        "size": [
          2.4000000953674316,
          0.6000000238418579,
          2.4000000953674316
        ],
        "texture": ref("base/textures/atlas"),
        "turn": [
          0,
          0
        ],
        "uv": [
          0,
          0,
          0.5,
          1
        ],
        "wet": false
      }
    },
    {
      "at": 0.05000000074505806,
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
        "trauma": 0.3499999940395355
      }
    }
  ]
});
