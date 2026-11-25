// The map's surface: relief from the heightmap, filled with the map mode's colours and ruled with borders.
// MESH: TerrainView chunks; the view sets every sampler and the map's shape.

shader_type spatial;

const float SNOW_LINE = 0.75;
const int LOOKUP_WIDTH = 256;

group_uniforms map;
uniform sampler2D heights_tex : filter_linear, repeat_disable;
uniform sampler2D locations_tex : filter_nearest;
uniform sampler2D borders_tex : filter_nearest;
uniform sampler2D colors_tex : source_color, filter_nearest;
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
uniform float unselected_dimming : hint_range(0.0, 1.0) = 0.35;
uniform vec4 hover_color : source_color = vec4(1.0, 1.0, 1.0, 0.25);
uniform vec4 selected_color : source_color = vec4(1.0, 0.85, 0.3, 0.2);

group_uniforms look;
/** How strongly the map mode's colours cover the ground (0..1). */
uniform float fill_strength : hint_range(0.0, 1.0) = 0.65;
uniform vec4 lowland_color : source_color = vec4(0.42, 0.52, 0.3, 1.0);
uniform vec4 highland_color : source_color = vec4(0.5, 0.45, 0.38, 1.0);
uniform vec4 snow_color : source_color = vec4(0.92, 0.94, 0.96, 1.0);
uniform vec4 shallow_color : source_color = vec4(0.16, 0.38, 0.5, 1.0);
uniform vec4 deep_color : source_color = vec4(0.04, 0.12, 0.25, 1.0);
uniform vec4 border_color : source_color = vec4(0.08, 0.06, 0.05, 1.0);
/** Width of the boldest border (map units). */
uniform float border_width : hint_range(0.0, 2.0) = 0.35;

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

// A per-location lookup blended between the four nearest cells, so fills and highlights meet softly.
vec4 blended_lookup(sampler2D lookup, vec2 corner, vec2 blend) {
	vec4 north_west = texelFetch(lookup, lookup_texel(location_in(cell_at(corner))), 0);
	vec4 north_east = texelFetch(lookup, lookup_texel(location_in(cell_at(corner + vec2(1.0, 0.0)))), 0);
	vec4 south_west = texelFetch(lookup, lookup_texel(location_in(cell_at(corner + vec2(0.0, 1.0)))), 0);
	vec4 south_east = texelFetch(lookup, lookup_texel(location_in(cell_at(corner + vec2(1.0, 1.0)))), 0);
	return mix(mix(north_west, north_east, blend.x), mix(south_west, south_east, blend.x), blend.y);
}

// How strongly a border line covers this point (0..1).
float border_mask(vec2 corner, vec2 blend) {
	float row = blend.y < 0.5 ? 0.0 : 1.0;
	float column = blend.x < 0.5 ? 0.0 : 1.0;
	ivec2 west = cell_at(corner + vec2(0.0, row));
	ivec2 north = cell_at(corner + vec2(column, 0.0));

	float east_rank = location_in(west) != location_in(cell_at(corner + vec2(1.0, row))) ? texelFetch(borders_tex, west, 0).r : 0.0;
	float south_rank = location_in(north) != location_in(cell_at(corner + vec2(column, 1.0))) ? texelFetch(borders_tex, north, 0).g : 0.0;

	vec2 pixel = fwidth(map_position);
	float east_line = 1.0 - smoothstep(0.0, max(border_width * east_rank, pixel.x), abs(blend.x - 0.5));
	float south_line = 1.0 - smoothstep(0.0, max(border_width * south_rank, pixel.y), abs(blend.y - 0.5));

	return max(east_line * step(0.01, east_rank) * east_rank, south_line * step(0.01, south_rank) * south_rank);
}

vec3 ground_color(float height) {
	float above_sea = clamp((height - sea_level) / (1.0 - sea_level), 0.0, 1.0);
	vec3 ground = mix(lowland_color.rgb, highland_color.rgb, smoothstep(0.1, 0.55, above_sea));
	return mix(ground, snow_color.rgb, smoothstep(SNOW_LINE - 0.05, SNOW_LINE + 0.05, above_sea));
}

void vertex() {
	vec4 world = MODEL_MATRIX * vec4(VERTEX, 1.0);
	map_position = (terrain_from_world * world).xz;

	VERTEX.y = surface_at(map_position) + VERTEX.y * skirt_depth;

	vec2 step_size = map_size / vec2(textureSize(heights_tex, 0));
	float west = surface_at(map_position - vec2(step_size.x, 0.0));
	float east = surface_at(map_position + vec2(step_size.x, 0.0));
	float north = surface_at(map_position - vec2(0.0, step_size.y));
	float south = surface_at(map_position + vec2(0.0, step_size.y));
	NORMAL = normalize(vec3(west - east, 2.0 * step_size.x, north - south));
}

void fragment() {
	vec2 cell = map_position - 0.5;
	vec2 corner = floor(cell);
	vec2 blend = cell - corner;

	float height = height_at(map_position);
	bool is_water = height < sea_level;

	vec3 color = is_water
		? mix(shallow_color.rgb, deep_color.rgb, smoothstep(0.0, 0.2, sea_level - height))
		: ground_color(height);

	vec4 fill = blended_lookup(colors_tex, corner, blend);
	color = mix(color, fill.rgb, fill.a * fill_strength);

	vec2 lit = blended_lookup(highlights_tex, corner, blend).rg;
	color = mix(color, hover_color.rgb, hover_color.a * lit.r);
	color = mix(color, selected_color.rgb, selected_color.a * lit.g);
	if (has_selection) {
		color *= mix(1.0 - unselected_dimming, 1.0, lit.g);
	}
	// The blended highlight passes through one half at the selection's edge, which rules its outline.
	color = mix(color, selected_color.rgb, 1.0 - abs(lit.g * 2.0 - 1.0));

	color = mix(color, border_color.rgb, border_mask(corner, blend));

	ALBEDO = color;
	ROUGHNESS = is_water ? 0.35 : 0.9;
	SPECULAR = is_water ? 0.5 : 0.2;
}
