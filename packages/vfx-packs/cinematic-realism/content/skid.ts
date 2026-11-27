import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 20,
  "looping": false,
  "events": [],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 20,
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
        "kind": "decal",
        "angleFade": 70,
        "color": [
          1,
          1,
          1,
          1
        ],
        "emissive": [
          0,
          0,
          0
        ],
        "fade": 6,
        "normalScale": 1,
        "roughness": 0.75,
        "size": [
          1.399999976158142,
          0.30000001192092896,
          0.23999999463558197
        ],
        "texture": ref("base/vfx/cinematic-realism/textures/skid"),
        "turn": [
          0,
          0
        ],
        "uv": [
          0,
          0,
          1,
          1
        ],
        "wet": false
      }
    }
  ]
});
