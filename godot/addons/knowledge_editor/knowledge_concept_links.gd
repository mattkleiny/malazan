## The links out of a concept and into it, each opening the concept it names, with a way to make the concept a broken link names.

@tool
@icon("res://editor/icons/KnowledgeConceptLinks.svg")
class_name KnowledgeConceptLinks extends VBoxContainer

signal concept_added(concept: KnowledgeConcept) ## A concept was made for a link that named nothing known.

const CONCEPT_ICON: Texture2D = preload("res://editor/icons/KnowledgeConcept.svg")
const CONCEPT_KIND: StringName = &"concept"

var concept: KnowledgeConcept

var _outgoing: HFlowContainer
var _incoming: HFlowContainer


func _init() -> void:
	add_child(_make_heading("Links to"))
	_outgoing = HFlowContainer.new()
	add_child(_outgoing)
	add_child(_make_heading("Linked from"))
	_incoming = HFlowContainer.new()
	add_child(_incoming)


func _ready() -> void:
	if concept != null:
		show_markup(concept.body)


## Lists the links in a description, and the concepts linking to this one.
func show_markup(markup: String) -> void:
	_clear(_outgoing)
	var seen: Dictionary[StringName, bool] = {}
	for link: KnowledgeLink in KnowledgeBase.find_links(markup):
		if link.kind != CONCEPT_KIND or seen.has(link.id):
			continue
		seen[link.id] = true
		_outgoing.add_child(_make_concept_button(link.concept) if link.concept != null else _make_create_button(link.id))
	if _outgoing.get_child_count() == 0:
		_outgoing.add_child(_make_none_label())

	_clear(_incoming)
	if concept != null and not String(concept.id).is_empty():
		for other: KnowledgeConcept in KnowledgeBase.get_concepts_linking_to(concept.id):
			if other != concept:
				_incoming.add_child(_make_concept_button(other))
	if _incoming.get_child_count() == 0:
		_incoming.add_child(_make_none_label())


func _clear(flow: HFlowContainer) -> void:
	for child: Node in flow.get_children():
		flow.remove_child(child)
		child.queue_free()


func _make_heading(text: String) -> Label:
	var heading := Label.new()
	heading.text = text
	heading.theme_type_variation = &"HeaderSmall"
	return heading


func _make_none_label() -> Label:
	var none := Label.new()
	none.text = "None"
	none.modulate.a = 0.5
	return none


func _make_concept_button(shown: KnowledgeConcept) -> Button:
	var button := Button.new()
	button.text = shown.title if not shown.title.is_empty() else String(shown.id)
	button.icon = CONCEPT_ICON
	button.tooltip_text = "Open %s" % shown.id
	button.pressed.connect(EditorInterface.edit_resource.bind(shown))
	return button


func _make_create_button(id: StringName) -> Button:
	var button := Button.new()
	button.text = String(id)
	button.icon = EditorInterface.get_editor_theme().get_icon(&"Add", &"EditorIcons")
	button.add_theme_color_override(&"font_color", EditorInterface.get_editor_theme().get_color(&"error_color", &"Editor"))
	button.tooltip_text = "No concept has the id %s. Press to make one." % id
	button.pressed.connect(_on_create_pressed.bind(id))
	return button


func _on_create_pressed(id: StringName) -> void:
	var made: KnowledgeConcept = KnowledgeBase.add_concept(id, String(id).capitalize())
	if made == null:
		return
	EditorInterface.get_resource_filesystem().update_file(made.resource_path)
	concept_added.emit(made)
	show_markup(concept.body)
