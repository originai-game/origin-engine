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
        "kind": "beam",
        "color": [
          0.6000000238418579,
          1.2999999523162842,
          3,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/tracer-body"),
        "jitter": 0,
        "segments": 1,
        "to": [
          0,
          -0.8999999761581421,
          0
        ],
        "width": 0.09000000357627869
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
          0.800000011920929,
          1.600000023841858,
          3.200000047683716,
          0.8999999761581421
        ],
        "size": 0.1599999964237213
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
          0.20000000298023224,
          0.5,
          1.399999976158142,
          0.3499999940395355
        ],
        "size": 0.4000000059604645
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
          0.4000000059604645,
          0.699999988079071,
          1
        ],
        "distance": 3,
        "intensity": 0.699999988079071
      }
    }
  ]
});
