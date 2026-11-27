import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 0.983299970626831,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0.016699999570846558
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 0,
      "offset": [
        -1,
        0.800000011920929,
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
        "asset": ref("base/vfx/effekseer-starter/Spear2/n0.emitter")
      }
    },
    {
      "at": 0.016699999570846558,
      "life": 0,
      "offset": [
        0,
        1,
        0
      ],
      "rotate": [
        0,
        0,
        -101.30989837646484
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
        "asset": ref("base/vfx/effekseer-starter/Spear2/n1.emitter")
      }
    }
  ]
});
