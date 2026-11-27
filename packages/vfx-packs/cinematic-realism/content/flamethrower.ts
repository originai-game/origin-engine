import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 2,
  "looping": true,
  "events": [],
  "params": [],
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
        "asset": ref("base/vfx/cinematic-realism/emitters/flame-jet")
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/cinematic-realism/emitters/flame-soot")
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/cinematic-realism/emitters/heat-jet")
      }
    },
    {
      "at": 0,
      "life": 2,
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
        "kind": "glow",
        "color": [
          2.4000000953674316,
          1.399999976158142,
          0.5,
          0.6000000238418579
        ],
        "size": 0.30000001192092896
      }
    },
    {
      "at": 0,
      "life": 2,
      "offset": [
        0,
        1.7999999523162842,
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
        "distance": 7,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              2
            ],
            [
              0.30000001192092896,
              2.5999999046325684
            ],
            [
              0.6000000238418579,
              1.7999999523162842
            ],
            [
              1,
              2
            ]
          ]
        }
      }
    }
  ]
});
