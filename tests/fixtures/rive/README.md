# Rive import

Rive files converted at cook time into og-ui artboards (`docs/ui.md`
§11.2). Asset-only: no game, no reference frame.

`assets/` holds seven files from rive-runtime's unit tests
(`tests/unit_tests/assets`, revision `4f864426617a`, MIT, `LICENSE`), all
format 7.0:

| File | What it exercises |
|---|---|
| `shapetest.riv` | a shape of four cubic vertices on an artboard background; a paint with no colour of its own |
| `two_artboards.riv` (+ `.riv.ogt`) | two artboards; the import settings choose one and set its role, name, fit and align |
| `nested_artboard_opacity.riv` | a nested artboard drawn at its container's opacity |
| `cubic_value_test.riv` | a linear animation over a work area, keyed by cubic ease and value interpolators, played by a one-state machine |
| `custom_image_name.riv` | an embedded 15×15 PNG image, cooked to its own texture |
| `hosted_image_file.riv` | an image hosted on Rive's CDN: a cook error naming the asset |
| `light_switch.riv` | a state machine with a bool input: phase 2, refused with the feature named |

Tests that read it: og-cook `riv::tests` (`cargo test -p og-cook --lib
riv::`), og-cook `tests/rive.rs` (the cook with settings), og-tests-cooked
`ui_rive_draws_its_artboards_where_the_file_puts_them` (the `ui-rive`
scene on the GPU).
