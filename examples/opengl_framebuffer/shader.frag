#version 330 core

in vec4 vert_color;
in vec2 texture_uv;

uniform sampler2D in_texture;

out vec4 frag_color;

void main()
{
    frag_color = vert_color * texture(in_texture, texture_uv);
}
