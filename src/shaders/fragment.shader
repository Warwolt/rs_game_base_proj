#version 330 core

in vec4 vert_color;
in vec2 texture_uv;

uniform sampler2D in_texture;
uniform vec4 color_key; // set alpha to zero to disable

out vec4 frag_color;

void main()
{
    vec4 texel = texture(in_texture, texture_uv);
    if (texel == color_key) {
        discard;
    }
    frag_color = vert_color * texture(in_texture, texture_uv);
}
