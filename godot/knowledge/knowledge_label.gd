## Text written in knowledge markup, whose links open tooltips while the cursor rests on them.

@icon("res://editor/icons/KnowledgeLabel.svg")
class_name KnowledgeLabel extends RichTextLabel

@export_multiline var markup: String: set = set_markup ## The text to show, in knowledge markup.


func _ready() -> void:
	bbcode_enabled = true
	meta_underlined = false
	meta_hover_started.connect(_on_meta_hover_started)
	meta_hover_ended.connect(_on_meta_hover_ended)
	text = KnowledgeBase.format(markup)


func set_markup(value: String) -> void:
	markup = value
	if is_node_ready():
		text = KnowledgeBase.format(markup)


func _on_meta_hover_started(meta: Variant) -> void:
	var concept: KnowledgeConcept = KnowledgeBase.resolve(str(meta))
	if concept == null:
		return
	KnowledgeTooltips.enter_link(concept, self)


func _on_meta_hover_ended(_meta: Variant) -> void:
	KnowledgeTooltips.exit_link(self)
