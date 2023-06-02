#version 330 core

struct ThickLineInstance {
	vec4 offset_norm_length_px_angle_rad_ccw;
	vec4 colour_rgb;
};

layout(location=0) in vec2 aPos;

out float texCoord;
out vec4 colour;

uniform vec2 windowResolution;
const uint MAX_INSTANCES = 128u;
const float LINE_THICKNESS_PIXELS = 1.25;
uniform ThickLineInstance lineInstances[MAX_INSTANCES];

void main()
{
	vec2 scale = LINE_THICKNESS_PIXELS / windowResolution;
	ThickLineInstance instanceInfo = lineInstances[gl_InstanceID];
	vec2 offsetNorm = instanceInfo.offset_norm_length_px_angle_rad_ccw.xy;
	float lengthPx = instanceInfo.offset_norm_length_px_angle_rad_ccw.z;
	float angleRadCcw = instanceInfo.offset_norm_length_px_angle_rad_ccw.w;
	colour = instanceInfo.colour_rgb;
	texCoord = aPos.y * 0.5 + 0.5; // -1 -> 0, 1 -> 1
	vec2 pos = aPos * vec2(
		// Why do I have to divide by LINE_THICKNESS_PIXELS here?
		lengthPx / LINE_THICKNESS_PIXELS,
		LINE_THICKNESS_PIXELS
	);
	float angSin = sin(angleRadCcw);
	float angCos = cos(angleRadCcw);
	mat2 rotation = mat2(angCos, angSin, -angSin, angCos);
	// The "+ 0.5 / windowResolution" makes the lines look crisper
	pos = rotation * pos * scale + offsetNorm + 0.5 / windowResolution;
	gl_Position = vec4(pos, 0., 1.);
}
