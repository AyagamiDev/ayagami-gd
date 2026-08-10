extends VBoxContainer

var model: AyagamiModel
var parameter: String :
	set(v):
		parameter = v
		var current_value: float = model.get(parameter)
		var value_range: Vector2 = Vector2(0, 1) if parameter.begins_with("parts/") else model.get("%s/range" % [parameter])
		%Slider.min_value = value_range.x
		%Slider.max_value = value_range.y
		%Slider.set_value_no_signal(current_value)
		var prefix = v.left(v.find("/"))
		%ParameterName.text = v.trim_prefix(prefix + "/")

var editable: bool = true:
	set(toggle):
		%Slider.editable = toggle

func _on_reset_button_pressed() -> void:
	%Slider.value = model.property_get_revert(parameter)
	
func _on_slider_value_changed(value: float) -> void:
	model.set("parameters/%s" % [parameter], value)
	
func _process(delta: float) -> void:
	if model.is_loaded():
		%CurrentValue.text = "%.1f" % [model.get(parameter)]
		%RealValue.text = "%.1f" % [model.get("output/%s" % [parameter])]
