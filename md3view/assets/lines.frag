#version 330 core

in float texCoord;
in vec4 colour;

out vec4 FragColor;

const float thickness = 0.0;

// Anti-aliased line
void main() {
	float dist = abs(.5 - texCoord) * 2.;
	float factor = 1. / (1. - thickness);
	if (thickness > 0.) {
		dist = max(0., dist - thickness) * factor;
	}
	dist = 1. - dist;
	FragColor = colour * dist;
}
