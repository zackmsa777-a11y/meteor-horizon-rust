# SPDX-License-Identifier: GPL-3.0-only
"""Build editable SVG artwork from geometric paths; no image generation."""
from pathlib import Path
from html import escape

ROOT = Path(__file__).resolve().parent
PALETTES = {
    'light': {'ink': '#203449', 'muted': '#60788b', 'horizon': '#467c99'},
    'dark': {'ink': '#eef4fa', 'muted': '#a8bdcf', 'horizon': '#8cbcd2'},
}
AMBER = '#e7aa4c'


def geometry(palette):
    return f'''<g fill="{AMBER}">
    <path d="M87 63 136 14h12L99 63Z"/>
    <path d="m113 66 47-47h10l-47 47Z"/>
    <path d="m133 83 36-36h10l-36 36Z"/>
  </g>
  <path fill="{palette['ink']}" d="m55 89 23-21 28 4 22 22-12 28-28 13-29-18-8-16Z"/>
  <path fill="{AMBER}" d="m64 92 17-16 21 3 17 17-10 20-21 10-21-14-7-12Z"/>
  <path fill="{palette['ink']}" d="m60 94 21-18 7 22-21 14-7-12Z"/>
  <path fill="none" stroke="{palette['horizon']}" stroke-width="5" stroke-linecap="square"
    stroke-linejoin="miter" d="M23 141h45l9 9h34l9-9h49"/>
  <path fill="{palette['horizon']}" d="M43 163h106v3H43Z"/>'''


def document(width, height, title, desc, body):
    return f'''<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}"
  viewBox="0 0 {width} {height}" role="img" aria-labelledby="title desc">
  <!-- SPDX-License-Identifier: GPL-3.0-only -->
  <title id="title">{escape(title)}</title>
  <desc id="desc">{escape(desc)}</desc>
  {body}
</svg>
'''


def build():
    for theme, palette in PALETTES.items():
        suffix = '' if theme == 'light' else '-dark'
        (ROOT / f'logo{suffix}.svg').write_bytes(document(
            192, 192, 'Meteor Horizon',
            'A faceted meteor with three trails above a stepped horizon.',
            geometry(palette)).encode())
        body = f'''<g transform="translate(0 2) scale(.8)">{geometry(palette)}</g>
  <g font-family="Segoe UI, Arial, sans-serif">
    <text x="177" y="88" fill="{palette['ink']}" font-size="53" font-weight="650"
      letter-spacing="-1.6">Meteor Horizon</text>
    <text x="181" y="120" fill="{palette['muted']}" font-size="16">Native utility mod for Horizon</text>
  </g>'''
        (ROOT / f'wordmark{suffix}.svg').write_bytes(document(
            570, 160, 'Meteor Horizon', 'Meteor Horizon wordmark and meteor emblem.', body).encode())
    badges = ROOT / 'badges'
    badges.mkdir(exist_ok=True)
    for filename, label, width, accent in [
        ('rust.svg', 'Rust', 66, AMBER),
        ('minecraft.svg', 'Minecraft 26.3', 140, '#8cbcd2'),
        ('native.svg', 'Native .hnmod', 132, AMBER),
    ]:
        body = f'''<rect width="{width}" height="26" rx="5" fill="#203449"/>
  <rect x="9" y="10" width="6" height="6" rx="1" fill="{accent}"/>
  <text x="23" y="17" fill="#eef4fa" font-family="Segoe UI, Arial, sans-serif"
    font-size="11.5" font-weight="600">{escape(label)}</text>'''
        (badges / filename).write_bytes(document(width, 26, label, label, body).encode())


if __name__ == '__main__':
    build()
