import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 0.6000000238418579,
  "looping": false,
  "events": [],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 0,
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/cinematic-realism/emitters/thruster-puff")
      }
    },
    {
      "at": 0,
      "life": 0.07999999821186066,
      "offset": [
        0,
        0.029999999329447746,
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
        "kind": "glow",
        "color": [
          0.8999999761581421,
          0.949999988079071,
          1,
          0.4000000059604645
        ],
        "size": 0.10000000149011612
      }
    }
  ]
});
