import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 1.7999999523162842,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0.10000000149011612
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0.10000000149011612,
      "life": 0,
      "offset": [
        0,
        0.20000000298023224,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/chest-rays")
      }
    },
    {
      "at": 0.10000000149011612,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/motes")
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
        "asset": ref("base/vfx/cinematic-realism/emitters/wood-dust")
      }
    },
    {
      "at": 0.10000000149011612,
      "life": 0.6000000238418579,
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
          1.7999999523162842,
          1.2999999523162842,
          0.5,
          0.6000000238418579
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.4000000059604645
            ],
            [
              0.20000000298023224,
              1
            ],
            [
              1,
              0.5
            ]
          ]
        }
      }
    },
    {
      "at": 0.10000000149011612,
      "life": 1.2000000476837158,
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
        "distance": 5,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              0.15000000596046448,
              2.5
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
      "at": 0.10000000149011612,
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
          1,
          1
        ],
        "volume": 1,
        "asset": ref("base/vfx/cinematic-realism/audio/chime")
      }
    }
  ]
});
