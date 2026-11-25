"""Generates a placeholder fantasy map for the game to draw until a real one is painted.

Writes the three map sources into godot/map: definitions.csv, locations.png and
heightmap.png. Needs numpy and Pillow. Run from the repository root:

    python3 tools/placeholder_map.py
"""

from collections import defaultdict, deque
from pathlib import Path

import numpy as np
from PIL import Image

SEED = 7
WIDTH, HEIGHT = 2048, 1024  # Size of the location image (map units).
RELIEF_SCALE = 2  # Map units per heightmap sample.
SEA_LEVEL = 0.3  # Must match the map/sea_level config variable.
LAND_SHARE = 0.38  # Share of the map above the sea.

LAND_CELL = 24  # Spacing of land location seeds (map units).
SEA_CELL = 96  # Spacing of sea location seeds (map units).
TIER_CELLS = {"province": 64, "area": 160, "region": 400}
SEA_TIER_CELLS = {"province": 256, "area": 512, "region": 1024}
SMALL_LANDMASS = 40  # Landmasses with fewer locations join the nearest continent.

OUTPUT = Path(__file__).resolve().parent.parent / "godot" / "map"

rng = np.random.default_rng(SEED)


def periodic_noise(width, height, cells_x, octaves=6, persistence=0.5):
  """Fractal value noise in 0..1 that wraps east to west."""
  total = np.zeros((height, width))
  amplitude, norm = 1.0, 0.0
  xs = np.arange(width) / width
  ys = np.arange(height) / height

  for octave in range(octaves):
    gx = cells_x * 2**octave
    gy = max(2, gx * height // width)
    grid = rng.random((gy + 1, gx))

    fx = xs * gx
    fy = ys * gy
    x0 = np.floor(fx).astype(int)
    y0 = np.clip(np.floor(fy).astype(int), 0, gy - 1)
    tx = fx - x0
    ty = fy - y0
    tx = tx * tx * (3 - 2 * tx)
    ty = ty * ty * (3 - 2 * ty)
    x1 = (x0 + 1) % gx
    x0 = x0 % gx

    top = grid[y0][:, x0] * (1 - tx) + grid[y0][:, x1] * tx
    bottom = grid[y0 + 1][:, x0] * (1 - tx) + grid[y0 + 1][:, x1] * tx
    total += (top * (1 - ty)[:, None] + bottom * ty[:, None]) * amplitude

    norm += amplitude
    amplitude *= persistence

  return total / norm


def upsample(heights, factor):
  """Blends heights up to a finer grid the way the game samples them."""
  rows, columns = heights.shape
  xs = (np.arange(columns * factor) + 0.5) / factor - 0.5
  ys = np.clip((np.arange(rows * factor) + 0.5) / factor - 0.5, 0, rows - 1)
  x0 = np.floor(xs).astype(int)
  y0 = np.floor(ys).astype(int)
  tx, ty = xs - x0, ys - y0
  x1, y1 = (x0 + 1) % columns, np.minimum(y0 + 1, rows - 1)
  x0 %= columns

  top = heights[y0][:, x0] * (1 - tx) + heights[y0][:, x1] * tx
  bottom = heights[y1][:, x0] * (1 - tx) + heights[y1][:, x1] * tx
  return top * (1 - ty)[:, None] + bottom * ty[:, None]


def heightmap():
  width, height = WIDTH // RELIEF_SCALE, HEIGHT // RELIEF_SCALE
  shape = periodic_noise(width, height, 4, octaves=3)
  detail = periodic_noise(width, height, 8, octaves=6)
  ridges = 1 - np.abs(periodic_noise(width, height, 6, octaves=5) * 2 - 1)

  latitude = np.abs(np.linspace(-1, 1, height))[:, None]
  land = shape * 0.65 + detail * 0.35 - latitude**4 * 0.35
  land = (land - land.min()) / (land.max() - land.min())

  # A third of the map is land: lift it above the sea and raise ridges into mountain ranges.
  shore = np.quantile(land, 1 - LAND_SHARE)
  inland = np.clip((land - shore) / (1 - shore), 0, 1)
  elevation = np.where(
    land >= shore,
    SEA_LEVEL + 0.01 + inland * 0.25 + ridges**4 * np.sqrt(inland) * 0.45,
    land / shore * SEA_LEVEL * 0.98,
  )
  return np.clip(elevation, 0, 1)


def seeds(spacing):
  """One seed jittered within each cell of a grid, which wraps east to west."""
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
      nx = (cx + ox) % columns
      dx = np.abs(sx[ny, nx] - px)
      dx = np.minimum(dx, WIDTH - dx)
      dy = sy[ny, nx] - py
      distance = dx * dx + dy * dy
      closer = distance < best
      best = np.where(closer, distance, best)
      index = np.where(closer, ny * columns + nx, index)

  return index


SYLLABLES = "ar el is on ur ka ra th mo li en ve sa dor gal mir tan zel qua bry ost and ith".split()


def name(taken):
  while True:
    word = "".join(rng.choice(SYLLABLES) for _ in range(rng.integers(2, 4))).capitalize()
    if word not in taken:
      taken.add(word)
      return word


def main():
  heights = heightmap()
  is_land = upsample(heights, RELIEF_SCALE) >= SEA_LEVEL

  py, px = np.mgrid[0:HEIGHT, 0:WIDTH].astype(float) + 0.5
  warp = 6
  px_warped = (px + (periodic_noise(WIDTH, HEIGHT, 32, octaves=3) - 0.5) * warp * 2) % WIDTH
  py_warped = np.clip(py + (periodic_noise(WIDTH, HEIGHT, 32, octaves=3) - 0.5) * warp * 2, 0, HEIGHT - 0.01)

  land_x, land_y = seeds(LAND_CELL)
  sea_x, sea_y = seeds(SEA_CELL)
  land_ids = nearest_seed(px_warped, py_warped, land_x, land_y, LAND_CELL)
  sea_ids = nearest_seed(px_warped, py_warped, sea_x, sea_y, SEA_CELL) + land_x.size
  cells = np.where(is_land, land_ids, sea_ids)

  used, cells = np.unique(cells, return_inverse=True)
  cells = cells.reshape(HEIGHT, WIDTH)
  all_x = np.concatenate([land_x.ravel(), sea_x.ravel()])[used]
  all_y = np.concatenate([land_y.ravel(), sea_y.ravel()])[used]
  water = used >= land_x.size
  count = len(used)

  # Neighbours, across the seam too.
  neighbours = defaultdict(set)
  for a, b in ((cells, np.roll(cells, -1, axis=1)), (cells[:-1], cells[1:])):
    edge = a != b
    for x, y in set(zip(a[edge].tolist(), b[edge].tolist())):
      neighbours[x].add(y)
      neighbours[y].add(x)

  # Each location's mean elevation, from the pixels painted in it.
  sums = np.bincount(cells.ravel(), weights=upsample(heights, RELIEF_SCALE).ravel(), minlength=count)
  elevation = sums / np.bincount(cells.ravel(), minlength=count)
  moisture = periodic_noise(WIDTH, HEIGHT, 6, octaves=4)[np.clip(all_y.astype(int), 0, HEIGHT - 1), all_x.astype(int) % WIDTH]

  def terrain(index):
    if water[index]:
      return "sea"
    latitude = abs(all_y[index] / HEIGHT * 2 - 1)
    height = (elevation[index] - SEA_LEVEL) / (1 - SEA_LEVEL)
    if height > 0.45:
      return "mountains"
    if latitude > 0.78:
      return "tundra"
    if height > 0.25:
      return "hills"
    if latitude < 0.35 and moisture[index] < 0.42:
      return "desert"
    if moisture[index] > 0.62 and height < 0.08:
      return "marsh"
    if moisture[index] > 0.5:
      return "forest"
    return "plains"

  # Landmasses become continents; small ones join the nearest large one.
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

  large = [mass for mass, size in enumerate(sizes) if size >= SMALL_LANDMASS]
  centres = {mass: (all_x[landmass == mass].mean(), all_y[landmass == mass].mean()) for mass in large}

  def continent_of(index):
    if landmass[index] in centres:
      return landmass[index]

    def distance(mass):
      dx = abs(centres[mass][0] - all_x[index])
      return min(dx, WIDTH - dx) ** 2 + (centres[mass][1] - all_y[index]) ** 2

    return min(large, key=distance)

  # Group locations into tiers by the nearest seed of each coarser grid, so every division sits inside one parent.
  def tiers(spacings, prefix):
    chain = {}
    for tier, spacing in spacings.items():
      tx, ty = seeds(spacing)
      chain[tier] = (tx, ty, spacing)
    return chain

  land_tiers, sea_tiers = tiers(TIER_CELLS, "land"), tiers(SEA_TIER_CELLS, "sea")

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

  colors = set()
  rows = ["red,green,blue,location,province,area,region,continent,terrain"]
  palette = np.zeros((count, 3), dtype=np.uint8)

  for index in range(count):
    while True:
      color = tuple(int(c) for c in rng.integers(0, 256, 3))
      if color not in colors:
        colors.add(color)
        break
    palette[index] = color

    chain, kind = (sea_tiers, "sea") if water[index] else (land_tiers, "land")
    province, px_, py_ = division(chain, "province", all_x[index], all_y[index])
    area, ax, ay = division(chain, "area", px_, py_)
    region, rx, ry = division(chain, "region", ax, ay)

    if water[index]:
      continent = "The Deep"
      suffix = " Waters"
    else:
      # A region takes the continent most of it lies on, judged from its seed.
      key = (kind, region)
      if key not in names["region_continent"]:
        nearest = int(np.argmin(np.minimum(np.abs(all_x - rx), WIDTH - np.abs(all_x - rx)) ** 2 + (all_y - ry) ** 2 + water * 1e12))
        names["region_continent"][key] = named("continent", continent_of(nearest))
      continent = names["region_continent"][key]
      suffix = ""

    rows.append(",".join(str(part) for part in [
      *color,
      named("location", index, " Sea" if water[index] else ""),
      named("province", (kind, province), suffix),
      named("area", (kind, area), suffix),
      named("region", (kind, region), suffix),
      continent,
      terrain(index),
    ]))

  OUTPUT.mkdir(parents=True, exist_ok=True)
  (OUTPUT / "definitions.csv").write_text("\n".join(rows) + "\n")
  Image.fromarray(palette[cells]).save(OUTPUT / "locations.png", optimize=True)
  Image.fromarray((heights * 65535).astype(np.uint16)).save(OUTPUT / "heightmap.png")
  print(f"{count} locations ({int(water.sum())} sea), {len(large)} continents")


if __name__ == "__main__":
  main()
