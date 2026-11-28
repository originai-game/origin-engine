import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 2,
  "looping": true,
  "events": [],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 2,
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
        "kind": "beam",
        "color": [
          1.2000000476837158,
          0.949999988079071,
          0.5,
          0.5
        ],
        "effect": ref("base/vfx/cinematic-realism/beam-soft"),
        "jitter": 0,
        "segments": 1,
        "to": [
          0,
          6,
          0
        ],
        "width": 0.5
      }
    },
    {
      "at": 0,
      "life": 2,
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
        "kind": "beam",
        "color": [
          2,
          1.600000023841858,
          0.8999999761581421,
          0.6000000238418579
        ],
        "effect": ref("base/vfx/cinematic-realism/beam-soft"),
        "jitter": 0,
        "segments": 1,
        "to": [
          0,
          9,
          0
        ],
        "width": 0.14000000059604645
      }
    },
    {
      "at": 0,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/beacon-motes")
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
          1.600000023841858,
          1.2000000476837158,
          0.5,
          0.4000000059604645
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.699999988079071
            ],
            [
              0.5,
              0.8500000238418579
            ],
            [
              1,
              0.699999988079071
            ]
          ]
        }
      }
    },
    {
      "at": 0,
      "life": 2,
      "offset": [
        0,
        0.5,
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
          0.800000011920929,
          0.44999998807907104
        ],
        "distance": 4,
        "intensity": 0.800000011920929
      }
    }
  ]
});
