import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 4,
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
        "asset": ref("base/vfx/realistic-starter/torch-fire/flames.emitter")
      }
    },
    {
      "at": 0,
      "life": 0,
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/realistic-starter/torch-fire/embers.emitter")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.44999998807907104,
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
        "asset": ref("base/vfx/realistic-starter/torch-fire/smoke.emitter")
      }
    },
    {
      "at": 0,
      "life": 4,
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
          0.6000000238418579,
          0.25999999046325684
        ],
        "distance": 3,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.800000011920929
            ],
            [
              0.15000000596046448,
              0.6200000047683716
            ],
            [
              0.30000001192092896,
              0.9200000166893005
            ],
            [
              0.44999998807907104,
              0.6600000262260437
            ],
            [
              0.6000000238418579,
              0.8399999737739563
            ],
            [
              0.75,
              0.5799999833106995
            ],
            [
              0.8999999761581421,
              0.7599999904632568
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
