//! Terrain stamps read from rendered frames: a mesh stamped into a
//! terrain is shaded by the terrain's own layers by its own normal (rock
//! on a cliff's face, grass on the valley before it), and within `blend`
//! metres of the ground it meets it takes the ground's look, so the seam
//! where the face meets the valley disappears.
