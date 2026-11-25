## The map screen: picks the location under the cursor and passes map mode choices to the terrain.

class_name MapView extends Node3D

const TIERS: Array[String] = ["province", "area", "region", "continent"] ## The divisions named under a location, smallest first.

var _hovered: int = -1
var _selected: int = -1

@onready var _terrain: TerrainView = %TerrainView
@onready var _camera: MapCamera = %MapCamera
@onready var _location_label: Label = %LocationLabel
@onready var _mode_label: Label = %ModeLabel
@onready var _mode_bar: MapModeBar = %MapModeBar


func _ready() -> void:
	_location_label.text = ""
	_mode_bar.mode_chosen.connect(_on_mode_bar_mode_chosen)
	_mode_bar.mode_previewed.connect(_on_mode_bar_mode_previewed)
	_mode_bar.preview_ended.connect(_on_mode_bar_preview_ended)

	var labels: Dictionary[StringName, String] = {}
	for mode: String in _terrain.get_map_modes():
		labels[StringName(mode)] = _terrain.get_map_mode_label(mode)
	_mode_bar.show_modes(labels)

	_show_mode(_terrain.get_map_mode())


func _unhandled_input(event: InputEvent) -> void:
	var motion := event as InputEventMouseMotion
	if motion != null:
		_hover(_terrain.pick_location(_camera, motion.position))
		return

	var button := event as InputEventMouseButton
	if button != null and button.pressed and button.button_index == MOUSE_BUTTON_LEFT:
		_select(_terrain.pick_location(_camera, button.position))


func _hover(location: int) -> void:
	if location == _hovered:
		return

	_hovered = location
	_terrain.set_hovered_location(location)
	if _selected < 0:
		_describe(location)


func _select(location: int) -> void:
	_selected = location
	_terrain.set_selected_location(location)
	_describe(location if location >= 0 else _hovered)


func _describe(location: int) -> void:
	var info := _terrain.get_location_info(location)
	if info.is_empty():
		_location_label.text = ""
		return

	var tiers: Array[String] = []
	for tier: String in TIERS:
		if info.has(tier):
			tiers.append(info[tier])

	_location_label.text = "%s (%s)\n%s" % [info["name"], info["terrain"], " · ".join(tiers)]


func _show_mode(mode: StringName) -> void:
	_terrain.set_map_mode(mode)
	_mode_bar.show_chosen(mode)
	_mode_label.text = "Map mode: %s" % _terrain.get_map_mode_label(mode)


func _on_mode_bar_mode_chosen(mode: StringName) -> void:
	_terrain.clear_map_mode_preview()
	_show_mode(mode)


func _on_mode_bar_mode_previewed(mode: StringName) -> void:
	_terrain.preview_map_mode(mode)


func _on_mode_bar_preview_ended() -> void:
	_terrain.clear_map_mode_preview()
