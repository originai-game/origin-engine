import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "bursts": [
    {
      "count": 40,
      "cycles": 1,
      "interval": 0,
      "time": 0.10000000149011612
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.10000000149011612,
    "die": true,
    "friction": 0.800000011920929
  },
  "color": [
    0.4000000059604645,
    0.36000001430511475,
    0.30000001192092896,
    1
  ],
  "drag": 0.5,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    0.800000011920929,
    1.399999976158142
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 64,
  "rate": 0,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.8999999761581421,
      0.20000000298023224,
      0.8999999761581421
    ]
  },
  "sheet": {
    "blend": false,
    "columns": 3,
    "frames": 3,
    "mode": {
      "kind": "random"
    },
    "rows": 1
  },
  "size": [
    0.012000000104308128,
    0.02500000037252903
  ],
  "speed": [
    0.5,
    2
  ],
  "spread": 25,
  "texture": ref("base/vfx/realistic-starter/textures/dirt")
});
