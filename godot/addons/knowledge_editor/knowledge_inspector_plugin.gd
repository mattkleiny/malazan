## Gives a concept in the inspector a description editor that links as you type, and a panel of the links into and out of it.

@tool
class_name KnowledgeInspectorPlugin extends EditorInspectorPlugin

const FILE_PREFIX: String = "concept_" ## Concept files are named this, then their id.

var _links: KnowledgeConceptLinks
var _body: KnowledgeBodyProperty


func _can_handle(object: Object) -> bool:
	return object is KnowledgeConcept


func _parse_begin(object: Object) -> void:
	var concept := object as KnowledgeConcept
	_links = KnowledgeConceptLinks.new()
	_links.concept = concept
	_body = null

	if String(concept.id).is_empty() and not concept.resource_path.is_empty():
		add_custom_control(_make_naming_button(concept))


func _parse_property(object: Object, _type: Variant.Type, name: String, _hint: PropertyHint, _hint_text: String, _usage: int, _wide: bool) -> bool:
	if name != "body":
		return false

	_body = KnowledgeBodyProperty.new()
	_body.markup_edited.connect(_links.show_markup)
	_links.concept_added.connect(_body.refresh_links.unbind(1))
	add_property_editor(name, _body)
	return true


func _parse_end(_object: Object) -> void:
	add_custom_control(_links)


## A button naming a new concept after its file: concept_sea_level.tres gives the id sea_level and the title Sea Level.
func _make_naming_button(concept: KnowledgeConcept) -> Button:
	var id := concept.resource_path.get_file().get_basename().trim_prefix(FILE_PREFIX)
	var button := Button.new()
	button.text = "Name from file: %s" % id
	button.tooltip_text = "This concept has no id, so nothing can link to it yet."
	button.pressed.connect(_name_concept.bind(concept, StringName(id)))
	return button


func _name_concept(concept: KnowledgeConcept, id: StringName) -> void:
	var title := concept.title if not concept.title.is_empty() else String(id).capitalize()
	var undo_redo := EditorInterface.get_editor_undo_redo()
	undo_redo.create_action("Name concept from its file")
	undo_redo.add_do_property(concept, &"id", id)
	undo_redo.add_do_property(concept, &"title", title)
	undo_redo.add_do_method(concept, &"notify_property_list_changed")
	undo_redo.add_undo_property(concept, &"id", concept.id)
	undo_redo.add_undo_property(concept, &"title", concept.title)
	undo_redo.add_undo_method(concept, &"notify_property_list_changed")
	undo_redo.commit_action()
