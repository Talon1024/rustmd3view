#version 330 core

uniform bool gzdoom;
uniform isampler2D anim;
uniform mat4 eye;
uniform float frame; // interpolated
layout(location=0) in uint aIndex;
layout(location=1) in vec2 aUv;
out vec3 position;
out vec3 eyeNormal;
out vec2 uv;

const float MD3_XYZ_SCALE = 0.015625; //1./64
const float BYTE_TAU = 40.58451048843331062106; //255./(2.*pi)

vec3[2] toPosNorm(ivec4 raw) {
	vec3 xyz = vec3(raw.xyz) * MD3_XYZ_SCALE;
	vec3 normal = vec3(0.);
	if (!gzdoom) {
		switch (raw.w) {
			// special cases
			case 0: normal = vec3(0., 0., 1.); break;
			case 32768: normal = vec3(0., 0., -1.); break;
			default:
				float latitude = float((raw.w >> 8) & 0xFF) / BYTE_TAU;
				float longtude = float(raw.w & 0xFF) / BYTE_TAU;
				float sl = sin(longtude);
				normal = vec3(
					cos(latitude) * sl,
					sin(latitude) * sl,
					cos(longtude));
		}
	} else {
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
	// Which frames to use?
	int framea = int(floor(frame));
	int frameb = int(ceil(frame));
	// Vertex positions and normals are stored in an RGB integer texture, with
	// the vertex (by index) on the X axis, and the frame on the Y axis.
	ivec2 uva = ivec2(aIndex, framea);
	ivec2 uvb = ivec2(aIndex, frameb);
	ivec4 ia = texelFetch(anim, uva, 0);
	ivec4 ib = texelFetch(anim, uvb, 0);
	vec3[2] va = toPosNorm(ia);
	vec3[2] vb = toPosNorm(ib);
	position = mix(va[0], vb[0], interp);
	// Thanks to https://en.wikibooks.org/wiki/GLSL_Programming/Applying_Matrix_Transformations#Transforming_Directions for "pointing me in the right direction" 😉😉
	eyeNormal = (eye * vec4(mix(va[1], vb[1], interp), 0.)).xyz;
	eyeNormal.z = -eyeNormal.z;
	uv = aUv;
	gl_Position = eye * vec4(position, 1.);
}
