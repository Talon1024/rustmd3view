#version 330 core

uniform sampler2D tex;
uniform uint mode;
in vec3 position;
in vec3 eyeNormal;
in vec2 uv;
in vec2 barycenter;
out vec4 FragColor;

const uint MODE_TEXTURED = 0u;
const uint MODE_UNTEXTURED = 1u;
const uint MODE_NORMALS = 2u;
const uint MODE_WIREFRAME = 3u;

float gridFactor (vec2 vBC, float width);
float gridFactor (vec2 vBC, float width, float feather);

void main() {
	vec3 up = vec3(0., 1., 0.);
	float brightness = max(0., eyeNormal.z);
	// FragColor = vec4(vec3(brightness), 1.);
	// FragColor = vec4(eyeNormal, 1.);
	switch (mode) {
		case MODE_NORMALS:
			FragColor = vec4(eyeNormal, 1.);
			break;
		case MODE_UNTEXTURED:
			FragColor = vec4(vec3(brightness), 1.);
			break;
		case MODE_WIREFRAME:
			float wire = gridFactor(barycenter, 1.0, 5.0);
			FragColor = vec4(vec3(wire), 1.);
			break;
		case MODE_TEXTURED:
		default:
			FragColor = texture(tex, uv);
			FragColor.rgb *= brightness;
			break;
	}
}

// GLSL wireframe display
// © Ricky Reusser 2016. MIT License.

float gridFactor (vec2 vBC, float width, float feather) {
  float w1 = width - feather * 0.5;
  vec3 bary = vec3(vBC.x, vBC.y, 1.0 - vBC.x - vBC.y);
  vec3 d = fwidth(bary);
  vec3 a3 = smoothstep(d * w1, d * (w1 + feather), bary);
  return min(min(a3.x, a3.y), a3.z);
}

float gridFactor (vec2 vBC, float width) {
  vec3 bary = vec3(vBC.x, vBC.y, 1.0 - vBC.x - vBC.y);
  vec3 d = fwidth(bary);
  vec3 a3 = smoothstep(d * (width - 0.5), d * (width + 0.5), bary);
  return min(min(a3.x, a3.y), a3.z);
}
