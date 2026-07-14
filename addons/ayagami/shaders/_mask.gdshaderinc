// masking

uniform bool invert = false;
uniform bool has_mask = false;

uniform sampler2D tex_mask : filter_linear_mipmap, repeat_disable, hint_default_black;
uniform float mask_scale = 1.0;
instance uniform vec2 canvas_size = vec2(2,2);
instance uniform vec2 mask_offset = vec2(0,0);

varying vec2 MASK_UV;
varying vec4 modulate;

bool discard_mask() {
	if (has_mask) {
		if (MASK_UV.x > 1.0 || MASK_UV.y > 1.0 || MASK_UV.x < 0.0 || MASK_UV.y < 0.0) {
			return true;
		}
	}
	return false;
}

vec2 mask_uv(vec2 vtx) {
	vec2 mask_size = vec2(textureSize(tex_mask, 0)) / mask_scale;
    vec2 mask_vtx = vtx - mask_offset;
    return mask_vtx / mask_size;
}

float sample_mask() {
	if (!has_mask) {
		return 1.0;
	}
	
	vec4 clip_mask = texture(tex_mask, MASK_UV) * vec4(0,0,0,1);

    float mask_val = clip_mask.r + clip_mask.g + clip_mask.b + clip_mask.a;
    if (invert) {
        mask_val = 1.0 - mask_val;
    }
	return mask_val;
}

void vertex() {
	MASK_UV = mask_uv(VERTEX.xy);
	modulate = COLOR;
}