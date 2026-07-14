shader_type canvas_item;
render_mode blend_mix, unshaded;

void fragment() {
    vec4 color_tex = texture(TEXTURE, UV);
    COLOR = vec4(0,0,0,color_tex.a);
}
