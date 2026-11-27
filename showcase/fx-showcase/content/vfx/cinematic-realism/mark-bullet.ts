import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 30,
  "looping": false,
  "events": [],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 30,
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
        "angleFade": 75,
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
        "fade": 5,
        "normal": ref("base/vfx/cinematic-realism/textures/chip-normal"),
        "normalScale": 1.399999976158142,
        "roughness": 0.8999999761581421,
        "size": [
          0.3400000035762787,
          0.20000000298023224,
          0.3400000035762787
        ],
        "texture": ref("base/vfx/cinematic-realism/textures/chip"),
        "turn": [
          0,
          360
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
