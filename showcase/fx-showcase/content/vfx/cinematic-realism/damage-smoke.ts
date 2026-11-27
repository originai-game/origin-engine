import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 2,
  "looping": true,
  "events": [],
  "params": [
    {
      "default": 0,
      "name": "fire"
    }
  ],
  "elements": [
    {
      "at": 0,
      "life": 0,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/damage-smoke")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.25,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/flame-small")
      },
      "when": "fire"
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/cinematic-realism/emitters/spark-spits")
      },
      "when": "fire"
    },
    {
      "at": 0,
      "life": 2,
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
          0.550000011920929,
          0.20000000298023224
        ],
        "distance": 4,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              1
            ],
            [
              0.30000001192092896,
              1.399999976158142
            ],
            [
              0.6000000238418579,
              0.8999999761581421
            ],
            [
              1,
              1
            ]
          ]
        }
      },
      "when": "fire"
    }
  ]
});
