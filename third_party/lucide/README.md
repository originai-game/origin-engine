# Lucide fallback icons

Unmodified SVGs from [Lucide](https://github.com/lucide-icons/lucide), revision `8aadaf6c1c55cc050fe00ece3615c34040237aa3`. The upstream ISC and inherited Feather MIT notices are in `LICENSE`.

Engine names map to upstream icon filenames as follows: `back` → `arrow-left`, `bag` → `shopping-bag`, `bolt` → `zap`, `chevronDown` → `chevron-down`, `chevronLeft` → `chevron-left`, `chevronRight` → `chevron-right`, `close` → `x`, `coin` → `circle-dollar-sign`, `drop` → `droplet`, `fire` → `flame`, `gear` → `settings`, `home` → `house`, `music` → `eighth-notes`, `person` → `user`, `potion` → `flask-conical`, `tree` → `tree-pine`. Other names keep the upstream filename.

`crates/og-ui/src/default_icons.rs` owns the last-resort lookup and exposes `og_ui::DEFAULT_ICON_NAMES`. Game icon layers always win. No SVG is parsed and no geometry is allocated until that fallback icon is drawn. There is no atlas or texture allocation.
