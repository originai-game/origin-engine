import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 2.200000047683716,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/muzzle-blast")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.03999999910593033,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/muzzle-fan")
      }
    },
    {
      "at": 0.029999999329447746,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/gunsmoke-heavy")
      }
    },
    {
      "at": 0,
      "life": 0.07000000029802322,
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
          2.5999999046325684,
          1.5,
          0.550000011920929,
          0.800000011920929
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.5
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
      "life": 0.09000000357627869,
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
          0.699999988079071,
          0.3499999940395355
        ],
        "distance": 5,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              1.2000000476837158
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
        "trauma": 0.2800000011920929
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
          0.949999988079071,
          1.0499999523162842
        ],
        "volume": 1,
        "asset": ref("base/vfx/cinematic-realism/audio/shot-heavy")
      }
    }
  ]
});
