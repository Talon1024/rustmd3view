#version 330 core

uniform bool shaded;
in vec3 colour;
in vec3 normal;
out vec4 FragColor;

void main() {
	vec3 fColour = colour;
	if (shaded) {
		fColour *= dot(normal, vec3(0., 0., 1.));
	}
	FragColor = vec4(fColour, 1.);
}
