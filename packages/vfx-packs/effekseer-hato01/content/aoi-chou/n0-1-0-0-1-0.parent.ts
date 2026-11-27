import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.0.1 "Node"
export default file.particles({
  "maxParticles": 288,
  "shape": {
    "kind": "point"
  },
  "spread": 24.444,
  "rate": 22.2222,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    8.3333,
    11.6667
  ],
  "speed": [
    0.5236,
    0.6852
  ],
  "size": [
    0.0,
    0.0
  ],
  "subEmitters": [
    {
      "trigger": "life",
      "emitter": {
        "$ogRef": "base/vfx/effekseer-hato01/aoi-chou/n0-1-0-0-1-0.emitter"
      }
    }
  ]
});
