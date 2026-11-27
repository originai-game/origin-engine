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
        "kind": "mesh",
        "blend": "additive",
        "color": [
          0.30000001192092896,
          0.699999988079071,
          1.600000023841858,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/barrier-shell"),
        "faceCamera": false,
        "scale": 2.5999999046325684,
        "shape": {
          "kind": "sphere"
        },
        "spin": [
          0,
          12,
          0
        ]
      }
    },
    {
      "at": 0,
      "life": 2,
      "offset": [
        0,
        0.019999999552965164,
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
        "blend": "additive",
        "color": [
          0.4000000059604645,
          0.8999999761581421,
          2,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/beam"),
        "faceCamera": false,
        "scale": 1.2999999523162842,
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                1,
                1,
                0
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
                0
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.9399999976158142,
            "length": 360,
            "segments": 64,
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
          0.44999998807907104,
          0.699999988079071,
          1
        ],
        "distance": 4,
        "intensity": 0.5
      }
    }
  ]
});
