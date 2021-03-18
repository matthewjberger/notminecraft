#version 450 core

out vec4 outColor;

in vec2 uv;

uniform sampler2D tex;

void main()
{
  outColor = texture(tex, uv);
}