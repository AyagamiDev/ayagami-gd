extends Control

var dragging = false
var zoom: Vector2 = Vector2.ONE
var bounds: Rect2
@export var camera: Camera2D

func _on_model_loaded(model: AyagamiModel) -> void:
	var vs = get_viewport().get_visible_rect().size
	var ms = Vector2(model.size)
	var ri = ms.aspect()
	var rs = vs.aspect()
	var ts = Vector2(ms.x * vs.y/ms.y, vs.y) if rs > ri else Vector2(vs.x, ms.y * vs.x / ms.y)

	zoom = ts / ms
	camera.zoom = zoom
	camera.position = Vector2.ZERO
	bounds = Rect2(
		-model.size / 2.0,
		model.size
	)

func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_LEFT:
			dragging = event.pressed
		if event.button_index == MOUSE_BUTTON_WHEEL_UP:
			camera.zoom += Vector2.ONE * 0.01
		if event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			camera.zoom -= Vector2.ONE * 0.01
		camera.zoom = clamp(camera.zoom, Vector2(0.0001, 0.0001), Vector2(3.0, 3.0))

	if dragging:
		if event is InputEventMouseMotion:
			camera.position -= event.screen_relative / camera.zoom

func _on_camera_reset_pressed() -> void:
	camera.zoom = zoom
	camera.position = Vector2.ZERO
