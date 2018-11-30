#version 150

in vec3 Position;
in vec2 Uv;
in vec4 Color;

out vec2 a_Uv;
out vec4 a_Color;

uniform Common {
	mat4 Projection;
};

void main()
{
	a_Uv = Uv;
	a_Color = Color;
	gl_Position = Projection * vec4(Position, 1.0);
}