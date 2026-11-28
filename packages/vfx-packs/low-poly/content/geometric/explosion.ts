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
        "asset": ref("base/vfx/low-poly/geometric/emitters/fragments")
      }
    },
    {
      "at": 0,
      "life": 0.4000000059604645,
      "offset": [
        0,
        0.03999999910593033,
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
        "blend": "alpha",
        "color": [
          0.8999999761581421,
          0.2199999988079071,
          0.11999999731779099,
          1
        ],
        "effect": ref("base/vfx/low-poly/geometric/flat"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.15000000596046448
            ],
            [
              0.4000000059604645,
              1.2999999523162842
            ],
            [
              1,
              1.600000023841858
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                1,
                1,
                1
              ],
              [
                1,
                1,
                1,
                1
              ],
              [
                1,
                1,
                1,
                1
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.8399999737739563,
            "length": 360,
            "segments": 6,
            "start": 0
          }
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
        "trauma": 0.25
      }
    }
  ]
});
