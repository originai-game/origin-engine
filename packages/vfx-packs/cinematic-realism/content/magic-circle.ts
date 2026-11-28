import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 4,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0.4000000059604645
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 4,
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
        "angleFade": 70,
        "color": [
          0.11999999731779099,
          0.25,
          0.5,
          0.8999999761581421
        ],
        "emissive": [
          0.699999988079071,
          1.7000000476837158,
          3.4000000953674316
        ],
        "fade": 1,
        "normalScale": 1,
        "size": [
          2.4000000953674316,
          0.5,
          2.4000000953674316
        ],
        "texture": ref("base/vfx/cinematic-realism/textures/sigil"),
        "turn": [
          0,
          0
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
      "life": 3,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/circle-motes")
      }
    },
    {
      "at": 0,
      "life": 4,
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
        "kind": "glow",
        "color": [
          0.25,
          0.6000000238418579,
          1.399999976158142,
          0.20000000298023224
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.5
            ],
            [
              0.15000000596046448,
              2.200000047683716
            ],
            [
              0.800000011920929,
              2.200000047683716
            ],
            [
              1,
              1
            ]
          ]
        }
      }
    },
    {
      "at": 0,
      "life": 4,
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
          0.44999998807907104,
          0.699999988079071,
          1
        ],
        "distance": 5,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              0.15000000596046448,
              1.600000023841858
            ],
            [
              0.800000011920929,
              1.600000023841858
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
        "kind": "sound",
        "pitch": [
          0.8999999761581421,
          1
        ],
        "volume": 1,
        "asset": ref("base/vfx/cinematic-realism/audio/swell")
      }
    }
  ]
});
