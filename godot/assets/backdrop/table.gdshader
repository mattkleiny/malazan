// The table the map lies on: dark boards lit in a pool around the map and lost to shadow beyond it.
// MESH: a flat plane far larger than the map, in the map's own space with the map's north-west corner at the origin.

shader_type spatial;
render_mode unshaded;

#include "res://assets/worlds/terrain/noise.gdshaderinc"

group_uniforms map;
/** Size of the map lying on the table (map units). */
uniform vec2 map_size = vec2(1536.0, 2048.0);
/** How far in from the map's edges the table is left undrawn, so it never shows through the map (map units). */
uniform float hidden_inset : hint_range(0.0, 100.0) = 6.0;

group_uniforms boards;
uniform vec4 wood_color : source_color = vec4(0.33, 0.2, 0.11, 1.0);
/** The colour of the darkest boards and of the grain. */
uniform vec4 grain_color : source_color = vec4(0.15, 0.085, 0.045, 1.0);
/** Width of a board (map units). */
uniform float board_width : hint_range(20.0, 1000.0) = 230.0;
/** Length of a board (map units). */
uniform float board_length : hint_range(100.0, 8000.0) = 1900.0;
/** Width of the gap between boards (map units). */
uniform float gap_width : hint_range(0.0, 20.0) = 3.0;
/** How strongly the grain marks the wood (0..1). */
uniform float grain_strength : hint_range(0.0, 1.0) = 0.55;

group_uniforms light;
/** The darkness the table falls into away from the map. */
uniform vec4 dark_color : source_color = vec4(0.075, 0.06, 0.05, 1.0);
/** How far from the map's edges the light reaches (map units). */
uniform float light_reach : hint_range(100.0, 5000.0) = 1700.0;
/** How far the map's shadow spreads over the table (map units). */
uniform float shadow_reach : hint_range(0.0, 200.0) = 38.0;
/** Where the map's shadow falls, away from the light (map units). */
uniform vec2 shadow_offset = vec2(9.0, 12.0);
/** How dark the map's shadow is (0..1). */
uniform float shadow_strength : hint_range(0.0, 1.0) = 0.65;

varying vec2 table_position;

// How far a point lies outside the map's edges, moved by an offset (map units).
float outside_map(vec2 position, vec2 offset) {
	vec2 half_size = map_size * 0.5;
	return length(max(abs(position - offset - half_size) - half_size, 0.0));
}

// The wood at a point: boards laid east to west, each its own shade, with grain along them and gaps between.
vec3 boards_at(vec2 position, float pixel) {
	float row = floor(position.y / board_width);
	float along = position.x + hash(vec2(row, 3.0)) * board_length;
	vec2 board = vec2(floor(along / board_length), row);

	// Grain: long streaks, and rings that wander across the board.
	vec2 streak_point = vec2(along / 160.0, position.y / 7.0) + board * 31.0;
	float wander = cloud_noise(vec2(along / 420.0, position.y / 90.0) + board * 13.0);
	float rings = 0.5 + 0.5 * sin((position.y + wander * 120.0) / 5.5);
	float grain = mix(cloud_noise(streak_point), rings, 0.35) * (1.0 - smoothstep(1.0, 4.0, pixel));

	float shade = hash(board + 0.5) * 0.45 + grain * grain_strength;
	vec3 wood = mix(wood_color.rgb, grain_color.rgb, clamp(shade, 0.0, 1.0));

	// Gaps along and across the boards.
	float from_side = min(fract(position.y / board_width), 1.0 - fract(position.y / board_width)) * board_width;
	float from_end = min(fract(along / board_length), 1.0 - fract(along / board_length)) * board_length;
	float gap = 1.0 - smoothstep(gap_width * 0.5, gap_width * 0.5 + max(pixel, 0.5), min(from_side, from_end));
	return mix(wood, grain_color.rgb * 0.35, gap);
}

void vertex() {
	table_position = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xz;
}

void fragment() {
	vec2 inside_map = min(table_position, map_size - table_position);
	if (min(inside_map.x, inside_map.y) > hidden_inset) {
		discard;
	}

	// How much of the table one pixel covers here (map units).
	float pixel = length(fwidth(table_position)) * 0.7;

	vec3 color = boards_at(table_position, pixel);
	color *= 1.0 - shadow_strength * (1.0 - smoothstep(0.0, shadow_reach, outside_map(table_position, shadow_offset)));

	float lit = 1.0 - smoothstep(0.0, light_reach, outside_map(table_position, vec2(0.0)));
	ALBEDO = mix(dark_color.rgb, color, lit * lit);
}
