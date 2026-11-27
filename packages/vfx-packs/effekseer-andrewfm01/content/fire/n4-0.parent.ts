import { file } from '@origingame/engine/author';
// Effekseer node 4 "Debris"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "point"
  },
  "spread": 11.4218,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 5,
      "interval": 0.0167
    }
  ],
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.75,
    0.75
  ],
  "speed": [
    21.4243,
    33.2716
  ],
  "size": [
    0.0,
    0.0
  ],
  "subEmitters": [
    {
      "trigger": "life",
      "emitter": {
        "$ogRef": "base/vfx/effekseer-andrewfm01/fire/n4-0.emitter"
      }
    }
  ],
  "gravity": [
    0.0,
    -72.0,
    0.0
  ]
});
