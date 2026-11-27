import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 1,
  "looping": true,
  "events": [],
  "params": [
    {
      "default": 1,
      "name": "length"
    }
  ],
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
          3,
          1.2999999523162842,
          0.44999998807907104,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/tracer-body"),
        "jitter": 0,
        "segments": 1,
        "to": [
          0,
          -3,
          0
        ],
        "width": 0.029999999329447746,
        "stretch": "length"
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
        "kind": "beam",
        "color": [
          1.2000000476837158,
          0.4000000059604645,
          0.11999999731779099,
          0.3499999940395355
        ],
        "effect": ref("base/vfx/cinematic-realism/tracer-body"),
        "jitter": 0,
        "segments": 1,
        "to": [
          0,
          -1.2000000476837158,
          0
        ],
        "width": 0.11999999731779099,
        "stretch": "length"
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
          3,
          1.600000023841858,
          0.699999988079071,
          0.800000011920929
        ],
        "size": 0.05999999865889549
      }
    }
  ]
});
