# Representative UI references

`../tests/reference-frames.ts` names the retained PNGs by kit. Keep at most
five per kit: its main desktop and phone screen/HUD, then a visual state such
as an aimed card, a gacha reveal, a drag rejection, a checkpoint or a composition
underline. A new frame replaces a less useful one rather than adding another
locale, scale, hover or style matrix.

The materials kit has only two existing surface references; flight has none.
Do not manufacture references to fill the budget. The five existing style
sheet PNGs live separately in `../../styles/references`: fantasy and scifi
have desktop/phone sheets, storybook a phone sheet. Their bytes and comparison
tolerances are unchanged.

`selectReferenceFrames` removes unselected screenshot commands and their
comparison inputs. It preserves other commands, timestamps and order. Tests
still check every existing interaction and dump assertion, including locale
strings and showcase entrance/exit settling. No missing-file check is waived:
a retained reference missing from disk still fails.

Run `scripts/check.sh kits` (`ui` alias). To run the frame steps separately,
use `scripts/check.sh ui-styles ui-kit`. Browser references use `kit-web`;
locale behavior remains in the native suite instead of a duplicate browser
image matrix.

Audit the inventory and retained bytes against a base commit:

```sh
bun templates/ui-kit/scripts/audit-frames.ts 5387f79db
```

Counts assign a mixed showcase sheet once to its primary genre family. The
audit also prints style packs with no existing reference PNGs as zero.
