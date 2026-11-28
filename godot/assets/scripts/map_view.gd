## The map screen: picks what lies under the cursor in the shown map mode and passes map mode choices to the terrain.

@icon("res://editor/icons/MapView.svg")
class_name MapView extends Node3D

var _hovered: int = -1

@onready var _terrain: TerrainView = %TerrainView
@onready var _table: MapTable = %MapTable
@onready var _camera: MapCamera = %MapCamera
@onready var _selection_label: Label = %SelectionLabel
@onready var _mode_label: KnowledgeLabel = %ModeLabel
@onready var _mode_bar: MapModeBar = %MapModeBar


func _ready() -> void:
	_selection_label.text = ""
	_mode_bar.mode_chosen.connect(_on_mode_bar_mode_chosen)
	_mode_bar.mode_previewed.connect(_on_mode_bar_mode_previewed)
	_mode_bar.preview_ended.connect(_on_mode_bar_preview_ended)

	var labels: Dictionary[StringName, String] = {}
	for mode: String in _terrain.get_map_modes():
		labels[StringName(mode)] = _terrain.get_map_mode_label(mode)
	_mode_bar.show_modes(labels)
	_table.lay_under(_terrain.get_map_width(), _terrain.get_map_height())

	_show_mode(_terrain.get_map_mode())


func _unhandled_input(event: InputEvent) -> void:
	var motion := event as InputEventMouseMotion
	if motion != null:
		_hover(_terrain.pick_location(_camera, motion.position))
		return

	var button := event as InputEventMouseButton
	if button != null and button.pressed and button.button_index == MOUSE_BUTTON_LEFT:
		_terrain.select_location(_terrain.pick_location(_camera, button.position))
		_describe()


func _hover(location: int) -> void:
	if location == _hovered:
		return

	_hovered = location
	_terrain.hover_location(location)
	_describe()


## Names what was chosen, or else what the cursor rests on.
func _describe() -> void:
	var shown := _terrain.get_selected()
	if shown == null:
		shown = _terrain.get_hovered()
	if shown == null:
		_selection_label.text = ""
		return

	_selection_label.text = "\n".join([shown.title] + Array(shown.details))


func _show_mode(mode: StringName) -> void:
	_terrain.set_map_mode(mode)
	_mode_bar.show_chosen(mode)
	_mode_label.markup = "[concept:map_mode]: %s" % _terrain.get_map_mode_label(mode)
	_describe()


func _on_mode_bar_mode_chosen(mode: StringName) -> void:
	_terrain.clear_map_mode_preview()
	_show_mode(mode)


func _on_mode_bar_mode_previewed(mode: StringName) -> void:
	_terrain.preview_map_mode(mode)
	_describe()


func _on_mode_bar_preview_ended() -> void:
	_terrain.clear_map_mode_preview()
	_describe()
