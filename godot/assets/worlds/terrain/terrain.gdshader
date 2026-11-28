// The map's surface, drawn as an old paper map: parchment washed with the map mode's colours and inked with coasts,
// borders and the glyphs of each terrain.
// MESH: TerrainView chunks; the view sets every sampler and the map's shape.

shader_type spatial;
render_mode unshaded;

#include "res://assets/worlds/terrain/noise.gdshaderinc"
#include "res://assets/worlds/terrain/glyphs.gdshaderinc"

const int LOOKUP_WIDTH = 256;

// The terrains, as the view's terrain lookup numbers them.
const int TERRAIN_PLAINS = 0;
const int TERRAIN_FOREST = 1;
const int TERRAIN_HILLS = 2;
const int TERRAIN_MOUNTAINS = 3;
const int TERRAIN_DESERT = 4;
const int TERRAIN_MARSH = 5;
const int TERRAIN_TUNDRA = 6;
const int TERRAIN_SEA = 7;

// A border between two regions, as the borders texture ranks it; continents and shores rank higher.
const float REGION_RANK = 0.8;
// How many pixels across a glyph must be to be drawn at all, and to be drawn at full strength.
const vec2 GLYPH_LEGIBLE = vec2(6.0, 10.0);
const int SHORE_LINES = 3;

group_uniforms map;
uniform sampler2D heights_tex : filter_linear, repeat_disable;
uniform sampler2D locations_tex : filter_nearest;
uniform sampler2D borders_tex : filter_nearest;
uniform sampler2D colors_tex : source_color, filter_nearest;
/** The terrain of each location, laid out like the colours. */
uniform sampler2D terrains_tex : filter_nearest;
/** Size of the map (map units). */
uniform vec2 map_size = vec2(2048.0, 1024.0);
/** Height of the sea on the heightmap (0..1). */
uniform float sea_level : hint_range(0.0, 1.0) = 0.3;
/** Height of the tallest peak above the sea (map units). */
uniform float relief : hint_range(0.0, 500.0) = 40.0;
/** How far the chunk skirts hang (map units). */
uniform float skirt_depth : hint_range(0.0, 50.0) = 4.0;
uniform mat4 terrain_from_world = mat4(1.0);

group_uniforms highlight;
/** Per location: red while hovered, green while selected. */
uniform sampler2D highlights_tex : filter_nearest;
/** Whether anything is selected, so the rest of the map dims. */
uniform bool has_selection = false;
/** How much the map outside the selection darkens (0..1). */
uniform float unselected_dimming : hint_range(0.0, 1.0) = 0.22;
uniform vec4 hover_color : source_color = vec4(1.0, 0.97, 0.88, 0.35);
uniform vec4 selected_color : source_color = vec4(0.55, 0.13, 0.08, 0.14);

group_uniforms paper;
uniform vec4 paper_color : source_color = vec4(0.87, 0.79, 0.62, 1.0);
/** The colour the paper has aged to in its stains. */
uniform vec4 stain_color : source_color = vec4(0.71, 0.59, 0.4, 1.0);
/** The colour the paper darkens to at its edges. */
uniform vec4 scorch_color : source_color = vec4(0.3, 0.2, 0.12, 1.0);
/** How strongly the paper is stained (0..1). */
uniform float stain_strength : hint_range(0.0, 1.0) = 0.55;
/** How far across a stain is (map units). */
uniform float stain_size : hint_range(10.0, 1000.0) = 240.0;
/** How far in from the map's edges the paper darkens (map units). */
uniform float scorch_reach : hint_range(0.0, 400.0) = 70.0;
/** How far in from the map's edges its ruled frame lies (map units). */
uniform float frame_inset : hint_range(0.0, 100.0) = 14.0;

group_uniforms washes;
/** How strongly the map mode's colours tint the paper (0..1). */
uniform float fill_strength : hint_range(0.0, 1.0) = 0.6;
/** How much of its own colour a fill keeps; the rest fades towards the paper (0..1). */
uniform float fill_vividness : hint_range(0.0, 1.0) = 0.6;
/** The tint the sea lays over the paper. */
uniform vec4 sea_color : source_color = vec4(0.74, 0.8, 0.78, 1.0);
/** How strongly slopes facing away from the north-west are shaded (0..1). */
uniform float shading_strength : hint_range(0.0, 1.0) = 0.22;

group_uniforms ink;
uniform vec4 ink_color : source_color = vec4(0.17, 0.11, 0.07, 1.0);
/** The ink borders between regions are ruled in. */
uniform vec4 border_color : source_color = vec4(0.42, 0.13, 0.08, 1.0);
/** Width of the coastline (map units). */
uniform float coast_width : hint_range(0.0, 4.0) = 0.45;
/** Width of the boldest border (map units). */
uniform float border_width : hint_range(0.0, 2.0) = 0.4;
/** Length of one dash and its gap along a border (map units). */
uniform float border_dash : hint_range(0.5, 20.0) = 3.0;
/** How much fainter borders are drawn across water (0..1). */
uniform float sea_border_strength : hint_range(0.0, 1.0) = 0.25;
/** Depth of the seabed between one line drawn along the shore and the next (0..1 of the heightmap). */
uniform float shore_line_depth : hint_range(0.001, 0.1) = 0.0125;

group_uniforms glyphs;
/** Distance between the glyphs drawn when the map is seen close (map units). */
uniform float glyph_spacing : hint_range(2.0, 40.0) = 7.0;
/** Distance between the glyphs drawn when the map is seen from afar (map units). */
uniform float far_glyph_spacing : hint_range(4.0, 120.0) = 20.0;
/** Width of a glyph's pen strokes, as a share of the glyph's height. */
uniform float glyph_pen : hint_range(0.01, 0.2) = 0.045;
/** How dark the glyphs are inked (0..1). */
uniform float glyph_strength : hint_range(0.0, 1.0) = 0.8;

varying vec2 map_position;

vec2 height_uv(vec2 position) {
	vec2 half_texel = 0.5 / vec2(textureSize(heights_tex, 0));
	return clamp(position / map_size, half_texel, 1.0 - half_texel);
}

float height_at(vec2 position) {
	return texture(heights_tex, height_uv(position)).r;
}

// Height of the drawn surface: the ground on land, the sea's surface over water.
float surface_at(vec2 position) {
	return max(height_at(position) - sea_level, 0.0) * relief;
}

ivec2 cell_at(vec2 cell) {
	ivec2 size = textureSize(locations_tex, 0);
	return clamp(ivec2(cell), ivec2(0), size - 1);
}

int location_in(ivec2 cell) {
	vec2 id = texelFetch(locations_tex, cell, 0).rg * 255.0 + 0.5;
	return int(id.r) + int(id.g) * 256;
}

ivec2 lookup_texel(int location) {
	return ivec2(location % LOOKUP_WIDTH, location / LOOKUP_WIDTH);
}

int terrain_in(ivec2 cell) {
	return int(texelFetch(terrains_tex, lookup_texel(location_in(cell)), 0).r * 255.0 + 0.5);
}

bool is_water(int terrain) {
	return terrain >= TERRAIN_SEA;
}

// A per-location lookup blended between the four nearest cells, so fills and highlights meet softly.
vec4 blended_lookup(sampler2D lookup, vec2 corner, vec2 blend) {
	vec4 north_west = texelFetch(lookup, lookup_texel(location_in(cell_at(corner))), 0);
	vec4 north_east = texelFetch(lookup, lookup_texel(location_in(cell_at(corner + vec2(1.0, 0.0)))), 0);
	vec4 south_west = texelFetch(lookup, lookup_texel(location_in(cell_at(corner + vec2(0.0, 1.0)))), 0);
	vec4 south_east = texelFetch(lookup, lookup_texel(location_in(cell_at(corner + vec2(1.0, 1.0)))), 0);
	return mix(mix(north_west, north_east, blend.x), mix(south_west, south_east, blend.x), blend.y);
}

// The bare paper: its colour, mottled with stains and flecked with grain.
vec3 paper_at(vec2 position, float pixel) {
	float stains = cloud_noise(position / stain_size);
	float mottle = cloud_noise(position / (stain_size * 0.14) + 71.0);
	float grain = value_noise(position * 1.9) * (1.0 - smoothstep(0.2, 0.8, pixel));

	float aged = smoothstep(0.3, 0.8, stains) * 0.7 + (mottle - 0.5) * 0.5;
	vec3 paper = mix(paper_color.rgb, stain_color.rgb, clamp(aged, 0.0, 1.0) * stain_strength);
	return paper * (1.0 - grain * 0.06);
}

// The map mode's fill as a wash of watercolour over what lies beneath.
vec3 washed(vec3 beneath, vec4 fill) {
	vec3 pigment = mix(paper_color.rgb, fill.rgb, fill_vividness);
	return mix(beneath, beneath * pigment / paper_color.rgb, fill.a * fill_strength);
}

// How much the ground at a point is shaded for sloping away from light out of the north-west (0..1).
float slope_shade(vec2 position) {
	vec2 reach = map_size / vec2(textureSize(heights_tex, 0));
	float east = height_at(position + vec2(reach.x, 0.0)) - height_at(position - vec2(reach.x, 0.0));
	float south = height_at(position + vec2(0.0, reach.y)) - height_at(position - vec2(0.0, reach.y));
	return clamp(-(east + south) * 14.0, 0.0, 1.0);
}

// The broken lines drawn alongside the shore, each fainter than the last (0..1).
float shore_lines(vec2 position, float depth) {
	float spread = fwidth(depth);
	float lines = 0.0;

	for (int line = 1; line <= SHORE_LINES; line++) {
		float distance = abs(depth - float(line) * shore_line_depth) / max(spread, 1e-6);
		float broken = smoothstep(0.3, 0.45, value_noise(position * 0.11 + float(line) * 23.0));
		lines = max(lines, (1.0 - smoothstep(0.4, 1.3, distance)) * broken / float(line));
	}

	// Seen from afar the lines would run together, so they fade.
	return lines * (1.0 - smoothstep(0.2, 0.45, spread / shore_line_depth));
}

// The line of the coast, where the ground meets the sea (0..1).
float coast_line(float height, float pixel) {
	float distance = abs(height - sea_level) / max(fwidth(height), 1e-6);
	float half_width = max(coast_width / pixel, 1.4) * 0.5;
	return 1.0 - smoothstep(half_width - 0.5, half_width + 0.5, distance);
}

// How strongly one side of a cell is ruled as a border (0..1): bolder for higher ranks, dashed below a region's,
// faint across water and missing along a shore, which the coast line draws.
float border_side(ivec2 here, ivec2 there, float rank, float across, float along, float pixel) {
	if (rank < 0.01) {
		return 0.0;
	}

	bool wet_here = is_water(terrain_in(here));
	bool wet_there = is_water(terrain_in(there));
	if (wet_here != wet_there) {
		return 0.0;
	}

	float line = 1.0 - smoothstep(0.0, max(border_width * rank, pixel), across);
	float dash = rank < REGION_RANK ? step(0.4, fract(along / (border_dash * rank * 2.0))) : 1.0;
	// Lesser borders crowd the map from afar, so they fade out first.
	float legible = 1.0 - smoothstep(0.5, 1.5, pixel / (rank * rank * 3.0));
	return line * dash * legible * rank * (wet_here ? sea_border_strength : 1.0);
}

// The border ink at a point, and in y whether it is a region's border or greater.
vec2 border_ink(vec2 corner, vec2 blend, float pixel) {
	float row = blend.y < 0.5 ? 0.0 : 1.0;
	float column = blend.x < 0.5 ? 0.0 : 1.0;
	ivec2 west = cell_at(corner + vec2(0.0, row));
	ivec2 north = cell_at(corner + vec2(column, 0.0));
	ivec2 east = cell_at(corner + vec2(1.0, row));
	ivec2 south = cell_at(corner + vec2(column, 1.0));

	float east_rank = location_in(west) != location_in(east) ? texelFetch(borders_tex, west, 0).r : 0.0;
	float south_rank = location_in(north) != location_in(south) ? texelFetch(borders_tex, north, 0).g : 0.0;

	float east_line = border_side(west, east, east_rank, abs(blend.x - 0.5), map_position.y, pixel);
	float south_line = border_side(north, south, south_rank, abs(blend.y - 0.5), map_position.x, pixel);
	float rank = east_line > south_line ? east_rank : south_rank;
	return vec2(max(east_line, south_line), step(REGION_RANK - 0.01, rank));
}

// One glyph, standing at `foot`, for the terrain found there.
vec2 glyph_at(vec2 position, vec2 foot, float size, vec2 chance, float pixel) {
	int terrain = terrain_in(cell_at(foot));
	vec2 point = (position - foot) / size;
	float soft = pixel / size;
	float half_width = max(glyph_pen * 0.5, soft * 0.6);
	float luck = fract(chance.x * 7.31 + chance.y * 3.17);

	if (terrain == TERRAIN_MOUNTAINS) {
		return peak_glyph(point / 1.1, chance.x - 0.5, half_width / 1.1, soft / 1.1);
	}
	if (terrain == TERRAIN_HILLS && luck < 0.55) {
		return hill_glyph(point / 0.8, half_width / 0.8, soft / 0.8);
	}
	if (terrain == TERRAIN_FOREST) {
		return tree_glyph(point, half_width, soft);
	}
	if (terrain == TERRAIN_MARSH && luck < 0.6) {
		return reed_glyph(point, half_width, soft);
	}
	if (terrain == TERRAIN_DESERT && luck < 0.8) {
		return stipple_glyph(point, chance, half_width, soft);
	}
	if (terrain == TERRAIN_TUNDRA && luck < 0.3) {
		return stipple_glyph(point, chance, half_width, soft);
	}
	return vec2(0.0);
}

// The ink of a field of glyphs set `spacing` apart, each drawn over those north of it (0..1).
float glyph_field(vec2 position, float spacing, float pixel) {
	float legible = smoothstep(GLYPH_LEGIBLE.x, GLYPH_LEGIBLE.y, spacing / pixel);
	if (legible <= 0.0) {
		return 0.0;
	}

	vec2 scaled = position / spacing;
	float ink = 0.0;

	// A glyph stands no taller than a cell and its foot lies in the cell's southern part, so only its own row and the
	// one south can reach here.
	for (int step = 0; step <= 1; step++) {
		float row = floor(scaled.y) + float(step);
		// Every other row is set half a cell along, so the glyphs do not line up in columns.
		float shift = mod(row, 2.0) * 0.5;
		float home = floor(scaled.x - shift);
		vec2 row_glyphs = vec2(0.0);

		for (int column = -1; column <= 1; column++) {
			vec2 cell = vec2(home + float(column), row);
			vec2 chance = hash2(cell + spacing);
			vec2 foot = (cell + vec2(shift + 0.25 + chance.x * 0.5, 0.45 + chance.y * 0.53)) * spacing;
			float size = spacing * mix(0.6, 0.75, hash(cell + 7.0));

			row_glyphs = max(row_glyphs, glyph_at(position, foot, size, chance, pixel));
		}

		ink = max(ink * (1.0 - row_glyphs.y), row_glyphs.x);
	}

	return ink * legible;
}

// The glyphs of the land: small ones seen close, giving way to larger ones from afar (0..1).
float glyph_ink(vec2 position, float pixel) {
	float near = glyph_field(position, glyph_spacing, pixel);
	float far = glyph_field(position, far_glyph_spacing, pixel);
	float near_legible = smoothstep(GLYPH_LEGIBLE.x, GLYPH_LEGIBLE.y, glyph_spacing / pixel);
	return mix(far, near, near_legible);
}

// The frame ruled just inside the map's edges: a bold line with a fine one within (0..1).
float frame_lines(float from_edge, float pixel) {
	float soft = max(pixel, 0.05);
	float bold = 1.0 - smoothstep(0.9, 0.9 + soft, abs(from_edge - frame_inset));
	float fine = 1.0 - smoothstep(0.25, 0.25 + soft, abs(from_edge - frame_inset - 3.5));
	return max(bold, fine);
}

void vertex() {
	vec4 world = MODEL_MATRIX * vec4(VERTEX, 1.0);
	map_position = (terrain_from_world * world).xz;

	VERTEX.y = surface_at(map_position) + VERTEX.y * skirt_depth;
}

void fragment() {
	vec2 cell = map_position - 0.5;
	vec2 corner = floor(cell);
	vec2 blend = cell - corner;
	// How much of the map one pixel covers here (map units).
	float pixel = max(length(fwidth(map_position)) * 0.7, 1e-4);

	float height = height_at(map_position);
	float land = smoothstep(-0.5, 0.5, (height - sea_level) / max(fwidth(height), 1e-6));

	// Paper and washes.
	vec3 paper = paper_at(map_position, pixel);
	vec3 color = mix(paper * sea_color.rgb / paper_color.rgb, paper, land);
	color = washed(color, blended_lookup(colors_tex, corner, blend));
	color *= 1.0 - slope_shade(map_position) * shading_strength * land;

	// Selection.
	vec2 lit = blended_lookup(highlights_tex, corner, blend).rg;
	color = mix(color, hover_color.rgb, hover_color.a * lit.r);
	color = mix(color, selected_color.rgb, selected_color.a * lit.g);
	if (has_selection) {
		color *= mix(1.0 - unselected_dimming, 1.0, lit.g);
	}

	// Ink.
	float ink = glyph_ink(map_position, pixel) * glyph_strength * land;
	ink = max(ink, shore_lines(map_position, sea_level - height) * 0.55 * (1.0 - land));
	ink = max(ink, coast_line(height, pixel));
	color = mix(color, ink_color.rgb, ink * ink_color.a);

	vec2 border = border_ink(corner, blend, pixel);
	color = mix(color, mix(ink_color.rgb, border_color.rgb, border.y), border.x * 0.85);
	// The blended highlight passes through one half at the selection's edge, which rules its outline.
	color = mix(color, selected_color.rgb, 1.0 - abs(lit.g * 2.0 - 1.0));

	// Edges.
	vec2 from_edges = min(map_position, map_size - map_position);
	float from_edge = min(from_edges.x, from_edges.y);
	color = mix(color, ink_color.rgb, frame_lines(from_edge, pixel));
	float ragged = from_edge + (cloud_noise(map_position / 30.0) - 0.5) * scorch_reach * 0.6;
	color = mix(color, color * scorch_color.rgb, (1.0 - smoothstep(0.0, scorch_reach, ragged)) * 0.8);

	ALBEDO = color;
}
