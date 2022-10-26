#version 330 core

uniform mat4 eye;
layout(location=0) in vec3 aPosition;
layout(location=1) in vec3 aColour;
out vec3 colour;

void main() {
	colour = aColour;
	gl_Position = eye * aPosition;
}
