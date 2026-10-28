# pixelorama

The outline of Pixelorama's OutlineInline layer effect
(https://github.com/Orama-Interactive/Pixelorama,
`src/Shaders/Effects/OutlineInline.gdshaderinc` at commit
`4ed451dec0c3304ffb3e9e20dca6648679892633`, itself based on
godotshaders.com's "2D outline/inline"), ported from a Godot fragment
shader to the cook's texels in
`crates/og-cook/src/cook/conform/texture2d/outline.rs`: the line the art
contract (`ArtLine`) draws into a sprite's cells, with its diamond, circle
and square brushes, inside or outside the shape. MIT, the notice in
`LICENSE`. No file of Pixelorama is copied; the port follows the shader
line by line.
