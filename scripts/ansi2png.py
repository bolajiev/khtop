#!/usr/bin/env python3
"""Convert tmux capture-pane -pe ANSI output to a PNG with terminal colors."""
import re, sys, os
from PIL import Image, ImageDraw, ImageFont

FONT = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"
FONT_BOLD = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf"
CELL_W, CELL_H, FONT_PX = 10, 20, 16

# xterm-256 palette basics (16 colors + a few extras)
ANSI = {
    30: "#000000", 31: "#cd3131", 32: "#0dbc79", 33: "#e5e510",
    34: "#2472c8", 35: "#bc3fbc", 36: "#11a8cd", 37: "#e5e5e5",
    90: "#666666", 91: "#f14c4c", 92: "#23d18b", 93: "#f5f543",
    94: "#3b8eea", 95: "#d670d6", 96: "#29b8db", 97: "#ffffff",
    40: "#000000", 41: "#cd3131", 42: "#0dbc79", 43: "#e5e510",
    44: "#2472c8", 45: "#bc3fbc", 46: "#11a8cd", 47: "#e5e5e5",
    100: "#666666", 101: "#f14c4c", 102: "#23d18b", 103: "#f5f543",
    104: "#3b8eea", 105: "#d670d6", 106: "#29b8db", 107: "#ffffff",
}
DEFAULT_FG = "#e8e8e8"
DEFAULT_BG = "#0a0e16"

def cell_grid(ansi_text):
    """Return (grid, styles) where grid[r][c] = char, styles[r][c] = (fg,bg,bold)."""
    grid, styles = [], []
    fg, bg, bold = DEFAULT_FG, None, False

    def push_line(row, st):
        grid.append(row); styles.append(st)

    row, st = [], []
    # split on cursor-move escape codes; tmux capture emits per-line output
    lines = ansi_text.split("\n")
    for line in lines:
        i = 0
        while i < len(line):
            ch = line[i]
            if ch == "\x1b":
                m = re.match(r"\x1b\[([0-9;]*)m", line[i:])
                if m:
                    codes = [int(x) for x in m.group(1).split(";") if x] or [0]
                    j = 0
                    while j < len(codes):
                        c = codes[j]
                        if c == 0: fg, bg, bold = DEFAULT_FG, None, False
                        elif c == 1: bold = True
                        elif c == 22: bold = False
                        elif c in ANSI: fg = ANSI[c]
                        elif c == 39: fg = DEFAULT_FG
                        elif c == 49: bg = None
                        else:
                            if c in (38, 48) and j + 2 < len(codes) and codes[j+1] == 5:
                                n = codes[j+2]
                                # 256-color: map to rgb via palette approximation
                                rgb = xterm256(n)
                                if c == 38: fg = rgb
                                else: bg = rgb
                                j += 2
                        j += 1
                    i += m.end() - m.start()
                    continue
                m = re.match(r"\x1b\[([0-9;]*)[A-Za-z]", line[i:])
                if m:
                    i += m.end() - m.start()
                    continue
                m = re.match(r"\x1b\][^\x07]*\x07", line[i:])
                if m:
                    i += m.end() - m.start()
                    continue
                i += 1
                continue
            if ch == "\r":
                i += 1
                continue
            row.append((ch, (fg, bg, bold)))
            i += 1
        # pad row to consistent width
        if row:
            push_line(row, st)
        row, st = [], []
    return grid, styles

def xterm256(n):
    if n < 16:
        return ANSI.get(30 + n if n < 8 else 90 + n - 8, DEFAULT_FG)
    if n < 232:
        n -= 16
        r, g, b = n // 36, (n // 6) % 6, n % 6
        def v(x): return 0 if x == 0 else 55 + x * 40
        return "#{:02x}{:02x}{:02x}".format(v(r), v(g), v(b))
    v = 8 + (n - 232) * 10
    return "#{:02x}{:02x}{:02x}".format(v, v, v)

def render(ansi_path, out_path):
    text = open(ansi_path, encoding="utf-8", errors="replace").read()
    grid, styles = cell_grid(text)
    rows, cols = len(grid), max((len(r) for r in grid), default=0)
    img = Image.new("RGB", (cols * CELL_W + 16, rows * CELL_H + 16), "#0a0e16")
    d = ImageDraw.Draw(img)
    fonts = {False: ImageFont.truetype(FONT, FONT_PX), True: ImageFont.truetype(FONT_BOLD, FONT_PX)}
    for r, row in enumerate(grid):
        for c, (ch, (fg, bg, bold)) in enumerate(row):
            x, y = 8 + c * CELL_W, 8 + r * CELL_H
            if bg:
                d.rectangle([x, y, x + CELL_W, y + CELL_H], fill=bg)
            d.text((x, y), ch, font=fonts[bold], fill=fg)
    img.save(out_path)
    print(f"{out_path}: {img.size[0]}x{img.size[1]}")

if __name__ == "__main__":
    src = sys.argv[1]
    out = sys.argv[2] if len(sys.argv) > 2 else os.path.splitext(src)[0] + ".png"
    render(src, out)
