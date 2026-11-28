## A searchable list of every concept, to choose one to link, or to make a new concept when none fits.

@tool
@icon("res://editor/icons/KnowledgeLinkPicker.svg")
class_name KnowledgeLinkPicker extends PopupPanel

signal concept_chosen(concept: KnowledgeConcept, is_new: bool) ## A concept was chosen from the list, or made from the search.

const CONCEPT_ICON: Texture2D = preload("res://editor/icons/KnowledgeConcept.svg")
const PICKER_SIZE: Vector2i = Vector2i(360, 400)

var _search: LineEdit
var _list: ItemList
var _create: Button


func _init() -> void:
	var lines := VBoxContainer.new()
	_search = LineEdit.new()
	_search.placeholder_text = "Search concepts, or name a new one"
	_search.clear_button_enabled = true
	lines.add_child(_search)

	_list = ItemList.new()
	_list.size_flags_vertical = Control.SIZE_EXPAND_FILL
	lines.add_child(_list)

	_create = Button.new()
	_create.alignment = HORIZONTAL_ALIGNMENT_LEFT
	_create.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	lines.add_child(_create)
	add_child(lines)


func _ready() -> void:
	_create.icon = get_theme_icon(&"Add", &"EditorIcons")
	_search.text_changed.connect(_on_search_text_changed)
	_search.text_submitted.connect(_on_search_text_submitted)
	_search.gui_input.connect(_on_search_gui_input)
	_list.item_activated.connect(_on_list_item_activated)
	_create.pressed.connect(_on_create_pressed)


## Opens at a point of the screen, searching for some text to begin with.
func open_at(screen_position: Vector2i, search: String) -> void:
	_search.text = search
	_show_matches(search)
	popup(Rect2i(screen_position, PICKER_SIZE))
	_search.grab_focus()
	_search.select_all()


func _show_matches(search: String) -> void:
	_list.clear()
	var needle := search.strip_edges().to_lower()
	for concept: KnowledgeConcept in KnowledgeBase.get_concepts():
		if needle.is_empty() or concept.title.to_lower().contains(needle) or String(concept.id).contains(_id_for(needle)):
			var index := _list.add_item("%s  ·  %s" % [concept.title, concept.id], CONCEPT_ICON)
			_list.set_item_metadata(index, concept)
	if _list.item_count > 0:
		_list.select(0)

	var id := _id_for(search)
	_create.visible = id.is_valid_ascii_identifier() and KnowledgeBase.get_concept(id) == null
	_create.text = "Create concept \"%s\"  ·  %s" % [search.strip_edges(), id]


## The id a new concept named this would take: Sea Level gives sea_level.
func _id_for(search: String) -> String:
	return search.strip_edges().to_snake_case().replace(" ", "_")


func _choose(concept: KnowledgeConcept, is_new: bool) -> void:
	hide()
	concept_chosen.emit(concept, is_new)


func _choose_selected() -> void:
	var selected := _list.get_selected_items()
	if not selected.is_empty():
		_choose(_list.get_item_metadata(selected[0]), false)
	elif _create.visible:
		_on_create_pressed()


func _on_search_text_changed(search: String) -> void:
	_show_matches(search)


func _on_search_text_submitted(_search_text: String) -> void:
	_choose_selected()


func _on_search_gui_input(event: InputEvent) -> void:
	var key := event as InputEventKey
	if key == null or not key.pressed or _list.item_count == 0:
		return
	if key.keycode != KEY_DOWN and key.keycode != KEY_UP:
		return

	var selected := _list.get_selected_items()
	var index := selected[0] if not selected.is_empty() else -1
	index = clampi(index + (1 if key.keycode == KEY_DOWN else -1), 0, _list.item_count - 1)
	_list.select(index)
	_list.ensure_current_is_visible()
	_search.accept_event()


func _on_list_item_activated(index: int) -> void:
	_choose(_list.get_item_metadata(index), false)


func _on_create_pressed() -> void:
	var concept: KnowledgeConcept = KnowledgeBase.add_concept(_id_for(_search.text), _search.text.strip_edges())
	if concept == null:
		return
	EditorInterface.get_resource_filesystem().update_file(concept.resource_path)
	_choose(concept, true)
