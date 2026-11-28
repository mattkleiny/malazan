## The table the map lies on: its boards, the light pooled around the map and the knife stuck through it.

@icon("res://editor/icons/MapTable.svg")
class_name MapTable extends Node3D

@export var knife_place: Vector2 = Vector2(0.86, 0.9) ## Where the knife stands, as a share of the map's width and height.
@export var drop: float = 1.0 ## How far below the sea's surface the table lies (map units).

@onready var _surface: MeshInstance3D = %Surface
@onready var _knife: Node3D = %Knife


## Lays the table under a map of this size (map units), whose north-west corner is at the table's origin.
func lay_under(map_width: float, map_height: float) -> void:
	var map_size := Vector2(map_width, map_height)
	_surface.position = Vector3(map_size.x * 0.5, -drop, map_size.y * 0.5)
	_knife.position = Vector3(map_size.x * knife_place.x, 0.0, map_size.y * knife_place.y)

	var boards := _surface.material_override as ShaderMaterial
	if boards == null:
		push_warning("MapTable: the surface has no table material")
		return

	boards.set_shader_parameter(&"map_size", map_size)
