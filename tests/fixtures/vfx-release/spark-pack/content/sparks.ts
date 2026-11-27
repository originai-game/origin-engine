import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 12,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "duration": 0.10000000149011612,
  "lifetime": [
    0.20000000298023224,
    0.4000000059604645
  ],
  "looping": false,
  "maxParticles": 16,
  "rate": 0,
  "shape": {
    "kind": "sphere",
    "radius": 0.05000000074505806,
    "surface": false
  },
  "size": [
    0.03999999910593033,
    0.05999999865889549
  ],
  "speed": [
    2,
    4
  ],
  "spread": 180,
  "texture": ref("base/vfx/spark-pack/spark")
});
