shader_type canvas_item;
render_mode blend_ayagami_mul, unshaded;

#include "_draw.gdshaderinc"

void fragment() {
	if (discard_mask()) {
		discard;
	}
	
	vec4 color_tex = texture(TEXTURE, UV);
	float mask_val = sample_mask();
	
	if (color_tex.a == 0.0 || mask_val == 0.0) {
		discard;
	}
	
	vec4 color = artmesh_color(color_tex);
	color *= color_base;
	color *= modulate;
	color *= vec4(vec3(1.0), mask_val);
	
	COLOR = vec4(color.rgb * color.a, color.a);
}