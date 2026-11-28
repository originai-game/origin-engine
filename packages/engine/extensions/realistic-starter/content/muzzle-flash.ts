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
        "asset": ref("base/vfx/realistic-starter/muzzle-flash/flash.emitter")
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/realistic-starter/muzzle-flash/plume.emitter")
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/realistic-starter/muzzle-flash/sparks.emitter")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0,
        -0.05000000074505806
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/realistic-starter/muzzle-flash/smoke.emitter")
      }
    },
    {
      "at": 0,
      "life": 0.05000000074505806,
      "offset": [
        0,
        0,
        -0.07999999821186066
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
          0.6000000238418579,
          1
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
              0.30000001192092896
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
        0,
        -0.10000000149011612
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
          0.30000001192092896
        ],
        "distance": 4,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              6
            ],
            [
              1,
              0
            ]
          ]
        }
      }
    }
  ]
});
