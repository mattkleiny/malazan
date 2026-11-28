## The map editor's view of the chosen map label: its text, size, angle and ink, and taking it off the map.

@tool
@icon("res://editor/icons/MapLabelInspector.svg")
class_name MapLabelInspector extends VBoxContainer

signal label_edited(label: MapLabel) ## The chosen label was changed and committed; carries the label as it should now be.
signal remove_requested ## The remove button was pressed.

var _shown: MapLabel

@onready var _hint: Label = %LabelHint
@onready var _fields: GridContainer = %LabelFields
@onready var _text: LineEdit = %LabelText
@onready var _size: SpinBox = %LabelSize
@onready var _angle: SpinBox = %LabelAngle
@onready var _is_water: CheckBox = %LabelIsWater
@onready var _remove_button: Button = %RemoveLabelButton


func _ready() -> void:
	_text.text_submitted.connect(func(_text: String) -> void: _commit())
	_text.focus_exited.connect(_commit)
	_size.value_changed.connect(func(_value: float) -> void: _commit())
	_angle.value_changed.connect(func(_value: float) -> void: _commit())
	_is_water.toggled.connect(func(_on: bool) -> void: _commit())
	_remove_button.pressed.connect(_on_remove_button_pressed)
	show_label(null)


## Shows a label's text, size, angle and ink, or nothing chosen for null.
func show_label(label: MapLabel) -> void:
	_shown = null
	_hint.visible = label == null
	_fields.visible = label != null
	_remove_button.disabled = label == null
	if label == null:
		return

	_text.text = label.text
	_size.set_value_no_signal(label.size)
	_angle.set_value_no_signal(label.angle)
	_is_water.set_pressed_no_signal(label.is_water)
	_shown = label


func _commit() -> void:
	if _shown == null:
		return

	var edited := MapLabel.new()
	edited.text = _text.text.strip_edges()
	edited.position = _shown.position
	edited.size = _size.value
	edited.angle = _angle.value
	edited.is_water = _is_water.button_pressed
	var is_same_size := absf(edited.size - _shown.size) < 0.01
	var is_same_angle := absf(edited.angle - _shown.angle) < 0.01
	if edited.text == _shown.text and is_same_size and is_same_angle and edited.is_water == _shown.is_water:
		return

	label_edited.emit(edited)


func _on_remove_button_pressed() -> void:
	remove_requested.emit()
