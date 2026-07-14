shader_type canvas_item;
render_mode blend_premul_alpha, unshaded;

#include "_draw.gdshaderinc"

void fragment() {
	if (discard_mask()) {
		discard;
	}
	
    vec4 color_tex = texture(TEXTURE, UV);
    color_tex.rgba = color_tex.rgba * color_multiply.rgba;
	color_tex.rgb = (color_tex.rgb + color_screen.rgb) - (color_tex.rgb * color_screen.rgb);

    float mask_val = sample_mask();

	if (mask_val < 0.01) {
		discard;
	}

	vec4 color = color_tex * color_base * modulate * vec4(1.0, 1.0, 1.0, mask_val);
	COLOR = vec4(color.rgb * color.a, color.a);
}