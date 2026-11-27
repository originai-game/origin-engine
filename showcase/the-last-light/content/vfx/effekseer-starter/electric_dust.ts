import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 3.3998000621795654,
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
        "asset": ref("base/vfx/effekseer-starter/electric_dust/n0-0.emitter")
      }
    },
    {
      "at": 0,
      "life": 0.13330000638961792,
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
          1,
          0.3562000095844269,
          0.04699999839067459,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/electric_dust/n0-0-0.effect"),
        "lifetime": 0.13330000638961792,
        "orbit": [
          0,
          0
        ],
        "rise": 0,
        "width": [
          0.25,
          0.0625
        ]
      }
    }
  ]
});
