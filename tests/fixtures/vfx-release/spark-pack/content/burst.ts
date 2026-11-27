import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 0.5,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0.019999999552965164
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 0.15000000596046448,
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
          2,
          1.399999976158142,
          0.6000000238418579,
          1
        ],
        "size": 1
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
        "asset": ref("base/vfx/spark-pack/sparks")
      }
    }
  ]
});
