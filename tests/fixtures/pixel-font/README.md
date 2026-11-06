# Pixel-font cook

DotGothic16's dots sit off a 16-pixel em grid. The existing og-cook test
`a_pixel_font_cooks_on_its_grid` compares it with Fusion Pixel's true
12-pixel grid and requires the off-grid diagnostic. This is an asset-only
fixture, not a game and not a shipped font.

The Latin-1 subset is moved unchanged from Lamp Dive. Copyright 2020
The DotGothic16 Project Authors
(https://github.com/fontworks-fonts/DotGothic16/); SIL Open Font License
1.1 is retained in `OFL.txt`.

Run `cargo test -p og-cook --lib a_pixel_font_cooks_on_its_grid`.
