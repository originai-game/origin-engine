# pixel-scenario

The Pixel VFX Forge library by Scenario
(https://www.scenario.com/explorations/pixel-vfx-forge/): 42 pixel-art
effects (combat, magic, elements, feedback, ambient) as checked `file.vfx2d`
declarations containing the Forge's recipe data. `og vfx add
<this directory>` installs them into `content/vfx/pixel-scenario/`, where each cooks
into a native flipbook sheet (`base/vfx/pixel-scenario/<name>`, one clip named `<name>`
with an `impact` event) and its texture `base/vfx/pixel-scenario/<name>#sheet`;
`og api show pixel-vfx` has the format.

The recipes are the Forge's library (`forms.js`'s `LIBRARY`, a prompt and
a seed each) as its `interpret` writes them, from Scenario's
`scenario-pixel-vfx` skill
(https://iieqkinwjorxrzjp.public.blob.vercel-storage.com/explorations/f5284ec0200005bb234c2b9ddafa82cd3ab54b3321dda785ce8cccd17d23cd2c/scenario-pixel-vfx-skill.zip,
the Forge's page links its repository https://github.com/scenario-labs/skills),
used with Scenario's permission (origin-engine's
`third_party/scenario-pixel-vfx/`). The same JSON pastes into the Forge's
web tool and renders the same frames there.
