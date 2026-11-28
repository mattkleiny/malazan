## A text box for knowledge markup that colours its links, completes them as they are typed, and opens the concept a link names on Ctrl+click.

@tool
@icon("res://editor/icons/KnowledgeMarkupEdit.svg")
class_name KnowledgeMarkupEdit extends CodeEdit

signal link_requested ## Ctrl+K was pressed, asking to choose a concept to link.
signal concept_opened(concept: KnowledgeConcept) ## A link to a concept was Ctrl+clicked.

const CONCEPT_ICON: Texture2D = preload("res://editor/icons/KnowledgeConcept.svg")
const CONCEPT_KIND: StringName = &"concept"
const MIN_HEIGHT: float = 160.0


func _init() -> void:
	custom_minimum_size.y = MIN_HEIGHT
	wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
	autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	code_completion_enabled = true
	code_completion_prefixes = [":"]
	symbol_lookup_on_click = true
	syntax_highlighter = KnowledgeMarkupHighlighter.new()


func _ready() -> void:
	add_theme_font_override(&"font", get_theme_font(&"source", &"EditorFonts"))
	add_theme_font_size_override(&"font_size", get_theme_font_size(&"source_size", &"EditorFonts"))
	text_changed.connect(_on_text_changed)
	symbol_validate.connect(_on_symbol_validate)
	symbol_lookup.connect(_on_symbol_lookup)


func _gui_input(event: InputEvent) -> void:
	var key := event as InputEventKey
	if key == null or not key.pressed or key.echo:
		return
	if key.keycode == KEY_K and key.is_command_or_control_pressed():
		link_requested.emit()
		accept_event()


func _request_code_completion(force: bool) -> void:
	var partial: String = KnowledgeBase.get_partial_link(_text_before_caret())
	if partial.get_slice(":", 0) != CONCEPT_KIND:
		return

	for concept: KnowledgeConcept in KnowledgeBase.get_concepts():
		var id := String(concept.id)
		add_code_completion_option(CodeEdit.KIND_PLAIN_TEXT, id, id + "]", Color.WHITE, CONCEPT_ICON)
	update_code_completion_options(force)


## Puts a link to a concept at the caret, worded by the selected text if there is any.
func insert_link(id: StringName) -> void:
	var shown := get_selected_text()
	var link := "[concept:%s]" % id if shown.is_empty() else "[concept:%s|%s]" % [id, shown]
	begin_complex_operation()
	delete_selection()
	insert_text_at_caret(link)
	end_complex_operation()
	grab_focus()


## Colours links afresh after the concepts they name have changed.
func refresh_links() -> void:
	syntax_highlighter.clear_highlighting_cache()
	queue_redraw()


func _text_before_caret() -> String:
	return get_line(get_caret_line()).left(get_caret_column())


func _concept_at(line: int, column: int) -> KnowledgeConcept:
	for link: KnowledgeLink in KnowledgeBase.find_links(get_line(line)):
		if column >= link.start and column < link.end:
			return link.concept
	return null


func _on_text_changed() -> void:
	if not KnowledgeBase.get_partial_link(_text_before_caret()).is_empty():
		request_code_completion()


func _on_symbol_validate(_symbol: String) -> void:
	var at := get_line_column_at_pos(Vector2i(get_local_mouse_position()))
	set_symbol_lookup_word_as_valid(at.y >= 0 and _concept_at(at.y, at.x) != null)


func _on_symbol_lookup(_symbol: String, line: int, column: int) -> void:
	var concept := _concept_at(line, column)
	if concept != null:
		concept_opened.emit(concept)
