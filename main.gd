extends Node

func _on_file_picker_pressed() -> void:
	var file_picker = %FileDialog
	file_picker.popup_centered()

func _on_file_selected(path: String) -> void:
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
	
	for i in %Parameters.get_children():
		i.queue_free()
		
	for property in model.get_property_list():
		if property.name.begins_with("parameters/"):
			print(property)
			var container = PanelContainer.new()
			container.size_flags_horizontal = Control.SIZE_EXPAND_FILL

			var layout = VBoxContainer.new()
			layout.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			container.add_child(layout)

			var label = Label.new()
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

			%Parameters.add_child(container)

	%ModelInfo.text = "Model: {0} | Parameters: {1} | Meshes: {2} | Size: {3}x{4}".format([
		path.get_file(),
		%Parameters.get_child_count(),
		model.get_node("Meshes").get_child_count(),
		model.size.x,
		model.size.y,
	])
