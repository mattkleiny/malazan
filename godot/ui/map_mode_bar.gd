## A row of icon buttons along the bottom of the screen, one per map mode: hovering one previews its mode, pressing one or its number key chooses it.

@icon("res://editor/icons/map_mode_bar.svg")
class_name MapModeBar extends PanelContainer

signal mode_chosen(mode: StringName) ## A button or its number key was pressed.
signal mode_previewed(mode: StringName) ## The cursor came to rest on a button.
signal preview_ended ## The cursor left the buttons.

const ICON_PATH: String = "res://ui/map_modes/%s.svg" ## Where each mode's icon lives, by its key.
const UNKNOWN_ICON: Texture2D = preload("res://ui/map_modes/unknown.svg") ## Shown for a mode with no icon of its own.
const MODE_KEYS: Array[Key] = [KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_7, KEY_8, KEY_9] ## Keys choosing the modes, in the order the buttons are laid out.
const BUTTON_SIZE: Vector2 = Vector2(48, 48)

@export var chosen_style: StyleBox ## Behind the chosen mode's button.
@export var hover_style: StyleBox ## Behind the button under the cursor.

var _empty_style: StyleBoxEmpty = StyleBoxEmpty.new()
var _group: ButtonGroup = ButtonGroup.new()
var _by_mode: Dictionary[StringName, Button] = {}

@onready var _buttons: HBoxContainer = %Buttons


func _ready() -> void:
	for placeholder: Node in _buttons.get_children():
		placeholder.free()


func _unhandled_input(event: InputEvent) -> void:
	var key := event as InputEventKey
	if key == null or not key.pressed or key.echo:
		return

	var index := MODE_KEYS.find(key.keycode)
	if index < 0 or index >= _by_mode.size():
		return

	mode_chosen.emit(_by_mode.keys()[index])


## Lays out one button per mode, in the order given, with its label as the tooltip.
func show_modes(labels: Dictionary[StringName, String]) -> void:
	for button: Button in _by_mode.values():
		button.free()
	_by_mode.clear()

	for mode: StringName in labels:
		var button := _make_button(mode, "%s  [%d]" % [labels[mode], _by_mode.size() + 1])
		_buttons.add_child(button)
		_by_mode[mode] = button


## Marks the chosen mode's button and unmarks the rest, without choosing it again.
func show_chosen(mode: StringName) -> void:
	if not _by_mode.has(mode):
		return

	# setting a button without its signal leaves the rest of its group pressed, so each is set by hand
	for other: StringName in _by_mode:
		_by_mode[other].set_pressed_no_signal(other == mode)


func _make_button(mode: StringName, tooltip: String) -> Button:
	var button := Button.new()
	button.name = String(mode).to_pascal_case()
	button.icon = _icon_for(mode)
	button.tooltip_text = tooltip
	button.toggle_mode = true
	button.button_group = _group
	button.focus_mode = Control.FOCUS_NONE
	button.custom_minimum_size = BUTTON_SIZE
	button.icon_alignment = HORIZONTAL_ALIGNMENT_CENTER
	button.add_theme_stylebox_override(&"normal", _empty_style)
	button.add_theme_stylebox_override(&"pressed", chosen_style)
	button.add_theme_stylebox_override(&"hover_pressed", chosen_style)
	button.add_theme_stylebox_override(&"hover", hover_style)
	button.pressed.connect(func() -> void: mode_chosen.emit(mode))
	button.mouse_entered.connect(func() -> void: mode_previewed.emit(mode))
	button.mouse_exited.connect(preview_ended.emit)
	return button


func _icon_for(mode: StringName) -> Texture2D:
	var path := ICON_PATH % mode
	if not ResourceLoader.exists(path):
		return UNKNOWN_ICON
	return load(path)
