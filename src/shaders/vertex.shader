#version 330 core

layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec4 in_color;
layout (location = 2) in vec2 in_texture_uv;

uniform mat4 projection;

out vec4 vert_color;
out vec2 texture_uv;

void main()
{
    gl_Position = projection * vec4(in_pos, 1.0);
    vert_color = in_color;
    texture_uv = in_texture_uv;
}
