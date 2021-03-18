#version 450 core

layout (location = 0) in vec3 v_position;
layout (location = 1) in vec2 v_uv;

uniform mat4 mvp;

out vec2 uv;

void main()
{
    vec4 position = vec4(v_position, 1.0);
    gl_Position = mvp * position;
    uv = v_uv;
}