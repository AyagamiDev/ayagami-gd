# register_blend.gd
# Make sure this is included in your Project's autoloads to fix blending
# Once the PR is merged or a better solution is exposed, this will be handled
# within the plugin itself

@tool
extends Node

static func _static_init():
	var mix = RDPipelineColorBlendStateAttachment.new()
	mix.enable_blend = true
	mix.src_color_blend_factor = RenderingDevice.BLEND_FACTOR_ONE
	mix.dst_color_blend_factor = RenderingDevice.BLEND_FACTOR_ONE_MINUS_SRC_ALPHA
	mix.src_alpha_blend_factor = RenderingDevice.BLEND_FACTOR_ONE
	mix.dst_alpha_blend_factor = RenderingDevice.BLEND_FACTOR_ONE_MINUS_SRC_ALPHA
	
	var add = RDPipelineColorBlendStateAttachment.new()
	add.enable_blend = true
	add.src_color_blend_factor = RenderingDevice.BLEND_FACTOR_ONE
	add.dst_color_blend_factor = RenderingDevice.BLEND_FACTOR_ONE
	add.src_alpha_blend_factor = RenderingDevice.BLEND_FACTOR_ZERO
	add.dst_alpha_blend_factor = RenderingDevice.BLEND_FACTOR_ONE
	
	var mul = RDPipelineColorBlendStateAttachment.new()
	mul.enable_blend = true
	mul.src_color_blend_factor = RenderingDevice.BLEND_FACTOR_DST_COLOR
	mul.dst_color_blend_factor = RenderingDevice.BLEND_FACTOR_ONE_MINUS_SRC_ALPHA
	mul.src_alpha_blend_factor = RenderingDevice.BLEND_FACTOR_ZERO
	mul.dst_alpha_blend_factor = RenderingDevice.BLEND_FACTOR_ONE
	
	RenderingServer.register_blend_mode(RenderingServer.SHADER_CANVAS_ITEM, "ayagami_mix", mix)
	RenderingServer.register_blend_mode(RenderingServer.SHADER_CANVAS_ITEM, "ayagami_add", add)
	RenderingServer.register_blend_mode(RenderingServer.SHADER_CANVAS_ITEM, "ayagami_mul", mul)
	
	print("Ayagami blend modes registered")
	
