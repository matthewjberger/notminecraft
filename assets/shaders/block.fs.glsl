#version 450 core

out vec4 outColor;

in vec2 uv;

uniform sampler2DArray tex;
uniform int blockId;

void main()
{
  vec2 texCoord = uv;
  texCoord.y *= -1.0;
  outColor = texture(tex, vec3(texCoord, blockId));
  if (outColor.a == 0.0) {
    discard;
  }
}