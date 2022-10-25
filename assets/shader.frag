#version 330 core

uniform sampler2D tex;
in vec3 position;
in vec3 normal;
in vec2 uv;
out vec4 FragColor;

void main() {
	vec3 up = vec3(0., 1., 0.);
	float brightness = abs(dot(up, normal));
	FragColor = texture(tex, uv) * brightness;
}
