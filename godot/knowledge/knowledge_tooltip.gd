## A panel describing one concept, which locks in place once it has been open long enough for the cursor to move onto it.

@icon("res://editor/icons/KnowledgeTooltip.svg")
class_name KnowledgeTooltip extends PanelContainer

@export var lock_delay: float = 0.8 ## Seconds open before it locks.
@export var locked_style: StyleBox ## The panel once locked.

var concept: KnowledgeConcept
var locked: bool = false

@onready var _title: Label = %Title
@onready var _body: KnowledgeLabel = %Body
@onready var _lock_progress: ProgressBar = %LockProgress


## Fills the panel from a concept and starts it towards locking.
func show_concept(shown: KnowledgeConcept) -> void:
	concept = shown
	_title.text = shown.title
	_body.markup = shown.body

	_lock_progress.value = 0.0
	var tween := create_tween()
	tween.tween_property(_lock_progress, ^"value", _lock_progress.max_value, lock_delay)
	tween.tween_callback(_lock)


func _lock() -> void:
	locked = true
	add_theme_stylebox_override(&"panel", locked_style)
