## The map editor's dock: the brush, the list of locations, the chosen location's definitions, and saving.

@tool
@icon("res://editor/icons/MapEditorDock.svg")
class_name MapEditorDock extends VBoxContainer

signal tool_changed(tool: StringName) ## Another tool was taken up.
signal location_chosen(location: int) ## A location was chosen from the list.
signal add_requested ## The new location button was pressed.
signal field_edited(location: int, field: StringName, value: String) ## One of the chosen location's definitions was changed.
signal save_requested ## The save button was pressed.

var _location: int = -1

@onready var _brush_panel: MapBrushPanel = %BrushPanel
@onready var _location_list: MapLocationList = %LocationList
@onready var _inspector: MapLocationInspector = %LocationInspector
@onready var _save_button: Button = %SaveButton
@onready var _status: Label = %Status


func _ready() -> void:
	_status.text = ""
	_brush_panel.tool_changed.connect(_on_brush_panel_tool_changed)
	_location_list.location_chosen.connect(_on_location_list_location_chosen)
	_location_list.add_requested.connect(_on_location_list_add_requested)
	_inspector.field_edited.connect(_on_inspector_field_edited)
	_save_button.pressed.connect(_on_save_button_pressed)


## Lists the map's locations, by id, and the terrains they can be.
func show_locations(names: PackedStringArray, terrains: PackedStringArray) -> void:
	_location_list.show_locations(names)
	_inspector.show_terrains(terrains)


## Shows a location's definitions, and marks it in the list.
func show_location(location: int, fields: Dictionary[StringName, String]) -> void:
	_location = location
	_location_list.show_chosen(location)
	_inspector.show_fields(fields)


func show_status(text: String) -> void:
	_status.text = text


func get_tool() -> StringName:
	return _brush_panel.current_tool


func get_radius() -> float:
	return _brush_panel.get_radius()


func get_strength() -> float:
	return _brush_panel.get_strength()


## The chosen location, or -1 for none.
func get_location() -> int:
	return _location


func _on_brush_panel_tool_changed(tool: StringName) -> void:
	tool_changed.emit(tool )


func _on_location_list_location_chosen(location: int) -> void:
	location_chosen.emit(location)


func _on_location_list_add_requested() -> void:
	add_requested.emit()


func _on_inspector_field_edited(field: StringName, value: String) -> void:
	if _location >= 0:
		field_edited.emit(_location, field, value)


func _on_save_button_pressed() -> void:
	save_requested.emit()
