"""Traces the Patrone mark into terminal text, for fastfetch.

Each character cell takes whichever block or shape character, in whichever
color, best reproduces the logo under it: full and partial blocks, sextants
(2x3 cells), the diagonal "smooth mosaic" wedges and eighth blocks from
Unicode's block elements and Symbols for Legacy Computing. VTE, Patrone
Terminal's engine, draws all of them itself as exact shapes, so slopes come
out as straight diagonals rather than stair steps. Each cell's color is the
one that makes it look most like the logo over Patrone Terminal's
background, so cells the edge only partly covers are dimmer: the edges are
shaded, as smoothed text is, and the logo keeps its own shading.

The shapes are matched as VTE draws them: terminal-glyphs.png is VTE's
drawing of terminal-glyphs.txt (render_vte.py, below), so a new VTE can be
measured again. The mark's grid is shifted to where it fits best.

The output is a fastfetch logo with its colors written in (24-bit color
codes), for a "file-raw" logo in branding/fastfetch/config.jsonc:

    python tools/make_terminal_logo.py logos/patrone-logo.png fastfetch/logo.txt --preview logo.png

To measure the shapes again (on a machine with VTE's GTK 4 bindings and cage):

    python tools/make_terminal_logo.py --sheet sheet.ansi
    WLR_BACKENDS=headless WLR_RENDERER=pixman GSK_RENDERER=cairo \\
        cage -- python3 tools/render_vte.py sheet.ansi tools/terminal-glyphs.png 34 18 "JetBrains Mono 26"
"""

import argparse
import itertools
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter

TOOLS = Path(__file__).resolve().parent

# Patrone Terminal's background (src/theme.rs in billsecond/patrone-terminal), which the logo is
# matched against
BACKGROUND = np.array([0x07, 0x0A, 0x10], dtype=float)
# The wordmark: PATRONE in bold white, OS in bold Patrone Blue
WORDMARK = "1;97"
OS = "1;38;2;40;139;250"
# A cell whose best color is this close to the background is left empty
FAINT = 14
# How far a cell's color may be dimmed below the mark's: blocks and other
# rectangles a long way, which softens edges the way a smoothed picture's
# edge pixels are, but not so far that they show as faint squares; the
# diagonal wedges hardly at all (dimmed, their points looked like dark
# shards)
DIMMEST_RECTANGLE = 0.45
DIMMEST_WEDGE = 0.8
WEDGES = range(0x1FB3C, 0x1FB70)

# The sheet's layout: a row of full blocks to find the grid by, then the
# characters of terminal-glyphs.txt, 32 to a row
SHEET_COLUMNS = 32
# Each shape is compared at this many pixels a cell
MASK = (16, 36)

# Which characters may be used. "blocks" are the oldest and most widely
# drawn; "smooth" adds sextants, wedges and eighth blocks (Unicode 13, drawn
# by VTE, Kitty, WezTerm and Windows Terminal); "octants" adds 2x4 cells
# (Unicode 16, newer still).
GLYPH_SETS = {
    "blocks": [(0x20, 0x20), (0x2588, 0x2588), (0x2580, 0x2580), (0x2584, 0x2584),
               (0x258C, 0x258C), (0x2590, 0x2590), (0x2596, 0x259F)],
    # (Without the one-eighth bars, ▁ ▔ ▏ ▕, which show as hairlines)
    "smooth": [(0x1FB00, 0x1FB6F), (0x2582, 0x2587), (0x2589, 0x258E), (0x1FB82, 0x1FB8B)],
    "octants": [(0x1CD00, 0x1CDE5)],
}


def sheet_characters() -> str:
    return (TOOLS / "terminal-glyphs.txt").read_text(encoding="utf-8")


def write_sheet(path: str) -> None:
    """The text whose drawing by VTE is terminal-glyphs.png."""
    chars = sheet_characters()
    rows = ["█" * SHEET_COLUMNS]
    rows += [chars[i:i + SHEET_COLUMNS].ljust(SHEET_COLUMNS) for i in range(0, len(chars), SHEET_COLUMNS)]
    Path(path).write_text("\n".join("\x1b[97m" + row for row in rows) + "\n", encoding="utf-8")


def load_glyphs(sets: list[str]) -> tuple[list[str], np.ndarray, float]:
    """The allowed characters, their shapes as VTE draws them (coverage, 0 to
    1, MASK pixels a cell), and a cell's height over its width."""
    pixels = np.asarray(Image.open(TOOLS / "terminal-glyphs.png").convert("L")).astype(float)
    background = np.median(pixels[-8:, -8:])
    coverage = np.clip((pixels - background) / (255 - background), 0, 1)
    # The first row is full blocks: it gives the grid's corner and pitch,
    # which are fractions of a pixel. The row below starts with a space.
    ys, xs = np.nonzero(coverage > 0.5)
    first_x, first_y = xs.min(), ys.min()
    across, down = first_y + 5, first_x + 5
    cell_w = coverage[across].sum() / SHEET_COLUMNS
    cell_h = coverage[: first_y + int(3 * cell_w), down].sum()
    left = first_x - (coverage[across, first_x - 1] if first_x else 0)
    top = first_y - (coverage[first_y - 1, down] if first_y else 0)
    sheet = Image.fromarray(coverage.astype(np.float32), mode="F")

    allowed = set()
    for name in sets:
        for lo, hi in GLYPH_SETS[name]:
            allowed.update(chr(c) for c in range(lo, hi + 1))
    chars, masks = [], []
    for i, char in enumerate(sheet_characters()):
        if char not in allowed:
            continue
        row, col = 1 + i // SHEET_COLUMNS, i % SHEET_COLUMNS
        box = (left + col * cell_w, top + row * cell_h, left + (col + 1) * cell_w, top + (row + 1) * cell_h)
        masks.append(np.asarray(sheet.resize(MASK, Image.BOX, box=box)).astype(float))
        chars.append(char)
    return chars, np.stack(masks), cell_h / cell_w


def blur(image: np.ndarray, radius: float) -> np.ndarray:
    """A Gaussian blur of each channel."""
    x = np.arange(-int(3 * radius), int(3 * radius) + 1)
    kernel = np.exp(-x ** 2 / (2 * radius ** 2))
    kernel /= kernel.sum()
    for axis in (0, 1):
        image = np.apply_along_axis(lambda line: np.convolve(line, kernel, mode="same"), axis, image)
    return image


def load_mark(path: str, scale: int = 6) -> np.ndarray:
    """The logo over Patrone Terminal's background (RGB, 0 to 255), enlarged
    and cropped to the mark, with a margin of the background. The logo's
    edges are drawn darker; its colors are taken from just inside instead,
    so the edge cells' shading comes from how much the mark covers them."""
    image = Image.open(path).convert("RGBA")
    rgba = np.asarray(image).astype(float)
    # The inside: opaque, two pixels in from the edge
    inside = np.asarray(Image.fromarray(((rgba[..., 3] > 250) * 255).astype(np.uint8))
                        .filter(ImageFilter.MinFilter(5))).astype(float) / 255
    weight = blur(inside, 3)[..., None]
    color = blur(rgba[..., :3] * inside[..., None], 3) / np.maximum(weight, 1e-6)
    size = (image.width * scale, image.height * scale)
    color = np.stack([np.asarray(Image.fromarray(color[..., c].astype(np.float32), mode="F")
                                 .resize(size, Image.BILINEAR)) for c in range(3)], axis=-1)
    alpha = np.asarray(image.getchannel("A").resize(size, Image.LANCZOS)).astype(float)[..., None] / 255
    color = np.clip(color, 0, 255)
    over = color * alpha + BACKGROUND * (1 - alpha)
    ys, xs = np.nonzero(alpha[..., 0] > 0.5)
    # The logo as seen, and its colors with how much each pixel is covered
    mark = np.concatenate([over, color * alpha, alpha], axis=-1)
    return mark[ys.min():ys.max() + 1, xs.min():xs.max() + 1]


def cells(mark: np.ndarray, width: int, aspect: float, ox: float, oy: float) -> tuple[np.ndarray, np.ndarray]:
    """The logo under each cell, MASK pixels a cell, for a grid `width` cells
    wide shifted by (ox, oy) cells, shape (rows, width, pixels, 3); and the
    mark's own color there, (rows, width, 3)."""
    cell_w = mark.shape[1] / (width - 1)
    cell_h = cell_w * aspect
    rows = int(np.ceil(mark.shape[0] / cell_h + oy))
    pad_x, pad_y = int(np.ceil(cell_w)), int(np.ceil(cell_h))
    padded = np.zeros((mark.shape[0] + 2 * pad_y, mark.shape[1] + 2 * pad_x, mark.shape[2]))
    padded[..., :3] = BACKGROUND
    padded[pad_y:pad_y + mark.shape[0], pad_x:pad_x + mark.shape[1]] = mark
    channels = [Image.fromarray(padded[..., c].astype(np.float32), mode="F") for c in range(mark.shape[2])]
    seen = np.zeros((rows, width, MASK[0] * MASK[1], 3))
    own = np.zeros((rows, width, 3))
    for row, col in itertools.product(range(rows), range(width)):
        x0 = pad_x + (col - ox) * cell_w
        y0 = pad_y + (row - oy) * cell_h
        box = (x0, y0, x0 + cell_w, y0 + cell_h)
        for c in range(3):
            seen[row, col, :, c] = np.asarray(channels[c].resize(MASK, Image.BOX, box=box)).ravel()
        means = [np.asarray(channels[c].resize((1, 1), Image.BOX, box=box))[0, 0] for c in range(3, 7)]
        own[row, col] = np.array(means[:3]) / max(means[3], 1e-6)
    return seen, own


def fit(target: np.ndarray, own: np.ndarray, masks: np.ndarray, dims: np.ndarray) -> tuple[np.ndarray, np.ndarray, float]:
    """For every cell, the shape and shade that make it look most like the
    logo. A cell shows k times the mark's own color there where its shape
    covers it, and the background elsewhere; for each shape the best k is a
    least-squares fit, kept between the shape's dimmest (`dims`) and 1, and
    the shape is the one with the smallest error left."""
    rows, width, pixels, _ = target.shape
    d = target.reshape(-1, pixels, 3) - BACKGROUND             # cells, pixels, 3
    c = own.reshape(-1, 3)                                     # cells, 3
    m = masks.reshape(len(masks), -1)                          # shapes, pixels
    mm = (m ** 2).sum(1)                                       # shapes
    # With u = m c and v = m B, the error is |d + v - k u|^2
    du = np.einsum("gp,np->ng", m, (d * c[:, None, :]).sum(2)) + mm[None] * (c @ BACKGROUND)[:, None]
    uu = mm[None] * (c ** 2).sum(1)[:, None]
    dd = (d ** 2).sum((1, 2))[:, None] + 2 * np.einsum("gp,np->ng", m, d @ BACKGROUND) \
        + mm[None] * (BACKGROUND ** 2).sum()
    k = np.clip(du / np.maximum(uu, 1e-9), dims[None], 1)
    cost = dd - 2 * k * du + k ** 2 * uu
    choice = cost.argmin(1)
    picked = np.arange(len(choice))
    color = k[picked, choice][:, None] * c
    return choice.reshape(rows, width), color.reshape(rows, width, 3), float(cost[picked, choice].sum())


def trace(path: str, width: int, sets: list[str]) -> list[list[tuple[str, tuple[int, int, int] | None]]]:
    """The logo's rows of (character, color) cells."""
    chars, masks, aspect = load_glyphs(sets)
    dims = np.array([DIMMEST_WEDGE if ord(char) in WEDGES else DIMMEST_RECTANGLE for char in chars])
    mark = load_mark(path)
    best = None
    for ox, oy in itertools.product(np.linspace(0, 1, 6, endpoint=False), repeat=2):
        choice, color, cost = fit(*cells(mark, width, aspect, ox, oy), masks, dims)
        if best is None or cost < best[2]:
            best = (choice, color, cost)
    choice, color, _ = best
    grid = []
    for row in range(choice.shape[0]):
        line = []
        for col in range(width):
            char = chars[choice[row, col]]
            rgb = tuple(int(round(v)) for v in color[row, col])
            if char == " " or np.abs(np.array(rgb) - BACKGROUND).max() < FAINT:
                line.append((" ", None))
            else:
                line.append((char, rgb))
        grid.append(line)
    # A cell on its own, with nothing beside it, is a stray fleck
    for row, line in enumerate(grid):
        for col, (char, rgb) in enumerate(line):
            neighbors = [grid[r][c] for r in range(row - 1, row + 2) for c in range(col - 1, col + 2)
                         if (r, c) != (row, col) and 0 <= r < len(grid) and 0 <= c < len(grid[r])]
            if rgb is not None and all(n[1] is None for n in neighbors):
                line[col] = (" ", None)
    for line in grid:
        while line and line[-1][0] == " ":
            line.pop()
    while grid and not grid[0]:
        grid.pop(0)
    while grid and not grid[-1]:
        grid.pop()
    return grid


def centered(text: str, width: int) -> str:
    return " " * max(0, (width - len(text)) // 2) + text


def ansi(grid: list, width: int) -> list[str]:
    lines = []
    for row in grid:
        # Not bold, which fastfetch turns on for the logo: VTE draws bold
        # wedges and sextants thicker, and the edges came out streaked
        line, current = "\x1b[22m", None
        for char, rgb in row:
            if rgb is not None and rgb != current:
                line += "\x1b[38;2;%d;%d;%dm" % rgb
                current = rgb
            line += char
        lines.append(line + ("\x1b[0m" if current else ""))
    indent = centered("P A T R O N E O S", width)[:-len("P A T R O N E O S")]
    lines += ["", f"{indent}\x1b[{WORDMARK}mP A T R O N E \x1b[{OS}mO S\x1b[0m"]
    return lines


def preview(grid: list, path: str) -> None:
    """Draws the logo's shapes as VTE does (from terminal-glyphs.png) on
    Patrone Terminal's background; render_vte.py draws the whole thing."""
    chars, masks, aspect = load_glyphs(list(GLYPH_SETS))
    index = {char: i for i, char in enumerate(chars)}
    cell_w, cell_h = 18, int(round(18 * aspect))
    canvas = np.ones(((len(grid) + 2) * cell_h, (max(map(len, grid)) + 2) * cell_w, 3)) * BACKGROUND
    for row, line in enumerate(grid, start=1):
        for col, (char, rgb) in enumerate(line, start=1):
            if rgb is None:
                continue
            mask = Image.fromarray((masks[index[char]] * 255).astype(np.uint8)).resize((cell_w, cell_h), Image.BILINEAR)
            shape = np.asarray(mask).astype(float)[..., None] / 255
            region = canvas[row * cell_h:(row + 1) * cell_h, col * cell_w:(col + 1) * cell_w]
            region[:] = region * (1 - shape) + np.array(rgb) * shape
    Image.fromarray(canvas.astype(np.uint8)).save(path)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("logo", nargs="?", help="the mark, on a transparent background")
    parser.add_argument("output", nargs="?")
    parser.add_argument("--width", type=int, default=32, help="width of the mark in characters")
    parser.add_argument("--glyphs", default="blocks,smooth,octants",
                        help="the character sets to use, of: " + ", ".join(GLYPH_SETS))
    parser.add_argument("--preview", help="also draw the mark to this PNG")
    parser.add_argument("--sheet", help="write the text to draw with render_vte.py for terminal-glyphs.png, and stop")
    args = parser.parse_args()

    if args.sheet:
        write_sheet(args.sheet)
        return
    grid = trace(args.logo, args.width, args.glyphs.split(","))
    lines = ansi(grid, max(map(len, grid)))
    with open(args.output, "w", encoding="utf-8", newline="\n") as out:
        out.write("\n".join(lines) + "\n")
    if args.preview:
        preview(grid, args.preview)


if __name__ == "__main__":
    main()
