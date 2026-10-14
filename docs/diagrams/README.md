# Architecture diagram assets

[SVG](architecture-overview.svg) is the scalable documentation diagram. [PNG](architecture-overview.png) is the preview. [Mermaid](architecture-overview.mmd) is an editable architecture source. The [command sequence](command-sequence.mmd) covers acceptance, rejection, and lost acknowledgements.

All labels and accessibility descriptions are English. The maintained diagram excludes optional gateways and remote MCP mentioned in an early concept illustration. Mermaid and SVG express the same architecture with separate layout sources; update both when changing structure.

Regenerate the SVG from the repository root:

```bash
python3 scripts/render_architecture.py
```

PNG generation additionally requires `cairosvg`. Fonts are referenced by installed system family names; no font files are distributed in this repository.
