#version 450 core

out vec4 outColor;

in vec2 uv;

uniform sampler2D tex;

void main()
{
  vec2 texCoord = uv;
  texCoord.y *= -1.0;
  outColor = texture(tex, texCoord);
}