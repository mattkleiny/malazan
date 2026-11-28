// Glyphs: the small pen drawings a paper map marks its ground with.
// Each is drawn in its own space: the foot at the origin, one unit tall, x east and y south.
// Each returns its ink in x and, in y, the ground it hides of whatever was drawn behind it.

float segment_distance(vec2 point, vec2 from, vec2 to) {
	vec2 along = to - from;
	float share = clamp(dot(point - from, along) / dot(along, along), 0.0, 1.0);
	return length(point - from - along * share);
}

// How much ink a pen of the given half width leaves at a distance from its line; `soft` is the blur of one pixel.
float pen(float distance, float half_width, float soft) {
	return 1.0 - smoothstep(half_width - soft, half_width + soft, distance);
}

// A mountain: two slopes meeting at a peak, hatched on its eastern face.
vec2 peak_glyph(vec2 point, float lean, float half_width, float soft) {
	vec2 west = vec2(-0.58, 0.0);
	vec2 east = vec2(0.58, 0.0);
	vec2 top = vec2(lean * 0.3, -1.0);

	float slopes = min(segment_distance(point, west, top), segment_distance(point, top, east));

	// Inside the triangle: east of the western slope, west of the eastern one, above the foot.
	vec2 rise = top - west;
	vec2 fall = east - top;
	float within_west = rise.x * (point.y - west.y) - rise.y * (point.x - west.x);
	float within_east = fall.x * (point.y - top.y) - fall.y * (point.x - top.x);
	float inside = step(0.0, within_west) * step(0.0, within_east) * step(point.y, 0.0);

	// Strokes running down the eastern face, parallel to the western slope.
	vec2 spur = vec2(top.x + 0.22, 0.0) - top;
	float shadow = step(0.0, -(spur.x * (point.y - top.y) - spur.y * (point.x - top.x)));
	float across = (point.x + point.y * 0.72) * 4.5;
	float hatch = pen(abs(fract(across) - 0.5) / 4.5, half_width * 0.8, soft) * inside * shadow;

	return vec2(max(pen(slopes, half_width, soft), hatch * 0.85), inside);
}

// A hill: a low round back with a stroke of shade under its eastern shoulder.
vec2 hill_glyph(vec2 point, float half_width, float soft) {
	vec2 reach = vec2(0.75, 0.55);
	float rim = length(point / reach);
	float back = abs(rim - 1.0) * min(reach.x, reach.y) + step(0.0, point.y) * 4.0;
	float shade = segment_distance(point, vec2(0.2, -0.3), vec2(0.5, -0.06));

	return vec2(max(pen(back, half_width, soft), pen(shade, half_width * 0.8, soft) * 0.8), step(rim, 1.0) * step(point.y, 0.0));
}

// A tree: a round crown on a short trunk.
vec2 tree_glyph(vec2 point, float half_width, float soft) {
	vec2 crown = vec2(0.0, -0.62);
	float radius = 0.38;
	float reach = length(point - crown);
	float trunk = segment_distance(point, vec2(0.0, -0.24), vec2(0.0, 0.0));
	float shade = segment_distance(point, crown + vec2(0.14, -0.12), crown + vec2(0.2, 0.12));

	float ink = max(pen(abs(reach - radius), half_width, soft), pen(trunk, half_width, soft));
	return vec2(max(ink, pen(shade, half_width * 0.8, soft) * 0.8), step(reach, radius));
}

// A tuft of reeds standing in water.
vec2 reed_glyph(vec2 point, float half_width, float soft) {
	float water = segment_distance(point, vec2(-0.55, 0.0), vec2(0.55, 0.0));
	float blades = min(
		segment_distance(point, vec2(0.0, -0.04), vec2(0.0, -0.5)),
		min(segment_distance(point, vec2(-0.2, -0.04), vec2(-0.34, -0.38)), segment_distance(point, vec2(0.2, -0.04), vec2(0.34, -0.38))));

	return vec2(max(pen(water, half_width, soft), pen(blades, half_width, soft)), 0.0);
}

// A scatter of dots, for sand or frozen ground; `scatter` (0..1 each way) moves them about.
vec2 stipple_glyph(vec2 point, vec2 scatter, float half_width, float soft) {
	float dots = min(
		length(point - vec2(-0.35 + scatter.x * 0.3, -0.25)),
		min(length(point - vec2(0.3, -0.55 + scatter.y * 0.3)), length(point - vec2(scatter.y * 0.4 - 0.1, -0.85 + scatter.x * 0.2))));

	return vec2(pen(dots, half_width * 1.3, soft), 0.0);
}
