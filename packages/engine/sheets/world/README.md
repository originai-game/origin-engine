# In-world components

The field and crowd pages exercise anchors, ground projections, occlusion
and decluttering in a 3D scene. Their final frame review runs on the L4;
the orb's software renderer does not supply an established world baseline.

The HTML-with-CSS conversion keeps the existing ids, parts and semantic
anchors. Nameplate bars contain their fills, placement previews contain
their grid and range, and optional text and shapes use conditional markup.
DamageNumbers and OverheadBars keep their mass rendering paths; SelectionRing,
Telegraph, PathLine and PlacementGhost keep their ground projection.

For the 2D controls, the HUD sheet's `tactics-default` page reviews radial
commands and selected/unselected map layers at desktop and phone widths.
The world documents are also covered by the built-in component fixture
and mounting tests. Those checks do not replace the L4's world frame review.
