## The map editor's list of locations, with a search to narrow it and a button to add one.

@tool
@icon("res://editor/icons/MapLocationList.svg")
class_name MapLocationList extends VBoxContainer

signal location_chosen(location: int) ## A location was picked from the list.
signal add_requested ## The new location button was pressed.

var _names: PackedStringArray = PackedStringArray()
var _chosen: int = -1

@onready var _search: LineEdit = %Search
@onready var _items: ItemList = %Items
@onready var _add_button: Button = %AddButton


func _ready() -> void:
	_items.clear()
	_search.text_changed.connect(_on_search_text_changed)
	_items.item_selected.connect(_on_items_item_selected)
	_add_button.pressed.connect(_on_add_button_pressed)


## Lists the locations by id, keeping the search.
func show_locations(names: PackedStringArray) -> void:
	_names = names
	_fill()


## Marks a location in the list without choosing it again.
func show_chosen(location: int) -> void:
	_chosen = location
	_items.deselect_all()
	for index in _items.item_count:
		if _items.get_item_metadata(index) == location:
			_items.select(index)
			_items.ensure_current_is_visible()
			return


func _fill() -> void:
	_items.clear()
	var search := _search.text.strip_edges().to_lower()
	for location in _names.size():
		var name := _names[location]
		if not search.is_empty() and not name.to_lower().contains(search):
			continue

		var index := _items.add_item(name)
		_items.set_item_metadata(index, location)

	show_chosen(_chosen)


func _on_search_text_changed(_text: String) -> void:
	_fill()


func _on_items_item_selected(index: int) -> void:
	_chosen = _items.get_item_metadata(index)
	location_chosen.emit(_chosen)


func _on_add_button_pressed() -> void:
	add_requested.emit()
