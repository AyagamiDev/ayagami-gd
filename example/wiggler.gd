extends AyagamiMutator

func _apply(pose: AyagamiPose) -> void:
	var now = Time.get_ticks_msec() / 500.0
	var wiggle = sin(now) * 10.0
	
	pose.blend("parameters/ParamBodyAngleZ", wiggle)
	
