import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 1,
  "looping": true,
  "events": [],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 1,
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
          1.2999999523162842,
          0.800000011920929,
          3,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/tracer-body"),
        "lifetime": 0.2199999988079071,
        "orbit": [
          0,
          0
        ],
        "rise": 0,
        "width": [
          0.11999999731779099,
          0
        ]
      }
    },
    {
      "at": 0,
      "life": 1,
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
          1.600000023841858,
          1.2000000476837158,
          3.200000047683716,
          0.8999999761581421
        ],
        "size": 0.18000000715255737
      }
    },
    {
      "at": 0,
      "life": 1,
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
          0.5,
          0.30000001192092896,
          1.399999976158142,
          0.30000001192092896
        ],
        "size": 0.5
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/cinematic-realism/emitters/bolt-flecks")
      }
    },
    {
      "at": 0,
      "life": 1,
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
          0.6499999761581421,
          0.5,
          1
        ],
        "distance": 3.5,
        "intensity": 0.800000011920929
      }
    }
  ]
});
