# SDL_GameControllerDB

`gamecontrollerdb.txt` is the community gamepad mapping database
(https://github.com/mdqinc/SDL_GameControllerDB), zlib licensed (LICENSE),
taken unchanged as gilrs 0.11.2 ships it. og-input reads it
(`crates/og-input/src/padmap.rs`) to map a desktop host's raw pads to the
standard layout; the web build carries only og's own lines.
