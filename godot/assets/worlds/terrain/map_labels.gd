## Letters the map's labels across the ground, where the designer placed them, in the game and in the editor.

@tool
@icon("res://editor/icons/MapLabels.svg")
class_name MapLabels extends Node3D

const LAND_FONT: Font = preload("res://assets/ui/fonts/map_lettering.tres")
const WATER_FONT: Font = preload("res://assets/ui/fonts/map_lettering_water.tres")
const FONT_SIZE: int = 64 ## Size the lettering is drawn at before it is scaled onto the map (px).

@export var land_color: Color = Color(0.17, 0.11, 0.07, 0.9) ## Ink for the names of the land.
@export var water_color: Color = Color(0.2, 0.28, 0.33, 0.8) ## Ink for the names of seas and lakes.
@export var halo_color: Color = Color(0.89, 0.82, 0.66, 0.55) ## Paper showing around the letters, so they read over the drawings.
@export var lift: float = 0.6 ## How far the lettering floats above the ground (map units).

var _terrain: TerrainView
var _letterings: Array[Label3D] = []


func _ready() -> void:
	# the labels lie on the terrain they are a child of
	_terrain = get_parent() as TerrainView
	if _terrain == null:
		push_warning("MapLabels: must be a child of a TerrainView to letter its labels")
		return

	_terrain.labels_changed.connect(_on_terrain_labels_changed)
	_show(_terrain.get_map_labels())


## The label whose lettering lies under a point of the screen, or -1 for none; the last lettered wins.
func label_at(camera: Camera3D, screen_position: Vector2) -> int:
	for index in range(_letterings.size() - 1, -1, -1):
		var outline := get_screen_outline(camera, index)
		if not outline.is_empty() and Geometry2D.is_point_in_polygon(screen_position, outline):
			return index
	return -1


## The corners of a label's lettering on the screen, or none when any lies behind the camera.
func get_screen_outline(camera: Camera3D, index: int) -> PackedVector2Array:
	if index < 0 or index >= _letterings.size():
		return PackedVector2Array()

	var lettering := _letterings[index]
	var width := lettering.font.get_string_size(lettering.text, HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE).x
	var half := Vector2(width, FONT_SIZE) * lettering.pixel_size / 2.0
	var corners: Array[Vector3] = [
		Vector3(-half.x, -half.y, 0.0),
		Vector3(half.x, -half.y, 0.0),
		Vector3(half.x, half.y, 0.0),
		Vector3(-half.x, half.y, 0.0),
	]

	var outline := PackedVector2Array()
	for corner: Vector3 in corners:
		var world := lettering.global_transform * corner
		if camera.is_position_behind(world):
			return PackedVector2Array()
		outline.append(camera.unproject_position(world))
	return outline


func _show(labels: Array[MapLabel]) -> void:
	while _letterings.size() > labels.size():
		_letterings.pop_back().queue_free()
	while _letterings.size() < labels.size():
		var lettering := Label3D.new()
		lettering.font_size = FONT_SIZE
		lettering.outline_size = FONT_SIZE / 5 # whole pixels
		lettering.outline_modulate = halo_color
		lettering.shaded = false
		lettering.no_depth_test = true
		lettering.render_priority = 1
		add_child(lettering)
		_letterings.append(lettering)

	for index in labels.size():
		_letter(_letterings[index], labels[index])


func _letter(lettering: Label3D, label: MapLabel) -> void:
	lettering.text = label.text
	lettering.font = WATER_FONT if label.is_water else LAND_FONT
	lettering.modulate = water_color if label.is_water else land_color
	lettering.pixel_size = label.size / FONT_SIZE

	# laid flat on the map, then turned about the up axis
	lettering.rotation = Vector3(-PI / 2.0, deg_to_rad(label.angle), 0.0)
	var ground := _terrain.get_surface_height(label.position.x, label.position.y)
	lettering.position = Vector3(label.position.x, ground + lift, label.position.y)


func _on_terrain_labels_changed() -> void:
	_show(_terrain.get_map_labels())
