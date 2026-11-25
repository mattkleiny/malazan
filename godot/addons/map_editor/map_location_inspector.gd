## The map editor's view of one location's definitions: its name, the divisions it sits in, and its terrain.

@tool
@icon("res://editor/icons/MapLocationInspector.svg")
class_name MapLocationInspector extends GridContainer

signal field_edited(field: StringName, value: String) ## A definition was changed and committed.

const FIELDS: Array[StringName] = [&"name", &"province", &"area", &"region", &"continent", &"terrain"] ## Every definition of a location, in the order shown.
const TEXT_FIELDS: Array[StringName] = [&"name", &"province", &"area", &"region", &"continent"]

var _shown: Dictionary[StringName, String] = {}

@onready var _edits: Dictionary[StringName, LineEdit] = {
	&"name": %Name,
	&"province": %Province,
	&"area": %Area,
	&"region": %Region,
	&"continent": %Continent,
}
@onready var _terrain: OptionButton = %Terrain


func _ready() -> void:
	for field: StringName in TEXT_FIELDS:
		var edit: LineEdit = _edits[field]
		edit.text = ""
		edit.text_submitted.connect(func(_text: String) -> void: _commit(field))
		edit.focus_exited.connect(func() -> void: _commit(field))
	_terrain.item_selected.connect(_on_terrain_item_selected)


## Fills the choices of terrain.
func show_terrains(terrains: PackedStringArray) -> void:
	_terrain.clear()
	for terrain: String in terrains:
		_terrain.add_item(terrain.capitalize())
		_terrain.set_item_metadata(_terrain.item_count - 1, terrain)


## Shows a location's definitions, by field.
func show_fields(fields: Dictionary[StringName, String]) -> void:
	_shown = fields
	for field: StringName in TEXT_FIELDS:
		_edits[field].text = fields.get(field, "")

	for index in _terrain.item_count:
		if _terrain.get_item_metadata(index) == fields.get(&"terrain", ""):
			_terrain.select(index)


func _commit(field: StringName) -> void:
	var value: String = _edits[field].text.strip_edges()
	if value == _shown.get(field, ""):
		return

	field_edited.emit(field, value)


func _on_terrain_item_selected(index: int) -> void:
	var value: String = _terrain.get_item_metadata(index)
	if value == _shown.get(&"terrain", ""):
		return

	field_edited.emit(&"terrain", value)
