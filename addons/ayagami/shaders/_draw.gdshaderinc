instance uniform vec4 color_base: source_color = vec4(1.0);
instance uniform vec4 color_screen: source_color = vec4(0.0,0.0,0.0,1.0);
instance uniform vec4 color_multiply: source_color = vec4(1.0);

#include "_mask.gdshaderinc"

vec4 artmesh_color(vec4 color_tex) {
	vec3 rgb = color_tex.rgb;
	rgb *= color_multiply.rgb;
	rgb += color_screen.rgb * color_tex.a;
	return vec4(rgb, color_tex.a);
}
