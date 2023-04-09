#version 330 core
out vec4 frag_color;

uniform vec4 color_uniform;

void main() {
    frag_color = color_uniform;
}
