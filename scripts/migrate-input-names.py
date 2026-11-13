#!/usr/bin/env python3
"""Run from the repository root: migrate input API names, not generated files."""
from pathlib import Path
import re

renames = {
    "actionDown": "isActionPressed",
    "actionPressed": "isActionJustPressed",
    "actionReleased": "isActionJustReleased",
    "setContexts": "setActionMaps",
    "og_input_action_down": "og_input_is_action_pressed",
    "og_input_action_pressed": "og_input_is_action_just_pressed",
    "og_input_action_released": "og_input_is_action_just_released",
    "og_input_set_contexts": "og_input_set_action_maps",
    "og_input_contexts": "og_input_action_maps",
}
paths = [
    "crates/og-api/api/input.ogt",
    "crates/og-runtime/src/host/api/actions.rs",
    "crates/og-host-node/src/host.rs",
    "crates/og-host-web/src/host.rs",
    "packages/engine/src/input.ts",
    "packages/engine/tests/input-actions.test.ts",
]
pattern = re.compile(r"\b(" + "|".join(renames) + r")\b")
for name in paths:
    path = Path(name)
    before = path.read_text()
    text = pattern.sub(lambda match: renames[match[0]], before)
    if name.endswith("input.ogt"):
        text = text.replace("field context ", "field actionMap ")
        text = text.replace("fn contexts {", "fn actionMaps {")
        text = text.replace("param contexts ", "param actionMaps ")
        text = re.sub(r"\bcontexts\b", "action maps", text)
        text = re.sub(r"\bcontext\b", "action map", text)
    if name.endswith("src/input.ts"):
        text = text.replace("  contexts:", "  actionMaps:")
    if before != text:
        path.write_text(text)
        print(path)

# Rust implementation names remain internal; public JSON has no old keys.
path = Path("crates/og-input/src/actions.rs")
before = path.read_text()
text = before.replace('"context"', '"actionMap"').replace('"contexts"', '"actionMaps"')
if before != text:
    path.write_text(text)
    print(path)
