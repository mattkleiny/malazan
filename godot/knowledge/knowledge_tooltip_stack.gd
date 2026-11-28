## The tooltips open on screen, each opened from a link in the one before it.

@icon("res://editor/icons/KnowledgeTooltipStack.svg")
class_name KnowledgeTooltipStack extends CanvasLayer

const TOOLTIP_SCENE: PackedScene = preload("res://knowledge/knowledge_tooltip.tscn")
const CURSOR_OFFSET: Vector2 = Vector2(18, 18) ## How far from the cursor a tooltip opens.

@export var show_delay: float = 0.35 ## Seconds the cursor rests on a link before its tooltip opens.
@export var leave_grace: float = 0.3 ## Seconds a locked tooltip outlasts the cursor leaving it.

var _tooltips: Array[KnowledgeTooltip] = []
var _hovered: KnowledgeConcept
var _hovered_source: Control
var _hovered_depth: int = 0 # how many tooltips deep the hovered link sits
var _hover_time: float = 0.0
var _away_time: float = 0.0


func _process(delta: float) -> void:
	# a source hidden or freed under the cursor never reports the cursor leaving
	if _hovered != null and not (is_instance_valid(_hovered_source) and _hovered_source.is_visible_in_tree()):
		_hovered = null

	if _hovered != null and _tooltips.size() == _hovered_depth:
		_hover_time += delta
		if _hover_time >= show_delay:
			_open(_hovered)

	_close_abandoned(delta)


## Notes the cursor resting on a link to a concept, shown by a control inside or outside the open tooltips.
func enter_link(concept: KnowledgeConcept, source: Control) -> void:
	_hovered = concept
	_hovered_source = source
	_hovered_depth = _depth_of(source)
	_hover_time = 0.0

	if _tooltips.size() > _hovered_depth and _tooltips[_hovered_depth].concept != concept:
		_close_from(_hovered_depth)


## Notes the cursor leaving the link it rested on in this control.
func exit_link(source: Control) -> void:
	if source != _hovered_source:
		return
	_hovered = null
	_hovered_source = null


func _open(concept: KnowledgeConcept) -> void:
	var tooltip: KnowledgeTooltip = TOOLTIP_SCENE.instantiate()
	add_child(tooltip)
	tooltip.show_concept(concept)
	_tooltips.append(tooltip)

	# its height is only known once its text is laid out, so it is placed again whenever that changes
	var cursor := get_viewport().get_mouse_position()
	tooltip.resized.connect(_place.bind(tooltip, cursor))
	_place(tooltip, cursor)


## Puts a tooltip beside the cursor, on whichever side keeps it on screen.
func _place(tooltip: KnowledgeTooltip, cursor: Vector2) -> void:
	var screen := get_viewport().get_visible_rect().size
	var corner := cursor + CURSOR_OFFSET
	if corner.x + tooltip.size.x > screen.x:
		corner.x = cursor.x - CURSOR_OFFSET.x - tooltip.size.x
	if corner.y + tooltip.size.y > screen.y:
		corner.y = cursor.y - CURSOR_OFFSET.y - tooltip.size.y
	tooltip.position = corner.clamp(Vector2.ZERO, (screen - tooltip.size).max(Vector2.ZERO))


## Closes the tooltips the cursor has left: at once while unlocked, after a moment's grace once locked.
func _close_abandoned(delta: float) -> void:
	var kept := _hovered_depth + 1 if _hovered != null else 0
	var cursor := get_viewport().get_mouse_position()
	for depth: int in _tooltips.size():
		if _tooltips[depth].get_global_rect().has_point(cursor):
			kept = maxi(kept, depth + 1)

	if kept >= _tooltips.size():
		_away_time = 0.0
		return

	_away_time += delta
	if _tooltips[kept].locked and _away_time < leave_grace:
		return
	_close_from(kept)


func _close_from(depth: int) -> void:
	for tooltip: KnowledgeTooltip in _tooltips.slice(depth):
		tooltip.queue_free()
	_tooltips.resize(depth)
	_away_time = 0.0


## How many tooltips deep a control sits: zero outside them all.
func _depth_of(source: Control) -> int:
	for depth: int in _tooltips.size():
		if _tooltips[depth].is_ancestor_of(source):
			return depth + 1
	return 0
