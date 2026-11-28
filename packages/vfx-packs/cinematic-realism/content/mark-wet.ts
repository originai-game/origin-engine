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
          0.2800000011920929,
          0.28999999165534973,
          0.30000001192092896,
          1
        ],
        "emissive": [
          0,
          0,
          0
        ],
        "fade": 8,
        "normalScale": 1,
        "roughness": 0.11999999731779099,
        "size": [
          0.8999999761581421,
          0.4000000059604645,
          0.8999999761581421
        ],
        "texture": ref("base/vfx/cinematic-realism/textures/wet"),
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
        "wet": true
      }
    }
  ]
});
