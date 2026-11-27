import { file } from '@origingame/engine/author';
// Effekseer node 0.11.0.0 "BeyondGenerator"
export default file.particles({
  "maxParticles": 15,
  "shape": {
    "kind": "point"
  },
  "spread": 26.5651,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 12,
      "interval": 0.0167
    }
  ],
  "duration": 0.2,
  "looping": false,
  "lifetime": [
    2.5,
    2.5
  ],
  "speed": [
    0.9546,
    2.8638
  ],
  "size": [
    0.0,
    0.0
  ],
  "subEmitters": [
    {
      "trigger": "life",
      "emitter": {
        "$ogRef": "base/vfx/effekseer-pierre02/Sword_Moonlight/n0-11-0-0-3.emitter"
      }
    }
  ]
});
