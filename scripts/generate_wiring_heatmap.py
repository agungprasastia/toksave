#!/usr/bin/env python3
"""Generate the toksave Wiring Matrix heatmap as SVG (light + dark variants).

Usage:
    python3 scripts/generate_wiring_heatmap.py

Outputs:
    assets/wiring-matrix-dark.svg
    assets/wiring-matrix-light.svg

Edit MATRIX below when the wiring changes, then re-run this script and
commit the regenerated SVGs.
"""
from __future__ import annotations

import os

AGENTS = [
    "Claude",
    "OpenCode",
    "Codex",
    "Antigravity",
    "Copilot",
    "Droid",
    "Devin",
    "Warp",
    "Cursor",
]

TOOLS = ["RTK", "Caveman", "Ponytail", "CodeGraph", "Context-Mode", "Principles"]

# category -> (fill color, label used in legend)
CATEGORIES = {
    "hook": ("#3B82F6", "Hook — CLI interceptor"),
    "plugin": ("#A855F7", "Plugin — agent extension"),
    "skill": ("#F59E0B", "Skill — markdown skill file"),
    "mcp": ("#14B8A6", "MCP — Model Context Protocol server"),
    "instr": ("#64748B", "Instr. — instruction-block only"),
}

# tool -> agent -> (category, short_label)
MATRIX: dict[str, dict[str, tuple[str, str]]] = {
    "RTK": {
        "Claude": ("hook", "Hook + Allow"),
        "OpenCode": ("plugin", "Plugin"),
        "Codex": ("hook", "Hook"),
        "Antigravity": ("hook", "Hook + Allow"),
        "Copilot": ("hook", "Hook + Allow"),
        "Droid": ("hook", "Hook"),
        "Devin": ("hook", "Hook"),
        "Warp": ("hook", "Hook"),
        "Cursor": ("hook", "Hook + Allow"),
    },
    "Caveman": {
        "Claude": ("plugin", "Plugin"),
        "OpenCode": ("plugin", "Plugin"),
        "Codex": ("skill", "Skill"),
        "Antigravity": ("skill", "Skill"),
        "Copilot": ("skill", "Skill"),
        "Droid": ("skill", "Skill"),
        "Devin": ("skill", "Skill"),
        "Warp": ("skill", "Skill"),
        "Cursor": ("skill", "Skill"),
    },
    "Ponytail": {
        "Claude": ("plugin", "Plugin"),
        "OpenCode": ("plugin", "Plugin"),
        "Codex": ("plugin", "Plugin"),
        "Antigravity": ("plugin", "Plugin"),
        "Copilot": ("skill", "Skill"),
        "Droid": ("skill", "Skill"),
        "Devin": ("skill", "Skill"),
        "Warp": ("skill", "Skill"),
        "Cursor": ("skill", "Skill"),
    },
    "CodeGraph": {
        "Claude": ("mcp", "MCP + Allow"),
        "OpenCode": ("mcp", "MCP + Auto-index"),
        "Codex": ("mcp", "MCP"),
        "Antigravity": ("mcp", "MCP + Hook"),
        "Copilot": ("mcp", "MCP + Hook"),
        "Droid": ("mcp", "MCP + Hook"),
        "Devin": ("mcp", "MCP"),
        "Warp": ("mcp", "MCP"),
        "Cursor": ("mcp", "MCP"),
    },
    "Context-Mode": {
        "Claude": ("mcp", "MCP + Allow"),
        "OpenCode": ("plugin", "Plugin"),
        "Codex": ("mcp", "MCP + Hook"),
        "Antigravity": ("mcp", "MCP"),
        "Copilot": ("mcp", "MCP + Hook"),
        "Droid": ("mcp", "MCP"),
        "Devin": ("mcp", "MCP"),
        "Warp": ("mcp", "MCP"),
        "Cursor": ("mcp", "MCP"),
    },
    "Principles": {a: ("instr", "Instr.") for a in AGENTS},
}

# ── Layout ──
COL_HEADER_W = 128
ROW_HEADER_H = 46
CELL_W = 116
CELL_H = 54
PAD = 24
LEGEND_H = 46
TITLE_H = 34
SUBTITLE_H = 22
FOOTNOTE_H = 24

GRID_W = COL_HEADER_W + CELL_W * len(AGENTS)
GRID_H = ROW_HEADER_H + CELL_H * len(TOOLS)

CANVAS_W = GRID_W + PAD * 2
CANVAS_H = (
    PAD
    + TITLE_H
    + SUBTITLE_H
    + 12
    + GRID_H
    + 14
    + LEGEND_H
    + FOOTNOTE_H
    + PAD
)


def esc(s: str) -> str:
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def render(theme: str) -> str:
    dark = theme == "dark"
    bg = "#0D1117" if dark else "#FFFFFF"
    grid_line = "#30363D" if dark else "#D0D7DE"
    header_bg = "#161B22" if dark else "#F6F8FA"
    header_text = "#E6EDF3" if dark else "#1F2328"
    title_color = "#F0F6FC" if dark else "#0B0D0F"
    subtitle_color = "#8B949E" if dark else "#57606A"
    cell_text = "#F8FAFC"

    parts: list[str] = []
    parts.append(
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{CANVAS_W}" '
        f'height="{CANVAS_H}" viewBox="0 0 {CANVAS_W} {CANVAS_H}" '
        f'font-family="-apple-system,BlinkMacSystemFont,Segoe UI,Helvetica,Arial,sans-serif">'
    )
    parts.append(f'<rect width="{CANVAS_W}" height="{CANVAS_H}" fill="{bg}"/>')

    # Title + subtitle
    title_y = PAD + 18
    parts.append(
        f'<text x="{PAD}" y="{title_y}" font-size="18" font-weight="700" '
        f'fill="{title_color}">TokSave Wiring Matrix</text>'
    )
    subtitle_y = title_y + 20
    parts.append(
        f'<text x="{PAD}" y="{subtitle_y}" font-size="12" '
        f'fill="{subtitle_color}">Integration mechanism used per tool \u00d7 agent '
        f'(color = mechanism, label = exact wiring)</text>'
    )

    grid_x = PAD
    grid_y = subtitle_y + 20

    # Column headers (agents)
    for i, agent in enumerate(AGENTS):
        x = grid_x + COL_HEADER_W + i * CELL_W
        parts.append(
            f'<rect x="{x}" y="{grid_y}" width="{CELL_W}" height="{ROW_HEADER_H}" '
            f'fill="{header_bg}" stroke="{grid_line}" stroke-width="1"/>'
        )
        parts.append(
            f'<text x="{x + CELL_W / 2}" y="{grid_y + ROW_HEADER_H / 2 + 4}" '
            f'font-size="12" font-weight="600" text-anchor="middle" '
            f'fill="{header_text}">{esc(agent)}</text>'
        )

    # Corner cell
    parts.append(
        f'<rect x="{grid_x}" y="{grid_y}" width="{COL_HEADER_W}" '
        f'height="{ROW_HEADER_H}" fill="{header_bg}" stroke="{grid_line}" stroke-width="1"/>'
    )
    parts.append(
        f'<text x="{grid_x + 12}" y="{grid_y + ROW_HEADER_H / 2 + 4}" '
        f'font-size="11" font-weight="700" fill="{subtitle_color}" '
        f'letter-spacing="0.5">TOOL \\ AGENT</text>'
    )

    # Rows
    for r, tool in enumerate(TOOLS):
        y = grid_y + ROW_HEADER_H + r * CELL_H
        # Row header (tool name)
        parts.append(
            f'<rect x="{grid_x}" y="{y}" width="{COL_HEADER_W}" height="{CELL_H}" '
            f'fill="{header_bg}" stroke="{grid_line}" stroke-width="1"/>'
        )
        parts.append(
            f'<text x="{grid_x + 12}" y="{y + CELL_H / 2 + 4}" font-size="13" '
            f'font-weight="700" fill="{header_text}">{esc(tool)}</text>'
        )

        for c, agent in enumerate(AGENTS):
            x = grid_x + COL_HEADER_W + c * CELL_W
            category, label = MATRIX[tool][agent]
            fill, _ = CATEGORIES[category]
            parts.append(
                f'<rect x="{x + 3}" y="{y + 3}" width="{CELL_W - 6}" '
                f'height="{CELL_H - 6}" rx="6" fill="{fill}" '
                f'fill-opacity="{0.92 if dark else 0.85}"/>'
            )
            cx = x + CELL_W / 2
            cy = y + CELL_H / 2
            if " + " in label:
                first, second = label.split(" + ", 1)
                parts.append(
                    f'<text x="{cx}" y="{cy - 4}" font-size="11" font-weight="700" '
                    f'text-anchor="middle" fill="{cell_text}">{esc(first)}</text>'
                )
                parts.append(
                    f'<text x="{cx}" y="{cy + 11}" font-size="10" font-weight="600" '
                    f'text-anchor="middle" fill="{cell_text}" fill-opacity="0.9">'
                    f'+ {esc(second)}</text>'
                )
            else:
                parts.append(
                    f'<text x="{cx}" y="{cy + 4}" font-size="12" font-weight="700" '
                    f'text-anchor="middle" fill="{cell_text}">{esc(label)}</text>'
                )

    # Outer grid border
    parts.append(
        f'<rect x="{grid_x}" y="{grid_y}" width="{GRID_W}" height="{GRID_H}" '
        f'fill="none" stroke="{grid_line}" stroke-width="1.5"/>'
    )

    # Legend
    legend_y = grid_y + GRID_H + 26
    lx = grid_x
    for category, (fill, label) in CATEGORIES.items():
        parts.append(
            f'<rect x="{lx}" y="{legend_y - 12}" width="14" height="14" rx="3" '
            f'fill="{fill}"/>'
        )
        parts.append(
            f'<text x="{lx + 20}" y="{legend_y - 1}" font-size="11" '
            f'fill="{header_text}">{esc(label)}</text>'
        )
        lx += 22 + len(label) * 6.4 + 26

    # Footnote
    footnote_y = legend_y + 24
    parts.append(
        f'<text x="{grid_x}" y="{footnote_y}" font-size="10.5" '
        f'fill="{subtitle_color}">Nearly every wiring also writes a managed '
        f'instruction block (AGENTS.md / INSTRUCTIONS.md) in addition to the '
        f'mechanism shown above.</text>'
    )

    parts.append("</svg>")
    return "".join(parts)


def main() -> None:
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    assets_dir = os.path.join(repo_root, "assets")
    os.makedirs(assets_dir, exist_ok=True)

    for theme in ("dark", "light"):
        svg = render(theme)
        out_path = os.path.join(assets_dir, f"wiring-matrix-{theme}.svg")
        with open(out_path, "w", encoding="utf-8") as f:
            f.write(svg)
        print(f"wrote {out_path}")


if __name__ == "__main__":
    main()
