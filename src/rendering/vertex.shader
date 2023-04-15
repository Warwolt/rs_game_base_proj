#version 330 core
layout (location = 0) in vec3 pos;
layout (location = 1) in vec4 color;

uniform mat4 projection;

out vec4 vert_color;

void main()
{
    gl_Position = projection * vec4(pos, 1.0);
    vert_color = color;
}
