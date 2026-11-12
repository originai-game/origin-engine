# Core document conversion references

`core_documents_preserve_screen_pixels` renders the first fixture of each core
screen at tick 30, at 1280 × 720 and 390 × 844. Calibration renders all three
steps. `set_style_name("conversion")` disables art-slot debug path labels on
both versions, because unique document IDs and stable row keys change paths.

The 34 unchanged images were captured from the legacy documents before their
reader was removed. The converted documents produced exactly the same pixels.

Approved exception: `save-load-default-1280.png` (RMSE 0.000573) and
`save-load-default-390.png` (RMSE 0.000607) were re-recorded because CSS resolves
the selected parent's background before the Auto badge computes its tint.
Side-by-side inspection and pixel bounds confirmed that only the badge changed.
