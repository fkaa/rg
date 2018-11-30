#version 150

in vec2 a_Uv;
in vec4 a_Color;

uniform sampler2D Texture;

out vec4 Target;

void main()
{
	 Target = vec4(a_Color.rgb, texture2D(Texture, a_Uv).r * a_Color.a);
}