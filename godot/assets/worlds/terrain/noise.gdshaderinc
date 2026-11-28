// Noise: repeatable pseudo-random values over the plane.

float hash(vec2 point) {
	vec3 mixed = fract(vec3(point.xyx) * 0.1031);
	mixed += dot(mixed, mixed.yzx + 33.33);
	return fract((mixed.x + mixed.y) * mixed.z);
}

vec2 hash2(vec2 point) {
	vec3 mixed = fract(vec3(point.xyx) * vec3(0.1031, 0.1030, 0.0973));
	mixed += dot(mixed, mixed.yzx + 33.33);
	return fract((mixed.xx + mixed.yz) * mixed.zy);
}

// Smooth noise in 0..1, one feature per unit.
float value_noise(vec2 point) {
	vec2 cell = floor(point);
	vec2 blend = fract(point);
	blend = blend * blend * (3.0 - 2.0 * blend);

	float north = mix(hash(cell), hash(cell + vec2(1.0, 0.0)), blend.x);
	float south = mix(hash(cell + vec2(0.0, 1.0)), hash(cell + vec2(1.0, 1.0)), blend.x);
	return mix(north, south, blend.y);
}

// Three octaves of smooth noise in 0..1.
float cloud_noise(vec2 point) {
	return value_noise(point) * 0.57 + value_noise(point * 2.03 + 17.0) * 0.29 + value_noise(point * 4.01 + 41.0) * 0.14;
}
