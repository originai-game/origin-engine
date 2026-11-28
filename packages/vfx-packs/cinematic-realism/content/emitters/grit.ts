import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -400,
    400
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 20,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.15000000596046448,
    "die": false,
    "friction": 0.800000011920929
  },
  "color": [
    0.41999998688697815,
    0.3799999952316284,
    0.3199999928474426,
    1
  ],
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    0.5,
    0.8999999761581421
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 24,
  "rate": 0,
  "render": {
    "kind": "mesh",
    "mesh": "box"
  },
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 50,
    "radius": 0.11999999731779099
  },
  "size": [
    0.006000000052154064,
    0.017999999225139618
  ],
  "speed": [
    1.5,
    3
  ]
});
