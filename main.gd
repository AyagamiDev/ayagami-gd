extends Node

func _on_file_picker_pressed() -> void:
	var file_picker = %FileDialog
	file_picker.popup_centered()

func _on_file_selected(path: String) -> void:
	if path.is_empty():
		return
	
	var model = AyagamiLoader.load_model(path)
	model.name = "LoadedModel"
	
	var vs = get_viewport().get_visible_rect().size
	var ms = Vector2(model.size)
	var ri = ms.aspect()
	var rs = vs.aspect()
	var ts = Vector2(ms.x * vs.y/ms.y, vs.y) if rs > ri else Vector2(vs.x, ms.y * vs.x / ms.y)
	model.scale = ts / ms
	
	$Camera2D.position = (Vector2(model.size) * model.scale) / 2.0
	$LoadedModel.queue_free()
	await get_tree().process_frame
	add_child(model)
	
#region populate parameters
	for i in %ParameterList.get_children():
		i.queue_free()
		
	for property in model.get_property_list():
		if property.name.begins_with("parameters/"):
			var container = PanelContainer.new()
			container.size_flags_horizontal = Control.SIZE_EXPAND_FILL

			var layout = VBoxContainer.new()
			layout.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			container.add_child(layout)

			var label = Label.new()
			label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			label.clip_text = true
			label.text = (property.name as String).right(-len("parameters/"))
			layout.add_child(label)

			var slider = HSlider.new()
			slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			slider.min_value = property.hint_string.split(",")[0].to_float()
			slider.max_value = property.hint_string.split(",")[1].to_float()
			slider.step = 0.01
			slider.value = model.get(property.name)
			layout.add_child(slider)
			
			slider.value_changed.connect(
				func (v):
					model.set(property.name, v)
			)

			%ParameterList.add_child(container)
#endregion

#region populate parts
	for i in %PartList.get_children():
		i.queue_free()
		
	for property in model.get_property_list():
		if property.name.begins_with("parts/"):
			var container = PanelContainer.new()
			container.size_flags_horizontal = Control.SIZE_EXPAND_FILL

			var layout = VBoxContainer.new()
			layout.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			container.add_child(layout)

			var label = Label.new()
			label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			label.clip_text = true
			label.text = (property.name as String).right(-len("parts/"))
			layout.add_child(label)

			var slider = HSlider.new()
			slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			slider.min_value = 0.0
			slider.max_value = 1.0
			slider.step = 0.01
			slider.value = model.get(property.name)
			layout.add_child(slider)
			
			slider.value_changed.connect(
				func (v):
					model.set(property.name, v)
			)

			%PartList.add_child(container)
#endregion

#region load motions
	var anim_player = AnimationPlayer.new()
	model.add_child(anim_player)
	var anim_library = AyagamiLoader.load_motion_library(path.get_base_dir())
	anim_player.add_animation_library("", anim_library)
	anim_player.play("RESET")
	
	for i in %MotionList.get_children():
		i.queue_free()

	var btn_group = ButtonGroup.new()
	for motion in anim_library.get_animation_list():
		var btn = Button.new()
		btn.text = motion
		btn.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		btn.toggle_mode = true
		btn.button_group = btn_group
		btn.toggled.connect(
			func (t):
				if t:
					anim_player.play(motion)
		)
		%MotionList.add_child(btn)
#endregion

	%ModelInfo.text = "|".join([
		"Model: %s" % path.get_file(),
		"Parameters: %d" % %ParameterList.get_child_count(),
		"Parts: %d" % %PartList.get_child_count(),
		"Mesh: %d" % model.get_node("Meshes").get_child_count(),
		"Masks: %d" % model.get_node("Masks").get_child_count(),
		"Canvas Size: %dx%d" % [model.size.x, model.size.y]
	])

func walk_dir(path: String, fn: Callable):
	for f in DirAccess.get_files_at(path):
		fn.call(path.path_join(f))
	
	for d in DirAccess.get_directories_at(path):
		walk_dir(path.path_join(d), fn)
