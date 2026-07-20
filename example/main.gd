extends Node2D

var model: AyagamiModel

signal model_loaded(model: AyagamiModel)

func _on_file_picker_pressed() -> void:
	var file_picker = %FileDialog
	file_picker.popup_centered()

func _on_file_selected(path: String) -> void:
	if path.is_empty():
		return
	
	if model:
		model.queue_free()
		await get_tree().process_frame
	
	model = AyagamiLoader.load_model(path)
	model.name = "LoadedModel"
	
	add_child(model)
	
#region populate parameters
	for i in %ParameterList.get_children():
		i.queue_free()
		
	var param_sliders = {}
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
			label.text = (property.name as String).trim_prefix("parameters/")
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
			
			param_sliders[property.name] = slider

			%ParameterList.add_child(container)
#endregion

#region populate parts
	for i in %PartList.get_children():
		i.queue_free()
		
	var part_sliders = {}
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
			label.text = (property.name as String).trim_prefix("parts/")
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
			
			param_sliders[property.name] = slider

			%PartList.add_child(container)
#endregion

#region load motions
	var anim_player: AnimationPlayer = model.get_node("MotionController")
	var anim_library = AyagamiLoader.load_motion_library(model)
	anim_player.remove_animation_library("")
	anim_player.add_animation_library("", anim_library)
	anim_player.play("RESET")
	anim_player.animation_started.connect(
		func (anim):
			var a = anim_player.get_animation(anim)
			for track in range(a.get_track_count()):
				var track_path: String = a.track_get_path(track).get_concatenated_subnames()
				if track_path in part_sliders:
					part_sliders[track_path].editable = false
				if track_path in param_sliders:
					param_sliders[track_path].editable = false
	)
	anim_player.current_animation_changed.connect(
		func (anim):
			%PlayButton.set_pressed_no_signal(anim != "")
			if anim == "":
				for p in part_sliders.values():
					p.editable = true
				for p in param_sliders.values():
					p.editable = true
	)
	anim_player.animation_finished.connect(
		func (_anim):
			%PlayButton.set_pressed_no_signal(false)
			for p in part_sliders.values():
				p.editable = true
			for p in param_sliders.values():
				p.editable = true
	)
	
	%MotionList.clear()
	
	var anim_list = anim_library.get_animation_list().duplicate()
	anim_list.sort_custom(
		func (a, _b):
			return a == "RESET"
	)
	for motion in anim_list:
		%MotionList.add_item(motion)
#endregion

#region Expressions
	for i in %ExpressionList.get_children():
		i.queue_free()
	
	var exp_controller: AyagamiExpressionMutator = model.get_node("ExpressionController")
	var exp_library = AyagamiLoader.load_expression_library(path.get_base_dir(), true)
	var expressions = exp_library.keys() as Array[AyagamiExpression]
	exp_controller.expressions = expressions
	
	expressions.sort_custom(
		func (e, _e):
			return exp_library[e] == ""
	)
	var group = ""
	var btn_group = null
	for e in expressions:
		if group != exp_library[e]:
			group = exp_library[e]
			var l = Label.new()
			l.text = group
			%ExpressionList.add_child(l)
			btn_group = ButtonGroup.new()
			btn_group.allow_unpress = true
		
		exp_controller.set("expression_groups/%s" % e.get_name(), group)
		
		var ename = e.get_name()
		
		var row = HBoxContainer.new()
		var btn = CheckBox.new()
		btn.button_group = btn_group
		btn.text = ename
		btn.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		row.add_child(btn)

		var duration = SpinBox.new()
		duration.step = 0.01
		duration.min_value = 0
		duration.max_value = 5.0
		duration.value = 0.5
		duration.custom_minimum_size = Vector2i(64, 0)
		row.add_child(duration)
		
		btn.pressed.connect(
			func ():
				var toggled = btn.button_pressed
				exp_controller.create_tween().tween_property(
					exp_controller,
					"weight/%s" % e.get_name(),
					1.0 if toggled else 0.0,
					duration.value
				)
		)
		
		%ExpressionList.add_child(row)
#endregion
	await get_tree().process_frame
	
	%ModelInfo.text = "|".join([
		"Model: %s" % path.get_file(),
		"Parameters: %d" % %ParameterList.get_child_count(),
		"Parts: %d" % %PartList.get_child_count(),
		"Mesh: %d" % model.get_node("Meshes").get_child_count(),
		"Masks: %d" % model.get_node("Masks").get_child_count(),
		"Canvas Size: %dx%d" % [model.size.x, model.size.y]
	])
	
	model_loaded.emit(model)

func _on_motion_list_item_selected(index: int) -> void:
	if not model:
		return
	var motion = %MotionList.get_item_text(index)
	model.get_node("MotionController").play(motion)

func _on_play_button_toggled(toggled_on: bool) -> void:
	if not model:
		return
	var motion: AnimationPlayer = model.get_node("MotionController")
	if toggled_on:
		motion.play()
	else:
		motion.pause()

func _on_stop_button_pressed() -> void:
	if not model:
		return
	
	(model.get_node("MotionController") as AnimationPlayer).stop()
	
func _on_quality_toggle_toggled(toggled_on: bool) -> void:
	texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR_WITH_MIPMAPS if toggled_on else CanvasItem.TEXTURE_FILTER_NEAREST_WITH_MIPMAPS
