import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -240,
    240
  ],
  "bursts": [
    {
      "count": 26,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "drag": 2,
  "duration": 0.10000000149011612,
  "gravity": [
    0,
    -220,
    0
  ],
  "looping": false,
  "maxParticles": 40,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "sphere",
    "radius": 8,
    "surface": false
  },
  "size": [
    10,
    16
  ],
  "speed": [
    220,
    520
  ],
  "spread": 180,
  "texture": ref("@styles/shared/fx/shard.png")
});
