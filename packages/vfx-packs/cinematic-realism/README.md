# Cinematic realism

Nine visual mechanisms for `cinematic-realism` and its `muted` variant.
Install with `og vfx add cinematic-realism`; spawn
`vfx/cinematic-realism/<name>.ogt`. This downloadable pack adds nothing to
a game that does not install it. There are no game rules, sounds, damage,
rewards or automatic UI actions. Explosion, impact and muzzle include brief camera
impulses, decayed by the game's `og.Camera.traumaDecay` setting.

| Effect | Placement and scale |
|---|---|
| explosion | Origin on the ground; a 3 m fireball with flash, debris and a dust skirt, then a rising smoke column (reaches about 9 m) |
| impact | Origin on the surface, +y its normal; short sparks and dust |
| muzzle | Origin at the barrel, +y forward; a flame tongue with side petals, then gun smoke |
| pickup | Origin at the object; amber motes and a brief warm pulse |
| magic | Origin at the source; cool turbulent wisps and a lit core |
| dust | Origin on the ground; rolling dust, not an opaque smoke wall |
| splash | Origin on the water plane; a water column, ballistic spray, mist and a ripple |
| exhaust | Loop at the nozzle, +y downstream; a hot jet and a cooler wake; move/rotate the instance with the craft |
| celebration | Origin at the UI anchor projected into the scene; falling satin confetti |

`exhaust` is a looping emitter, not a flight controller. `celebration` is a
world-space visual, not a screen-space UI component. Use the game's own
camera projection and placement. The neutral colours and low light energy
leave exposure and the scene's single accent under the game's control.

Explosion and dust use Thomas Iché / Unity Labs' Houdini simulations,
released CC0 at https://unity.com/blog/engine-platform/free-vfx-image-sequences-flipbooks.
See `third_party/unity-vfx/LICENSE` in the engine source and each file's
provenance in `assets.json`. The effect compositions are MIT, Origin Engine.
