#version 450 core

out vec4 outColor;

in VS_OUT
{
  vec4 color;
} fs_in;


void main()
{
  outColor = fs_in.color;
}