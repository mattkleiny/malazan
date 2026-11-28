## Edits a concept's description: knowledge markup that links as you type, above a preview of how it reads.

@tool
@icon("res://editor/icons/KnowledgeBodyProperty.svg")
class_name KnowledgeBodyProperty extends EditorProperty

signal markup_edited(markup: String) ## The description changed, by typing or by a link being added.

const HINT: String = "Ctrl+K to link · type [concept: to complete · Ctrl+click a link to open it"

var _edit: KnowledgeMarkupEdit
var _preview: RichTextLabel
var _picker: KnowledgeLinkPicker
var _link_button: Button
var _is_updating: bool = false


func _init() -> void:
	var lines := VBoxContainer.new()

	var toolbar := HBoxContainer.new()
	_link_button = Button.new()
	_link_button.text = "Link"
	_link_button.tooltip_text = "Link to a concept (Ctrl+K). Selected text becomes the link's wording."
	toolbar.add_child(_link_button)
	var hint := Label.new()
	hint.text = HINT
	hint.modulate.a = 0.6
	hint.clip_text = true
	hint.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	toolbar.add_child(hint)
	lines.add_child(toolbar)

	_edit = KnowledgeMarkupEdit.new()
	lines.add_child(_edit)

	_preview = RichTextLabel.new()
	_preview.bbcode_enabled = true
	_preview.fit_content = true
	_preview.selection_enabled = true
	lines.add_child(_preview)

	_picker = KnowledgeLinkPicker.new()
	lines.add_child(_picker)

	add_child(lines)
	set_bottom_editor(lines)
	add_focusable(_edit)


func _ready() -> void:
	_link_button.icon = get_theme_icon(&"Link", &"EditorIcons")
	_link_button.pressed.connect(_open_picker)
	_edit.text_changed.connect(_on_edit_text_changed)
	_edit.link_requested.connect(_open_picker)
	_edit.concept_opened.connect(_on_edit_concept_opened)
	_preview.meta_clicked.connect(_on_preview_meta_clicked)
	_picker.concept_chosen.connect(_on_picker_concept_chosen)


func _update_property() -> void:
	var markup: String = get_edited_object().get(get_edited_property())
	if _edit.text != markup:
		_is_updating = true
		_edit.text = markup
		_is_updating = false
	_show_markup(markup)


## Shows links afresh after the concepts they name have changed.
func refresh_links() -> void:
	_edit.refresh_links()
	_show_markup(_edit.text)


func _show_markup(markup: String) -> void:
	_preview.text = KnowledgeBase.format(markup)
	markup_edited.emit(markup)


func _open_picker() -> void:
	var caret := Vector2i(_edit.get_caret_draw_pos()) + Vector2i(_edit.get_screen_position())
	_picker.open_at(caret, _edit.get_selected_text())


func _on_edit_text_changed() -> void:
	if _is_updating:
		return
	emit_changed(get_edited_property(), _edit.text)
	_show_markup(_edit.text)


func _on_edit_concept_opened(concept: KnowledgeConcept) -> void:
	EditorInterface.edit_resource(concept)


func _on_preview_meta_clicked(meta: Variant) -> void:
	var concept: KnowledgeConcept = KnowledgeBase.resolve(str(meta))
	if concept != null:
		EditorInterface.edit_resource(concept)


func _on_picker_concept_chosen(concept: KnowledgeConcept, is_new: bool) -> void:
	_edit.insert_link(concept.id)
	if is_new:
		refresh_links()
