import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 1,
  "looping": true,
  "events": [],
  "params": [
    {
      "default": 10,
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
          3.200000047683716,
          0.5,
          0.4000000059604645,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/beam"),
        "jitter": 0,
        "segments": 1,
        "to": [
          0,
          1,
          0
        ],
        "width": 0.03500000014901161,
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
          0.8999999761581421,
          0.10000000149011612,
          0.07999999821186066,
          0.3499999940395355
        ],
        "effect": ref("base/vfx/cinematic-realism/beam"),
        "jitter": 0,
        "segments": 1,
        "to": [
          0,
          1,
          0
        ],
        "width": 0.14000000059604645,
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
          0.699999988079071,
          0.5,
          0.699999988079071
        ],
        "size": 0.14000000059604645
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
          1,
          0.30000001192092896,
          0.25
        ],
        "distance": 2,
        "intensity": 0.4000000059604645
      }
    }
  ]
});
