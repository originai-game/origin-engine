import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 2,
  "looping": false,
  "events": [
    {
      "name": "healed",
      "time": 1.2000000476837158
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 2,
      "offset": [
        0,
        0.800000011920929,
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
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/fx/pillar"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.6000000238418579
            ],
            [
              0.20000000298023224,
              1.600000023841858
            ],
            [
              1,
              1.7000000476837158
            ]
          ]
        },
        "shape": {
          "kind": "cylinder"
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
      "life": 1.600000023841858,
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
        "kind": "trail",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/fx/streak"),
        "lifetime": 0.44999998807907104,
        "orbit": [
          0.75,
          1.2000000476837158
        ],
        "rise": 1,
        "width": [
          0.11999999731779099,
          0
        ]
      }
    },
    {
      "at": 0,
      "life": 1.600000023841858,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        180,
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
        "kind": "trail",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/fx/streak"),
        "lifetime": 0.44999998807907104,
        "orbit": [
          0.75,
          1.2000000476837158
        ],
        "rise": 1,
        "width": [
          0.11999999731779099,
          0
        ]
      }
    },
    {
      "at": 0,
      "life": 1.600000023841858,
      "offset": [
        0,
        0.20000000298023224,
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
        "asset": ref("base/fx/motes")
      }
    },
    {
      "at": 0,
      "life": 2,
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
        "kind": "light",
        "color": [
          0.30000001192092896,
          1,
          0.44999998807907104
        ],
        "distance": 6,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              0.20000000298023224,
              18
            ],
            [
              0.800000011920929,
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
      "at": 0.10000000149011612,
      "life": 1.899999976158142,
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
        "fade": 0.5,
        "normalScale": 1,
        "size": [
          2.200000047683716,
          0.5,
          2.200000047683716
        ],
        "texture": ref("base/textures/atlas"),
        "turn": [
          0,
          0
        ],
        "uv": [
          0.5,
          0,
          1,
          1
        ],
        "wet": false
      }
    },
    {
      "at": 1.2000000476837158,
      "life": 0.25,
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
        "kind": "glow",
        "color": [
          0.4000000059604645,
          2,
          0.800000011920929,
          1
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.800000011920929
            ],
            [
              1,
              1.7999999523162842
            ]
          ]
        }
      }
    }
  ]
});
