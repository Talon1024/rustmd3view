#version 330 core

uniform mat4 eye;
layout(location=0) in vec3 aPosition;
layout(location=1) in vec3 aColour;
layout(location=2) in vec3 aNormal;
out vec3 colour;
out vec3 normal;

void main() {
	colour = aColour;
	vec4 position = vec4(aPosition, 1.);
	normal = (eye * vec4(aNormal, 0.)).xyz;
	normal.z = -normal.z;
	gl_Position = eye * position;
}
