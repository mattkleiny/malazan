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
@export var edge_margin: float = 64.0 ## How far past the map's edges the view may show the table (map units).
@export var horizon_reach: float = 2.0 ## How far past the focus a view of the horizon counts as seeing, as a share of the distance.

var focus: Vector3 = Vector3(768.0, 0.0, 1024.0) ## The point on the map the camera looks at (map units).
var distance: float = 300.0 ## How far the camera sits from its focus (map units).

var _target_distance: float = 300.0
var _dragging: bool = false


func _ready() -> void:
	_target_distance = distance
	_place()


func _process(delta: float) -> void:
	var input := Input.get_vector(&"camera_pan_left", &"camera_pan_right", &"camera_pan_up", &"camera_pan_down")
	focus += Vector3(input.x, 0.0, input.y) * distance * pan_speed * delta
	distance = lerpf(distance, _target_distance, 1.0 - exp(-zoom_smoothing * delta))
	_keep_over_map()
	_place()


func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed(&"camera_zoom_in"):
		_target_distance = clampf(_target_distance / zoom_step, min_distance, max_distance)
	elif event.is_action_pressed(&"camera_zoom_out"):
		_target_distance = clampf(_target_distance * zoom_step, min_distance, max_distance)
	elif event.is_action(&"camera_drag"):
		_dragging = event.is_pressed()

	var motion := event as InputEventMouseMotion
	if motion != null and _dragging:
		var units_per_pixel := distance / get_viewport().get_visible_rect().size.y
		focus -= Vector3(motion.relative.x, 0.0, motion.relative.y) * units_per_pixel


## Keeps the ground the camera sees within the map's edges and the margin around them. On an axis where the view is
## wider than that, the map is centred instead.
func _keep_over_map() -> void:
	if not is_instance_valid(terrain):
		return

	var size := Vector2(terrain.get_map_width(), terrain.get_map_height())
	if size.x <= 0.0 or size.y <= 0.0:
		return

	_place()
	var seen := _seen_extent()
	var low := Vector2.ONE * -edge_margin - seen.position
	var high := size + Vector2.ONE * edge_margin - seen.end

	focus.x = clampf(focus.x, low.x, high.x) if low.x <= high.x else (low.x + high.x) * 0.5
	focus.z = clampf(focus.z, low.y, high.y) if low.y <= high.y else (low.y + high.y) * 0.5


## The ground the screen's corners look at, as offsets from the focus on the map's x and z (map units).
func _seen_extent() -> Rect2:
	var screen := get_viewport().get_visible_rect()
	var corners: Array[Vector2] = [
		screen.position, Vector2(screen.end.x, screen.position.y), screen.end, Vector2(screen.position.x, screen.end.y)
	]
	var reach := distance * horizon_reach
	var extent := Rect2()

	for i: int in corners.size():
		var origin := project_ray_origin(corners[i])
		var direction := project_ray_normal(corners[i])
		var flat := Vector2(direction.x, direction.z)
		var point := Vector2(origin.x, origin.z) + flat.normalized() * reach * 4.0
		if direction.y < 0.0:
			point = Vector2(origin.x, origin.z) + flat * (origin.y / -direction.y)

		var offset := (point - Vector2(focus.x, focus.z)).limit_length(reach)
		extent = Rect2(offset, Vector2.ZERO) if i == 0 else extent.expand(offset)

	return extent


## Sets the camera above its focus, tilting further down the further it pulls back.
func _place() -> void:
	var zoom := inverse_lerp(min_distance, max_distance, distance)
	var pitch := deg_to_rad(lerpf(MIN_PITCH, MAX_PITCH, clampf(zoom, 0.0, 1.0)))
	var offset := Vector3(0.0, sin(pitch), cos(pitch)) * distance

	global_position = focus + offset
	look_at(focus, Vector3.UP)

