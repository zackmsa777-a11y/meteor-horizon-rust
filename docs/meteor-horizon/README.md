# Meteor Horizon artwork

The emblem combines an angular meteor, three trails and a stepped horizon.
It is a separate identity for the mod; it does not replace the host game's art.

All artwork is editable SVG geometry and text. No image-generation model,
generated raster image, stock illustration or external font file was used.
The project includes AI-assisted code contributions, including SVG source work.

- `logo.svg` / `logo-dark.svg`: transparent emblems for light and dark surfaces.
- `wordmark.svg` / `wordmark-dark.svg`: wider versions with the project name.
- `badges/`: local README badges with no third-party image requests.
- `build_assets.py`: the geometry, palette and text that reproduce every SVG.

Rebuild with Python's standard library:

```sh
python docs/meteor-horizon/build_assets.py
```

Keep the emblem's proportions and clear space. Use the dark variant on dark
surfaces. The artwork sources are GPL-3.0-only.
