import { file } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A glossy clear bubble: brighter at its rim.
export default file.effect({
  "blend": "alpha",
  "doubleSided": false,
  "fresnel": {
    "power": 2.5,
    "strength": 1.6
  }
});
