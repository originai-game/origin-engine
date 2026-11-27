import { file } from "@origingame/engine/author";

export default file.look({
  // A narrower, stronger tilt-shift band than the toy's, a vignette and AgX:
  // a tabletop model photographed close.
  // The diorama variant of toy-3d (`variant "diorama" `): only what it changes of
  // the preset, a field it turns off written at its default.
  name: "Miniature dioramas",
  PostEffects: {
    dofTiltShift: 0.07,
    dofMaxBlur: 0.016,
    vignetteIntensity: 0.35,
    ao: false,
  },
  ArtShape: {
    characterFaces: 0,
  },
});
