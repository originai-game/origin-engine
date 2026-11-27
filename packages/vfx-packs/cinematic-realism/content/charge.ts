import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 1.5,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/inward-motes")
      }
    },
    {
      "at": 0,
      "life": 1.5,
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
        "kind": "glow",
        "color": [
          0.800000011920929,
          1.5,
          3.200000047683716,
          0.800000011920929
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.20000000298023224
            ],
            [
              0.5,
              0.30000001192092896
            ],
            [
              1,
              0.20000000298023224
            ]
          ]
        }
      }
    },
    {
      "at": 0,
      "life": 1.5,
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
        "kind": "glow",
        "color": [
          0.20000000298023224,
          0.5,
          1.399999976158142,
          0.30000001192092896
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.6000000238418579
            ],
            [
              0.5,
              0.800000011920929
            ],
            [
              1,
              0.6000000238418579
            ]
          ]
        }
      }
    },
    {
      "at": 0,
      "life": 1.5,
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
          0.44999998807907104,
          0.699999988079071,
          1
        ],
        "distance": 4,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.800000011920929
            ],
            [
              0.5,
              1.2999999523162842
            ],
            [
              1,
              0.800000011920929
            ]
          ]
        }
      }
    }
  ]
});
