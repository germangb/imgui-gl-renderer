#version 450 core

in vec2 v_uv;
in vec4 v_color;

out vec4 frag_color;

uniform sampler2D u_texture;

void main() {
    frag_color = texture(u_texture, v_uv) * v_color;
}

