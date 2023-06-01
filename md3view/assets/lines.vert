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
const uint LINE_THICKNESS_PIXELS = 2u;
uniform ThickLineInstance lineInstances[MAX_INSTANCES];

void main()
{
	vec2 scale = float(LINE_THICKNESS_PIXELS) / windowResolution;
	ThickLineInstance instanceInfo = lineInstances[gl_InstanceID];
	vec2 offsetNorm = instanceInfo.offset_norm_length_px_angle_rad_ccw.xy;
	float lengthPx = instanceInfo.offset_norm_length_px_angle_rad_ccw.z;
	float angleRadCcw = instanceInfo.offset_norm_length_px_angle_rad_ccw.w;
	colour = instanceInfo.colour_rgb;
	texCoord = aPos.y + 1. / 2.; // -1 -> 0, 1 -> 1
	vec2 pos = aPos * scale * vec2(lengthPx, 1.);
	float sin_angle = sin(angleRadCcw);
	float cos_angle = cos(angleRadCcw);
	mat2 rotation = mat2(cos_angle, sin_angle, -sin_angle, cos_angle);
	pos = rotation * pos + offsetNorm;
	gl_Position = vec4(pos, 0., 1.);
}
