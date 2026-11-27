import { file } from '@origingame/engine/author';

// A weathered ash pole with a crook, a cloth grip and an iron lantern hook.
// Its foot is the origin; the lantern hangs from the hook at [0.13, 1.74, 0].
export default file.model({ kind: 'geometry', desc: { items: [
  { name: 'ash', input: { kind: 'material', desc: { color: [0.2, 0.13, 0.075], roughness: 0.82 } } },
  { name: 'wrap', input: { kind: 'material', desc: { color: [0.46, 0.1, 0.07], roughness: 0.95 } } },
  { name: 'iron', input: { kind: 'material', desc: { color: [0.05, 0.05, 0.055], roughness: 0.5, metalness: 1 } } },
  { name: 'shaft', input: { kind: 'part', desc: { material: 'ash', placements: [{ transform: { position: [0, 0.95, 0] } }],
    geometry: { kind: 'cylinderGeometry', desc: { radiusTop: 0.017, radiusBottom: 0.024, height: 1.9, radialSegments: 9, heightSegments: 6 } } } } },
  { name: 'crook', input: { kind: 'part', desc: { material: 'ash', placements: [{ transform: { position: [0.075, 1.9, 0], quaternion: [0, 0, -0.2474, 0.9689] } }],
    geometry: { kind: 'torusGeometry', desc: { radius: 0.075, tube: 0.016, radialSegments: 8, tubularSegments: 20, arc: 3.64 } } } } },
  { name: 'grip', input: { kind: 'part', desc: { material: 'wrap', placements: [{ transform: { position: [0, 1.32, 0] } }],
    geometry: { kind: 'cylinderGeometry', desc: { radiusTop: 0.025, radiusBottom: 0.027, height: 0.2, radialSegments: 9 } } } } },
  { name: 'hook', input: { kind: 'part', desc: { material: 'iron', placements: [{ transform: { position: [0.06, 1.8, 0], quaternion: [0, 0, 0.7071, 0.7071] } }],
    geometry: { kind: 'cylinderGeometry', desc: { radiusTop: 0.005, radiusBottom: 0.005, height: 0.14, radialSegments: 6 } } } } },
  { name: 'ferrule', input: { kind: 'part', desc: { material: 'iron', placements: [{ transform: { position: [0, 0.04, 0] } }],
    geometry: { kind: 'cylinderGeometry', desc: { radiusTop: 0.026, radiusBottom: 0.02, height: 0.08, radialSegments: 9 } } } } },
] } });
