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
          1.399999976158142,
          1.0499999523162842,
          0.44999998807907104,
          0.5
        ],
        "effect": ref("base/vfx/cinematic-realism/beam-soft"),
        "jitter": 0,
        "segments": 1,
        "to": [
          0,
          1.399999976158142,
          0
        ],
        "width": 0.2199999988079071
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
        "kind": "trail",
        "color": [
          2,
          1.399999976158142,
          0.5,
          1
        ],
        "effect": null,
        "lifetime": 0.5,
        "orbit": [
          0.25,
          0.5
        ],
        "rise": 0.3499999940395355,
        "width": [
          0.019999999552965164,
          0
        ]
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
        180,
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
        "kind": "trail",
        "color": [
          2,
          1.399999976158142,
          0.5,
          1
        ],
        "effect": null,
        "lifetime": 0.5,
        "orbit": [
          0.25,
          0.5
        ],
        "rise": 0.3499999940395355,
        "width": [
          0.019999999552965164,
          0
        ]
      }
    },
    {
      "at": 0,
      "life": 2,
      "offset": [
        0,
        0.15000000596046448,
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
          1.7999999523162842,
          1.2999999523162842,
          0.5,
          0.44999998807907104
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.3499999940395355
            ],
            [
              0.5,
              0.44999998807907104
            ],
            [
              1,
              0.3499999940395355
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
        "kind": "light",
        "color": [
          1,
          0.800000011920929,
          0.44999998807907104
        ],
        "distance": 2.5,
        "intensity": 0.4000000059604645
      }
    }
  ]
});
