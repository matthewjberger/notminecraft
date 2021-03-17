#version 450 core

layout (location = 0) in vec3 v_position;

uniform mat4 mvp;

out VS_OUT
{
  vec4 color;
} vs_out;

void main()
{
    vec4 position = vec4(v_position, 1.0);
    gl_Position = mvp * position;
    vs_out.color = position * 2.0 + vec4(0.5, 0.5, 0.5, 1.0);
}