# Game

This is the project's ubiquitous language: one line per term, grouped by section. Each section links to its full entries in `docs/context/` once it has them.

## Scales

- **Fixed Tick**: One deterministic step of the simulation. _Avoid_: frame, update
- **Variable Tick**: One presentation step, taken once per rendered frame.
- **Simulation Clock**: The clock the simulation keeps for itself, handing out fixed ticks as time passes. _Avoid_: timer, physics rate

## Map

- **Map**: The world the game is played on: its locations, the tiers above them, and the lie of its land. _Avoid_: world, level
- **Location**: The smallest piece the map is divided into; every point of the map lies in exactly one. _Avoid_: tile, cell, province (that is the tier above)
- **Province**: A group of neighbouring locations.
- **Area**: A group of provinces.
- **Region**: A group of areas.
- **Continent**: The largest division of the map, a group of regions.
- **Terrain**: The kind of ground or water a location is made of, such as plains, mountains or sea. _Avoid_: biome, tile type
- **Key Colour**: The unique colour a location is painted in on the map's location image. _Avoid_: province colour, id colour
- **Sea Level**: The height at which the sea lies; ground below it is under water.
- **Neighbour**: A location sharing a border with another. _Avoid_: adjacent tile
- **Border**: The line where two locations meet, ranked by the highest tier it divides; a shore ranks highest. _Avoid_: edge, outline
- **Map Mode**: One way of colouring the map's locations, such as by terrain or by province, which also decides what picking the map selects; one is chosen at a time. _Avoid_: overlay, view, filter
- **Fill**: The colour a map mode gives a location, and how strongly it covers the ground. _Avoid_: tint, overlay colour
- **Map Mode Preview**: A map mode shown over the chosen one for a while, such as while the cursor rests on a button. _Avoid_: temporary mode, hover mode
- **Map Mode Fade**: The short blend from one map mode's fills to the next. _Avoid_: transition, tween
- **Map Mode Bar**: The row of map mode icons along the bottom of the screen; resting on one previews its mode, pressing it chooses it. _Avoid_: toolbar, mode picker
- **Pick**: The point of the map, and its location, under the cursor. _Avoid_: raycast, hit test
- **Selection**: The thing picking the map chooses in the shown map mode: a location, a whole province or region, or something a map mode offers, such as the wealth of an area. It is lit while hovered or chosen, and described to the player. _Avoid_: target, focus

## Map editing

- **Map Editor**: The editor tool for changing the map in place: painting locations, shaping the ground and editing the definitions. _Avoid_: level editor, map tool
- **Definitions**: The table naming every location, the divisions it sits in, its terrain and its key colour. _Avoid_: province table, map data
- **Brush**: The circle of ground a map editor tool acts on, with a radius and a strength. _Avoid_: cursor, pen
- **Stroke**: One press-drag-release of a brush; it is undone and redone as one. _Avoid_: drag, paint operation
- **Sculpt**: Raising, lowering, smoothing or flattening the ground with a brush. _Avoid_: terraform, deform

## Knowledge

- **Knowledge Base**: Everything the game can explain about itself to the player, each piece reachable from any other that mentions it. _Avoid_: encyclopedia, wiki, help
- **Concept**: One idea of the game the player can look up, with a name and a description. _Avoid_: entry, article, glossary term
- **Knowledge Link**: A mention of a piece of knowledge inside text, naming its kind and which one; resting the cursor on it opens its tooltip. _Avoid_: hyperlink, reference, keyword
- **Knowledge Markup**: Text written with knowledge links in it. _Avoid_: rich text, tagged text
- **Tooltip**: The panel describing a piece of knowledge, opened beside the cursor from a knowledge link; a link inside one opens another on top of it. _Avoid_: popup, hint, hover card
- **Lock**: A tooltip staying open after the cursor leaves its link, so the cursor can move onto it; it happens once the tooltip has been open a short while. _Avoid_: pin, freeze
