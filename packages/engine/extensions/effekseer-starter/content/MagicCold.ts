import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 2.1500000953674316,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0.833299994468689
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 0,
      "offset": [
        3,
        8,
        0
      ],
      "rotate": [
        0,
        0,
        159.4438934326172
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
        "asset": ref("base/vfx/effekseer-starter/MagicCold/n0.emitter")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        3,
        10,
        0
      ],
      "rotate": [
        0,
        0,
        163.3007049560547
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
        "asset": ref("base/vfx/effekseer-starter/MagicCold/n1.emitter")
      }
    },
    {
      "at": 0.833299994468689,
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
        "asset": ref("base/vfx/effekseer-starter/MagicCold/n4.emitter")
      }
    },
    {
      "at": 0.833299994468689,
      "life": 0.30000001192092896,
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
        "kind": "light",
        "color": [
          0.5325999855995178,
          1,
          1
        ],
        "distance": 12,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
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
      "at": 0.833299994468689,
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
        "trauma": 0.4000000059604645
      }
    }
  ]
});
