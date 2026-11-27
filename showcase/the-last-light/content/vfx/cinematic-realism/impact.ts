import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 1.2000000476837158,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0
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
        "asset": ref("base/vfx/cinematic-realism/emitters/impact-sparks")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.05000000074505806,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/impact-dust")
      }
    },
    {
      "at": 0,
      "life": 0,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/grit")
      }
    },
    {
      "at": 0,
      "life": 0.1599999964237213,
      "offset": [
        0,
        0.11999999731779099,
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
          0.8500000238418579,
          0.25,
          0.6499999761581421
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.18000000715255737
            ],
            [
              0.20000000298023224,
              0.36000001430511475
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
      "life": 0.11999999731779099,
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
          0.6200000047683716,
          0.2800000011920929
        ],
        "distance": 1.7999999523162842,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.18000000715255737
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
        "trauma": 0.18000000715255737
      }
    }
  ]
});
