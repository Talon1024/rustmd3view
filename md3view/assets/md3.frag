#version 330 core

uniform sampler2D tex;
uniform uint mode;
in vec3 position;
in vec3 eyeNormal;
in vec2 uv;
out vec4 FragColor;

const uint MODE_TEXTURED = 0u;
const uint MODE_UNTEXTURED = 1u;
const uint MODE_NORMALS = 2u;

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
		case MODE_TEXTURED:
		default:
			FragColor = texture(tex, uv);
			FragColor.rgb *= brightness;
			break;
	}
}
