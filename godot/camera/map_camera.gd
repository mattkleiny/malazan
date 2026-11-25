## Looks down on the map: pans over it, zooms in and out, and stays over it.

@icon("res://editor/icons/MapCamera.svg")
class_name MapCamera extends Camera3D

const MIN_PITCH: float = 35.0 ## Pitch fully zoomed in (degrees down from the horizon).
const MAX_PITCH: float = 75.0 ## Pitch fully zoomed out (degrees down from the horizon).

@export var terrain: TerrainView ## The map the camera stays over.
@export var min_distance: float = 20.0 ## Closest the camera comes to its focus (map units).
@export var max_distance: float = 900.0 ## Furthest the camera pulls back (map units).
@export var pan_speed: float = 1.2 ## Share of the view's distance panned per second.
@export var zoom_step: float = 1.15 ## Factor one wheel notch zooms by.
@export var zoom_smoothing: float = 10.0 ## How quickly the zoom settles (1/s).

var focus: Vector3 = Vector3(1024.0, 0.0, 512.0) ## The point on the map the camera looks at (map units).
var distance: float = 300.0 ## How far the camera sits from its focus (map units).

var _target_distance: float = 300.0
var _dragging: bool = false


func _ready() -> void:
	_target_distance = distance
	_place()


func _process(delta: float) -> void:
	var input := Input.get_vector(&"ui_left", &"ui_right", &"ui_up", &"ui_down")
	focus += Vector3(input.x, 0.0, input.y) * distance * pan_speed * delta
	distance = lerpf(distance, _target_distance, 1.0 - exp(-zoom_smoothing * delta))
	_keep_over_map()
	_place()


func _unhandled_input(event: InputEvent) -> void:
	var button := event as InputEventMouseButton
	if button != null:
		_apply_mouse_button(button)
		return

	var motion := event as InputEventMouseMotion
	if motion != null and _dragging:
		var units_per_pixel := distance / get_viewport().get_visible_rect().size.y
		focus -= Vector3(motion.relative.x, 0.0, motion.relative.y) * units_per_pixel


## Keeps the focus within the map's edges.
func _keep_over_map() -> void:
	if not is_instance_valid(terrain):
		return

	focus.x = clampf(focus.x, 0.0, terrain.get_map_width())
	focus.z = clampf(focus.z, 0.0, terrain.get_map_height())


## Sets the camera above its focus, tilting further down the further it pulls back.
func _place() -> void:
	var zoom := inverse_lerp(min_distance, max_distance, distance)
	var pitch := deg_to_rad(lerpf(MIN_PITCH, MAX_PITCH, clampf(zoom, 0.0, 1.0)))
	var offset := Vector3(0.0, sin(pitch), cos(pitch)) * distance

	global_position = focus + offset
	look_at(focus, Vector3.UP)


## Zooms on the wheel and drags with the middle or right button.
func _apply_mouse_button(button: InputEventMouseButton) -> void:
	match button.button_index:
		MOUSE_BUTTON_WHEEL_UP:
			_target_distance = clampf(_target_distance / zoom_step, min_distance, max_distance)
		MOUSE_BUTTON_WHEEL_DOWN:
			_target_distance = clampf(_target_distance * zoom_step, min_distance, max_distance)
		MOUSE_BUTTON_MIDDLE, MOUSE_BUTTON_RIGHT:
			_dragging = button.pressed
