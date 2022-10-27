#version 330 core

uniform mat4 eye;
layout(location=0) in vec3 aPosition;
layout(location=1) in vec3 aColour;
out vec3 colour;

void main() {
	colour = aColour;
	vec4 position = vec4(aPosition, 1.);
	gl_Position = eye * position;
}
