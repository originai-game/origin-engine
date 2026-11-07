# pixel-scenario

The Pixel VFX Forge library by Scenario
(https://www.scenario.com/explorations/pixel-vfx-forge/): 42 pixel-art
effects (combat, magic, elements, feedback, ambient) as `type vfx2d`
files, each naming its recipe, the Forge's JSON unchanged. `og vfx add
<this directory>` copies them into `assets/vfx/pixel-scenario/`, where each cooks
into a flipbook sheet (`vfx/pixel-scenario/<name>.ogt`, one clip named `<name>`
with an `impact` event) and its texture `vfx/pixel-scenario/<name>.ogt#sheet`;
`og api show pixel-vfx` has the format.

The recipes are the Forge's library (`forms.js`'s `LIBRARY`, a prompt and
a seed each) as its `interpret` writes them, from Scenario's
`scenario-pixel-vfx` skill
(https://iieqkinwjorxrzjp.public.blob.vercel-storage.com/explorations/f5284ec0200005bb234c2b9ddafa82cd3ab54b3321dda785ce8cccd17d23cd2c/scenario-pixel-vfx-skill.zip,
the Forge's page links its repository https://github.com/scenario-labs/skills),
used with Scenario's permission (origin-engine's
`third_party/scenario-pixel-vfx/`). The same JSON pastes into the Forge's
web tool and renders the same frames there.
