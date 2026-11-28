import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 1.7999999523162842,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0.10000000149011612
    }
  ],
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
        "asset": ref("base/vfx/cinematic-realism/emitters/aether")
      }
    },
    {
      "at": 0,
      "life": 0.6499999761581421,
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
          0.30000001192092896,
          0.8999999761581421,
          1.399999976158142,
          0.5
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.05999999865889549
            ],
            [
              0.20000000298023224,
              0.4000000059604645
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
      "life": 0.6499999761581421,
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
          0.30000001192092896,
          0.699999988079071,
          1
        ],
        "distance": 2,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              0.20000000298023224,
              0.30000001192092896
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
          1.2000000476837158,
          1.2999999523162842
        ],
        "volume": 0.699999988079071,
        "asset": ref("base/vfx/cinematic-realism/audio/swell")
      }
    }
  ]
});
