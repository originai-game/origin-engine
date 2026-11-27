import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "bursts": [
    {
      "count": 26,
      "cycles": 2,
      "interval": 0.02500000037252903,
      "time": 0
    }
  ],
  "duration": 0.04170000180602074,
  "lifetime": [
    1,
    1
  ],
  "looping": false,
  "maxParticles": 65,
  "rate": 0,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.0010000000474974513,
      0.1363999992609024,
      0.5457000136375427
    ]
  },
  "size": [
    0,
    0
  ],
  "speed": [
    4.534999847412109,
    14.418000221252441
  ],
  "speedOverLife": [
    [
      0,
      2.1308000087738037
    ],
    [
      0.25,
      1.3813999891281128
    ],
    [
      0.5,
      0.8472999930381775
    ],
    [
      0.75,
      0.5422999858856201
    ],
    [
      1,
      0.4657000005245209
    ]
  ],
  "spread": 34.3302001953125,
  "subEmitters": [
    {
      "emitter": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-2-0-0.emitter"),
      "inheritVelocity": 0,
      "trigger": "life"
    }
  ]
});
