#version 330 core

uniform sampler2D tex;
in vec3 position;
in vec3 eyeNormal;
in vec2 uv;
out vec4 FragColor;

void main() {
	vec3 up = vec3(0., 1., 0.);
	float brightness = max(0., -eyeNormal.z);
	FragColor = vec4(vec3(brightness), 1.);
	// FragColor = vec4(eyeNormal, 1.);
	// FragColor = texture(tex, uv);
}
