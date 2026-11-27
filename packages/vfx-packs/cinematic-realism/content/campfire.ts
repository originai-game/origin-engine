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
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0.07999999821186066,
        0.20000000298023224,
        0.05999999865889549
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
        "seed": 5,
        "asset": ref("base/vfx/cinematic-realism/emitters/flame-small")
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
        "asset": ref("base/vfx/cinematic-realism/emitters/fire-embers")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.699999988079071,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/fire-smoke")
      }
    },
    {
      "at": 0,
      "life": 0,
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/cinematic-realism/emitters/heat-rise")
      }
    },
    {
      "at": 0,
      "life": 2,
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
        "kind": "glow",
        "color": [
          1.600000023841858,
          0.800000011920929,
          0.25,
          0.18000000715255737
        ],
        "size": 0.699999988079071
      }
    },
    {
      "at": 0,
      "life": 2,
      "offset": [
        0,
        0.44999998807907104,
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
          0.2199999988079071
        ],
        "distance": 6,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              1.7999999523162842
            ],
            [
              0.20000000298023224,
              2.299999952316284
            ],
            [
              0.44999998807907104,
              1.600000023841858
            ],
            [
              0.699999988079071,
              2.0999999046325684
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
