## The map editor's brush: which tool is in hand, and how wide and strong it is.

@tool
@icon("res://editor/icons/MapBrushPanel.svg")
class_name MapBrushPanel extends VBoxContainer

signal tool_changed(tool: StringName) ## Another tool was taken up.

var current_tool: StringName = &"select" ## The tool in hand: select, paint, raise, lower, smooth, flatten or label.

var _group: ButtonGroup = ButtonGroup.new()

@onready var _tools: HBoxContainer = %Tools
@onready var _radius: HSlider = %Radius
@onready var _strength: HSlider = %Strength


func _ready() -> void:
	for button: Button in _tools.get_children():
		var named := StringName(button.name.to_snake_case())
		button.toggle_mode = true
		button.button_group = _group
		button.button_pressed = named == current_tool
		button.pressed.connect(func() -> void: _take_up(named))


## Width of the brush from its centre (map units).
func get_radius() -> float:
	return _radius.value


## How hard the brush shapes the ground (0..1).
func get_strength() -> float:
	return _strength.value


func _take_up(named: StringName) -> void:
	if named == current_tool:
		return

	current_tool = named
	tool_changed.emit(named)
