#version 330 core

uniform isampler2D anim;
uniform mat4 eye;
uniform float frame; // interpolated
layout(location=0) in uint aIndex;
layout(location=1) in vec2 aUv;
out vec3 position;
out vec3 normal;
out vec2 uv;

const float MD3_XYZ_SCALE = 0.015625; //1./64
const float BYTE_TAU = 40.58451048843331062106; //255./(2.*3.1415926535897932384626434)

vec3[2] toPosNorm(ivec4 raw) {
	vec3 xyz = vec3(raw.xyz) * MD3_XYZ_SCALE;
	vec3 normal = vec3(0.);
	switch (raw.w) {
		// special cases
		case 0: normal = vec3(0., 0., 1.);
		case 32768: normal = vec3(0., 0., -1.);
		default:
			float latitude = float((raw.w >> 8) & 0xFF) / BYTE_TAU;
			float longtude = float(raw.w & 0xFF) / BYTE_TAU;
			float sl = sin(longtude);
			normal = vec3(
				cos(latitude) * sl,
				sin(latitude) * sl,
				cos(longtude));
	}
	return vec3[2](xyz, normal);
}

void main() {
	float interp = fract(frame);
	int framea = int(floor(frame));
	int frameb = int(ceil(frame));
	ivec2 uva = ivec2(aIndex, framea); // Vertex on X axis, frame on Y axis
	ivec2 uvb = ivec2(aIndex, frameb);
	ivec4 ia = texelFetch(anim, uva, 0);
	vec3[2] va = toPosNorm(ia);
	ivec4 ib = texelFetch(anim, uvb, 0);
	vec3[2] vb = toPosNorm(ib);
	position = mix(va[0], vb[0], interp);
	normal = mix(va[1], vb[1], interp);
	uv = aUv;
	gl_Position = eye * vec4(position, 1.);
}
