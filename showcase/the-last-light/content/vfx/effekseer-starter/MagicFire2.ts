import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 1.909999966621399,
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
      "at": 0.0017000000225380063,
      "life": 0,
      "offset": [
        0,
        0.5,
        0
      ],
      "rotate": [
        0,
        0,
        -90
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
        "asset": ref("base/vfx/effekseer-starter/MagicFire2/n0.emitter")
      }
    }
  ]
});
