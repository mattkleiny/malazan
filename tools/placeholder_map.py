"""Generates a placeholder map for the game to draw until a real one is painted.

The map is one tall continent in the manner of a dark fantasy novel's frontispiece: an
ice-bound north, a great inland lake, mountain spines down either flank and wastes in the
south. Its outline, ranges and regions are laid out by hand below; noise roughens the rest.

Writes the three map sources into godot/map: definitions.csv, locations.png and
heightmap.png. Needs numpy and Pillow. Run from the repository root:

    python3 tools/placeholder_map.py
"""

from collections import defaultdict, deque
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

SEED = 11
WIDTH, HEIGHT = 1536, 2048  # Size of the location image (map units).
RELIEF_SCALE = 2  # Map units per heightmap sample.
SEA_LEVEL = 0.3  # Must match the map/sea_level config variable.

SHELF_REACH = 80  # How far from the shore the seabed keeps falling (map units).
SHELF_DROP = 0.2  # How far it falls over that reach (heightmap range); the terrain shader rules its shore lines by it.
COAST_ROUGHNESS = 0.5  # How far noise pushes the coast off the drawn outline, as a share of its softened edge.

LAND_CELL = 24  # Spacing of land location seeds (map units).
LAKE_CELL = 48  # Spacing of lake location seeds (map units).
SEA_CELL = 96  # Spacing of sea location seeds (map units).
TIER_CELLS = {"province": 64, "area": 160}
SEA_TIER_CELLS = {"province": 256, "area": 512}
SMALL_LANDMASS = 40  # Landmasses with fewer locations belong to the mainland's divisions.
SPECK = 30  # Landmasses of fewer heightmap samples are sunk.

MOUNTAINS_ABOVE = 0.36  # Mean height above the sea (0..1) from which a location is mountains.
HILLS_ABOVE = 0.17

CONTINENT = "Korthaelis"
OCEAN = "The Deep"
LAKE = "Lake Ysul"

# Everything below is laid out on the unit square: x east from the western edge, y south from the northern.
MAINLAND = [
  (0.22, 0.07), (0.34, 0.045), (0.48, 0.06), (0.60, 0.05), (0.72, 0.08), (0.80, 0.13), (0.78, 0.20), (0.83, 0.26),
  (0.86, 0.33), (0.80, 0.38), (0.74, 0.40), (0.76, 0.45), (0.84, 0.49), (0.88, 0.55), (0.84, 0.62), (0.76, 0.66),
  (0.72, 0.72), (0.74, 0.79), (0.70, 0.86), (0.62, 0.91), (0.54, 0.93), (0.47, 0.90), (0.44, 0.84), (0.40, 0.78),
  (0.33, 0.74), (0.30, 0.68), (0.34, 0.62), (0.31, 0.56), (0.24, 0.53), (0.18, 0.48), (0.20, 0.42), (0.27, 0.39),
  (0.30, 0.34), (0.24, 0.30), (0.16, 0.27), (0.12, 0.21), (0.14, 0.13),
]  # fmt: skip

# Each island is a centre, a radius either way and the region it makes.
ISLANDS = [
  ((0.27, 0.905), (0.055, 0.032), "Isle of Masks"),
  ((0.915, 0.215), (0.028, 0.03), "The Shrikes"),
  ((0.075, 0.385), (0.024, 0.034), "Widow's Rock"),
  ((0.09, 0.66), (0.03, 0.022), "Gallowmere"),
]

LAKE_SHORE = [(0.45, 0.50), (0.50, 0.475), (0.57, 0.49), (0.60, 0.525), (0.56, 0.555), (0.49, 0.55), (0.455, 0.53)]

# Mountain ranges: the line each follows and how broad it stands.
RANGES = [
  ([(0.21, 0.43), (0.26, 0.48), (0.32, 0.515), (0.36, 0.57), (0.365, 0.63)], 0.030),
  ([(0.66, 0.43), (0.71, 0.48), (0.735, 0.53), (0.72, 0.60), (0.68, 0.645)], 0.026),
  ([(0.19, 0.165), (0.28, 0.195), (0.37, 0.18), (0.44, 0.15)], 0.022),
  ([(0.50, 0.80), (0.56, 0.835), (0.62, 0.86)], 0.016),
  ([(0.24, 0.90), (0.30, 0.91)], 0.010),
]

# Each region grows from the seeds it is given, and its low ground is clothed by the rule beside it.
REGIONS = {
  "The Rimeshelf": ([(0.30, 0.075), (0.50, 0.075), (0.68, 0.09)], "tundra"),
  "Hollun Plateau": ([(0.22, 0.19), (0.38, 0.18)], "plateau"),
  "The Shrike Coast": ([(0.58, 0.19), (0.74, 0.22), (0.78, 0.31)], "woodland"),
  "Blackreed Fens": ([(0.21, 0.31), (0.30, 0.29)], "fen"),
  "Mossgrave Wood": ([(0.40, 0.31), (0.37, 0.40)], "forest"),
  "Tessari Plain": ([(0.56, 0.33), (0.66, 0.37), (0.52, 0.42)], "grassland"),
  "Kelvarn Range": ([(0.24, 0.46), (0.33, 0.53), (0.36, 0.61)], "foothills"),
  "The Dhavran Reach": ([(0.44, 0.47), (0.42, 0.56), (0.55, 0.59), (0.62, 0.50)], "downs"),
  "Cinder Peaks": ([(0.70, 0.47), (0.73, 0.56), (0.69, 0.63)], "hills"),
  "The Pallid Shore": ([(0.81, 0.52), (0.82, 0.60)], "strand"),
  "Veilwood": ([(0.36, 0.70), (0.42, 0.76)], "forest"),
  "The Sere Plain": ([(0.50, 0.67), (0.62, 0.69), (0.50, 0.74)], "steppe"),
  "Umber Wastes": ([(0.60, 0.77), (0.67, 0.81), (0.55, 0.79)], "desert"),
  "The Grey Marches": ([(0.52, 0.87), (0.62, 0.87)], "downs"),
}
ISLAND_COVER = "foothills"

# The oceans, named by where they lie.
SEAS = {
  "Ocean of Sighs": [(0.05, 0.25), (0.08, 0.50), (0.12, 0.75)],
  "Sea of Rime": [(0.30, 0.0), (0.70, 0.0), (0.95, 0.08)],
  "The Gallows Sea": [(0.95, 0.35), (0.97, 0.55), (0.90, 0.75)],
  "Reaver's Deep": [(0.20, 0.99), (0.50, 0.99), (0.85, 0.95)],
}

OUTPUT = Path(__file__).resolve().parent.parent / "godot" / "map"

rng = np.random.default_rng(SEED)


def fractal_noise(width, height, cells_x, octaves=6, persistence=0.5):
  """Fractal value noise in 0..1."""
  total = np.zeros((height, width))
  amplitude, norm = 1.0, 0.0
  xs = np.arange(width) / width
  ys = np.arange(height) / height

  for octave in range(octaves):
    gx = cells_x * 2**octave
    gy = max(2, gx * height // width)
    grid = rng.random((gy + 1, gx + 1))

    fx = xs * gx
    fy = ys * gy
    x0 = np.clip(np.floor(fx).astype(int), 0, gx - 1)
    y0 = np.clip(np.floor(fy).astype(int), 0, gy - 1)
    tx = fx - x0
    ty = fy - y0
    tx = tx * tx * (3 - 2 * tx)
    ty = ty * ty * (3 - 2 * ty)

    top = grid[y0][:, x0] * (1 - tx) + grid[y0][:, x0 + 1] * tx
    bottom = grid[y0 + 1][:, x0] * (1 - tx) + grid[y0 + 1][:, x0 + 1] * tx
    total += (top * (1 - ty)[:, None] + bottom * ty[:, None]) * amplitude

    norm += amplitude
    amplitude *= persistence

  return total / norm


def upsample(heights, factor):
  """Blends heights up to a finer grid the way the game samples them."""
  rows, columns = heights.shape
  xs = (np.arange(columns * factor) + 0.5) / factor - 0.5
  ys = np.clip((np.arange(rows * factor) + 0.5) / factor - 0.5, 0, rows - 1)
  xs = np.clip(xs, 0, columns - 1)
  x0 = np.floor(xs).astype(int)
  y0 = np.floor(ys).astype(int)
  tx, ty = xs - x0, ys - y0
  x1, y1 = np.minimum(x0 + 1, columns - 1), np.minimum(y0 + 1, rows - 1)

  top = heights[y0][:, x0] * (1 - tx) + heights[y0][:, x1] * tx
  bottom = heights[y1][:, x0] * (1 - tx) + heights[y1][:, x1] * tx
  return top * (1 - ty)[:, None] + bottom * ty[:, None]


def blurred(field, sigma):
  """A field smoothed by a Gaussian of the given spread (samples)."""
  rows, columns = field.shape
  fy = np.fft.fftfreq(rows)[:, None]
  fx = np.fft.rfftfreq(columns)[None, :]
  falloff = np.exp(-2 * np.pi**2 * sigma**2 * (fx**2 + fy**2))
  return np.fft.irfft2(np.fft.rfft2(field) * falloff, s=field.shape)


def drawn(width, height, draw):
  """A 0..1 mask of whatever `draw` paints, given a canvas and the scale from the unit square onto it."""
  canvas = Image.new("L", (width, height), 0)
  draw(ImageDraw.Draw(canvas), lambda points: [(x * width, y * height) for x, y in points])
  return np.asarray(canvas, dtype=float) / 255


def distance_from(mask, limit):
  """How many samples each point lies from the mask, up to a limit."""
  distance = np.where(mask, 0.0, float(limit))
  reached = mask.copy()

  for step in range(1, limit):
    grown = reached.copy()
    grown[1:] |= reached[:-1]
    grown[:-1] |= reached[1:]
    grown[:, 1:] |= reached[:, :-1]
    grown[:, :-1] |= reached[:, 1:]
    if step % 2 == 0:
      # Every other step also reaches diagonally, which rounds the corners off.
      grown[1:, 1:] |= reached[:-1, :-1]
      grown[1:, :-1] |= reached[:-1, 1:]
      grown[:-1, 1:] |= reached[1:, :-1]
      grown[:-1, :-1] |= reached[1:, 1:]
    distance[grown & ~reached] = step * 0.85
    reached = grown

  return distance


def bodies(mask):
  """Labels each connected body of the mask from 1, and gives their sizes."""
  rows, columns = mask.shape
  labels = np.zeros(mask.shape, dtype=np.int32)
  open_cells = mask.tolist()
  sizes = [0]

  for row in range(rows):
    for column in range(columns):
      if not open_cells[row][column]:
        continue

      label = len(sizes)
      open_cells[row][column] = False
      queue, members = deque([(row, column)]), []
      while queue:
        y, x = queue.popleft()
        members.append((y, x))
        for ny, nx in ((y - 1, x), (y + 1, x), (y, x - 1), (y, x + 1)):
          if 0 <= ny < rows and 0 <= nx < columns and open_cells[ny][nx]:
            open_cells[ny][nx] = False
            queue.append((ny, nx))

      ys, xs = zip(*members)
      labels[list(ys), list(xs)] = label
      sizes.append(len(members))

  return labels, sizes


def heightmap():
  """The heightmap, and the mask of where its inland water lies."""
  width, height = WIDTH // RELIEF_SCALE, HEIGHT // RELIEF_SCALE

  def outline(draw, scale):
    draw.polygon(scale(MAINLAND), fill=255)
    for (x, y), (rx, ry), _ in ISLANDS:
      draw.ellipse(scale([(x - rx, y - ry), (x + rx, y + ry)]), fill=255)

  lake = drawn(width, height, lambda draw, scale: draw.polygon(scale(LAKE_SHORE), fill=255))

  # The drawn outline, softened, then pushed about by noise so the coast runs ragged.
  drawn_land = drawn(width, height, outline)
  shape = blurred(drawn_land, 10) - blurred(lake, 6) * 1.6
  roughness = fractal_noise(width, height, 6, octaves=7, persistence=0.6)
  roughness = (roughness - roughness.mean()) / roughness.std()
  # Far out to sea the noise dies away, so no land rises where none was drawn.
  near_land = np.clip(np.maximum(blurred(drawn_land, 45) * 4 - 0.4, blurred(drawn_land, 16) * 5), 0, 1)
  land = shape + roughness * near_land * COAST_ROUGHNESS > 0.5

  # Specks of land are sunk, and stray pools filled, so only the drawn lake lies inland.
  masses, sizes = bodies(land)
  land &= np.array(sizes)[masses] >= SPECK
  waters, sizes = bodies(~land)
  open_sea = waters == waters[0, 0]
  for water in range(1, len(sizes)):
    pool = waters == water
    if not open_sea[pool].any() and lake[pool].max() < 0.5:
      land |= pool
  waters, _ = bodies(~land)
  inland_water = ~land & (waters != waters[0, 0])

  def ranges(breadth):
    def draw_ranges(draw, scale):
      for line, across in RANGES:
        draw.line(scale(line), fill=255, width=max(1, round(across * breadth * width)), joint="curve")

    return drawn(width, height, draw_ranges)

  peaks = np.clip(blurred(ranges(1.0), 9), 0, None)
  peaks /= peaks.max()
  foothills = np.clip(blurred(ranges(1.8), 14), 0, None)
  foothills /= foothills.max()
  crags = (1 - np.abs(fractal_noise(width, height, 10, octaves=5) * 2 - 1)) ** 2
  swell = fractal_noise(width, height, 8, octaves=5)

  inland = np.clip(distance_from(~land, 64) * RELIEF_SCALE / 120, 0, 1)
  ground = (
    SEA_LEVEL + 0.008 + inland * 0.05 + swell * 0.03 + foothills * swell * 0.16 + peaks**1.3 * (0.2 + crags * 0.42)
  )

  # The seabed falls steadily away from the shore, so lines of equal depth run alongside it.
  offshore = distance_from(land, SHELF_REACH // RELIEF_SCALE + 2) * RELIEF_SCALE
  offshore = blurred(offshore, 1.5)
  seabed = SEA_LEVEL - 0.004 - np.clip(offshore / SHELF_REACH, 0, 1) * SHELF_DROP - swell * 0.03

  return np.clip(np.where(land, ground, seabed), 0, 1), distance_from(inland_water, 4) < 3


def seeds(spacing):
  """One seed jittered within each cell of a grid."""
  columns, rows = WIDTH // spacing, HEIGHT // spacing
  jitter = rng.random((rows, columns, 2)) * 0.8 + 0.1
  cx, cy = np.meshgrid(np.arange(columns), np.arange(rows))
  return (cx + jitter[..., 0]) * spacing, (cy + jitter[..., 1]) * spacing


def nearest_seed(px, py, sx, sy, spacing):
  """Index of the nearest seed to each point, searching neighbouring cells."""
  rows, columns = sx.shape
  cx = np.floor(px / spacing).astype(int)
  cy = np.floor(py / spacing).astype(int)
  best = np.full(px.shape, np.inf)
  index = np.zeros(px.shape, dtype=np.int64)

  for oy in (-1, 0, 1):
    for ox in (-1, 0, 1):
      ny = np.clip(cy + oy, 0, rows - 1)
      nx = np.clip(cx + ox, 0, columns - 1)
      dx = sx[ny, nx] - px
      dy = sy[ny, nx] - py
      distance = dx * dx + dy * dy
      closer = distance < best
      best = np.where(closer, distance, best)
      index = np.where(closer, ny * columns + nx, index)

  return index


def nearest_named(places, x, y):
  """The name whose seeds lie nearest a point of the map."""

  def distance(name):
    return min((sx * WIDTH - x) ** 2 + (sy * HEIGHT - y) ** 2 for sx, sy in places[name])

  return min(places, key=distance)


OPENINGS = "dar gad rhi cap mor bar seg kel pan lae nath mot kor sal tul oth urd dren jhag tes ber eth ond ruth hal".split()
MIDDLES = "u a i o e ara uji ori eda ana ilo esh ath ul".split()
ENDINGS = "stan robi vi tan anth ghast leh van nion deron ilog rhun ral ket dris mar oss hend rell ghal ik eth".split()


def name(taken):
  while True:
    parts = [rng.choice(OPENINGS), rng.choice(MIDDLES), rng.choice(ENDINGS)]
    word = "".join(parts if rng.random() < 0.6 else parts[::2]).capitalize()
    if word not in taken:
      taken.add(word)
      return word


def cover(rule, wetness, northing):
  """The terrain a rule clothes low ground in, given how wet it is (0..1) and how far south (0..1)."""
  if rule == "tundra" or northing < 0.1:
    return "tundra"
  if rule == "plateau":
    return "hills" if wetness > 0.4 else "tundra"
  if rule == "woodland":
    return "forest" if wetness > 0.58 else "plains"
  if rule == "fen":
    return "marsh" if wetness < 0.65 else "forest"
  if rule == "forest":
    return "plains" if wetness < 0.15 else "forest"
  if rule == "grassland":
    return "hills" if wetness > 0.85 else "plains"
  if rule == "foothills":
    return "forest" if wetness > 0.45 else "hills"
  if rule == "downs":
    return "hills" if wetness > 0.62 else "plains"
  if rule == "strand":
    return "marsh" if wetness > 0.8 else "plains"
  if rule == "steppe":
    return "desert" if wetness < 0.35 else "plains"
  return rule


def main():
  heights, lake_mask = heightmap()
  relief = upsample(heights, RELIEF_SCALE)
  is_land = relief >= SEA_LEVEL
  is_lake = ~is_land & np.repeat(np.repeat(lake_mask, RELIEF_SCALE, axis=0), RELIEF_SCALE, axis=1)

  py, px = np.mgrid[0:HEIGHT, 0:WIDTH].astype(float) + 0.5
  warp = 6
  px_warped = np.clip(px + (fractal_noise(WIDTH, HEIGHT, 32, octaves=3) - 0.5) * warp * 2, 0, WIDTH - 0.01)
  py_warped = np.clip(py + (fractal_noise(WIDTH, HEIGHT, 32, octaves=3) - 0.5) * warp * 2, 0, HEIGHT - 0.01)

  # Land, lake and sea are each cut into locations around their own grid of seeds.
  grids = [seeds(LAND_CELL), seeds(LAKE_CELL), seeds(SEA_CELL)]
  spacings = [LAND_CELL, LAKE_CELL, SEA_CELL]
  kind_of_cell = np.where(is_land, 0, np.where(is_lake, 1, 2))
  cells = np.zeros((HEIGHT, WIDTH), dtype=np.int64)
  offset = 0
  for kind, ((sx, sy), spacing) in enumerate(zip(grids, spacings)):
    ids = nearest_seed(px_warped, py_warped, sx, sy, spacing) + offset
    cells = np.where(kind_of_cell == kind, ids, cells)
    offset += sx.size

  used, cells = np.unique(cells, return_inverse=True)
  cells = cells.reshape(HEIGHT, WIDTH)
  all_x = np.concatenate([sx.ravel() for sx, _ in grids])[used]
  all_y = np.concatenate([sy.ravel() for _, sy in grids])[used]
  kinds = np.searchsorted(np.cumsum([sx.size for sx, _ in grids]), used, side="right")
  water = kinds > 0
  count = len(used)

  # Neighbours, east and south.
  neighbours = defaultdict(set)
  for a, b in ((cells[:, :-1], cells[:, 1:]), (cells[:-1], cells[1:])):
    edge = a != b
    for x, y in set(zip(a[edge].tolist(), b[edge].tolist())):
      neighbours[x].add(y)
      neighbours[y].add(x)

  # Each location's mean height above the sea, from the pixels painted in it.
  sums = np.bincount(cells.ravel(), weights=relief.ravel(), minlength=count)
  elevation = (sums / np.bincount(cells.ravel(), minlength=count) - SEA_LEVEL) / (1 - SEA_LEVEL)
  damp = fractal_noise(WIDTH, HEIGHT, 9, octaves=4)[np.clip(all_y.astype(int), 0, HEIGHT - 1), all_x.astype(int)]
  wetness = np.argsort(np.argsort(damp)) / max(count - 1, 1)

  # Landmasses: the mainland is the largest, and each drawn island is whichever lies under its centre.
  landmass = np.full(count, -1)
  sizes = []
  for start in range(count):
    if water[start] or landmass[start] >= 0:
      continue
    landmass[start] = len(sizes)
    queue, size = deque([start]), 0
    while queue:
      here = queue.popleft()
      size += 1
      for there in neighbours[here]:
        if not water[there] and landmass[there] < 0:
          landmass[there] = len(sizes)
          queue.append(there)
    sizes.append(size)

  islands = {}
  for (x, y), _, title in ISLANDS:
    mass = landmass[cells[int(y * HEIGHT), int(x * WIDTH)]]
    if mass >= 0 and mass != int(np.argmax(sizes)):
      islands[mass] = title

  def tiers(spacings):
    return {tier: (*seeds(spacing), spacing) for tier, spacing in spacings.items()}

  land_tiers, sea_tiers = tiers(TIER_CELLS), tiers(SEA_TIER_CELLS)

  def division(chain, tier, x, y):
    tx, ty, spacing = chain[tier]
    index = int(nearest_seed(np.array([x]), np.array([y]), tx, ty, spacing)[0])
    return index, tx.ravel()[index], ty.ravel()[index]

  taken = set()
  names = defaultdict(dict)

  def named(tier, key, suffix=""):
    if key not in names[tier]:
      names[tier][key] = name(taken) + suffix
    return names[tier][key]

  def placed(index):
    """The province, area, region, continent and terrain of a location."""
    x, y = all_x[index], all_y[index]

    if kinds[index] == 2:
      province, sx, sy = division(sea_tiers, "province", x, y)
      area, sx, sy = division(sea_tiers, "area", sx, sy)
      region = nearest_named(SEAS, sx, sy)
      return named("province", ("sea", province), " Waters"), named("area", ("sea", area, region), " Deeps"), region, OCEAN, "sea"

    if kinds[index] == 1:
      return f"{LAKE} Shallows", LAKE, LAKE, CONTINENT, "lake"

    province, sx, sy = division(land_tiers, "province", x, y)
    island = islands.get(landmass[index])
    if island:
      rule = ISLAND_COVER
      province, area, region = named("province", (island, province)), island, island
    else:
      area, sx, sy = division(land_tiers, "area", sx, sy)
      region = nearest_named({title: spots for title, (spots, _) in REGIONS.items()}, sx, sy)
      rule = REGIONS[region][1]
      province, area = named("province", ("land", province)), named("area", ("land", area, region))

    if elevation[index] > MOUNTAINS_ABOVE:
      terrain = "mountains"
    elif elevation[index] > HILLS_ABOVE:
      terrain = "hills"
    else:
      terrain = cover(rule, wetness[index], y / HEIGHT)

    return province, area, region, CONTINENT, terrain

  colors = set()
  rows = ["red,green,blue,location,province,area,region,continent,terrain"]
  palette = np.zeros((count, 3), dtype=np.uint8)
  suffixes = ["", " Mere", " Sound"]

  for index in range(count):
    while True:
      color = tuple(int(c) for c in rng.integers(0, 256, 3))
      if color not in colors:
        colors.add(color)
        break
    palette[index] = color

    rows.append(",".join(str(part) for part in [*color, named("location", index, suffixes[kinds[index]]), *placed(index)]))

  OUTPUT.mkdir(parents=True, exist_ok=True)
  (OUTPUT / "definitions.csv").write_text("\n".join(rows) + "\n")
  Image.fromarray(palette[cells]).save(OUTPUT / "locations.png", optimize=True)
  Image.fromarray((heights * 65535).astype(np.uint16)).save(OUTPUT / "heightmap.png")
  print(f"{count} locations ({int(water.sum())} water), {len(islands)} islands")


if __name__ == "__main__":
  main()
