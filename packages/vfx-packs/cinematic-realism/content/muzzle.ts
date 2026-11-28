import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 1.399999976158142,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/muzzle")
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
        "asset": ref("base/vfx/cinematic-realism/emitters/muzzle-petals")
      }
    },
    {
      "at": 0.029999999329447746,
      "life": 0,
      "offset": [
        0,
        0.05999999865889549,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/gunsmoke")
      }
    },
    {
      "at": 0,
      "life": 0.05999999865889549,
      "offset": [
        0,
        0.07999999821186066,
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
          2.4000000953674316,
          1.399999976158142,
          0.5,
          0.800000011920929
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.2199999988079071
            ],
            [
              1,
              0.10000000149011612
            ]
          ]
        }
      }
    },
    {
      "at": 0,
      "life": 0.07000000029802322,
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
          0.699999988079071,
          0.3499999940395355
        ],
        "distance": 2.200000047683716,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
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
        "kind": "shake",
        "trauma": 0.11999999731779099
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
          0.9599999785423279,
          1.0399999618530273
        ],
        "volume": 1,
        "asset": ref("base/vfx/cinematic-realism/audio/shot")
      }
    }
  ]
});
