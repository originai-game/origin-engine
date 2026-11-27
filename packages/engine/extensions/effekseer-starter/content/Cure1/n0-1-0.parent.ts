import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "bursts": [
    {
      "count": 5,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "duration": 0.016699999570846558,
  "lifetime": [
    0.5,
    0.5
  ],
  "looping": false,
  "maxParticles": 7,
  "rate": 0,
  "shape": {
    "kind": "point"
  },
  "size": [
    0,
    0
  ],
  "speed": [
    2.828399896621704,
    2.828399896621704
  ],
  "subEmitters": [
    {
      "emitter": ref("base/vfx/effekseer-starter/Cure1/n0-1-0.emitter"),
      "inheritVelocity": 0,
      "trigger": "life"
    }
  ]
});
