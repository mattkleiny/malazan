## Edits the map a TerrainView draws: paints locations and shapes the ground in the 3D viewport, and edits the definitions from a dock.

@tool
extends EditorPlugin

const DOCK_SCENE: PackedScene = preload("res://addons/map_editor/map_editor_dock.tscn")
const MAP_FILES: Array[String] = ["definitions.csv", "locations.png", "heightmap.png"] ## The files a map is saved to, in its folder.
const DAB_SPACING: float = 0.3 ## Gap between dabs along a stroke, as a share of the brush's radius.
const OUTLINE_POINTS: int = 48
const OUTLINE_COLOR: Color = Color(1.0, 0.85, 0.4, 0.9)

var _terrain: TerrainView
var _dock: MapEditorDock
var _camera: Camera3D
var _brush_point: Vector3 = Vector3(NAN, NAN, NAN) ## Where the brush rests on the map, in the terrain's own space.
var _last_dab: Vector3 = Vector3(NAN, NAN, NAN)
var _flatten_level: float = 0.0
var _is_stroking: bool = false


func _enter_tree() -> void:
	_dock = DOCK_SCENE.instantiate()
	_dock.tool_changed.connect(_on_dock_tool_changed)
	_dock.location_chosen.connect(_on_dock_location_chosen)
	_dock.add_requested.connect(_on_dock_add_requested)
	_dock.field_edited.connect(_on_dock_field_edited)
	_dock.save_requested.connect(_on_dock_save_requested)
	add_control_to_dock(DOCK_SLOT_RIGHT_UL, _dock)
	_dock.hide()


func _exit_tree() -> void:
	remove_control_from_docks(_dock)
	_dock.queue_free()


func _handles(object: Object) -> bool:
	return object is TerrainView


func _edit(object: Object) -> void:
	_terrain = object as TerrainView
	if _terrain == null:
		return

	_dock.show_locations(_terrain.get_location_names(), _terrain.get_terrains())
	_show_tool(_dock.get_tool())


func _make_visible(visible: bool) -> void:
	_dock.visible = visible
	if not visible:
		_brush_point = Vector3(NAN, NAN, NAN)
		update_overlays()


func _forward_3d_gui_input(camera: Camera3D, event: InputEvent) -> int:
	if not is_instance_valid(_terrain):
		return AFTER_GUI_INPUT_PASS

	_camera = camera
	_terrain.set_view_camera(camera)

	var motion := event as InputEventMouseMotion
	if motion != null:
		_move_brush(motion.position)
		return AFTER_GUI_INPUT_STOP if _is_stroking else AFTER_GUI_INPUT_PASS

	var button := event as InputEventMouseButton
	if button == null or button.button_index != MOUSE_BUTTON_LEFT:
		return AFTER_GUI_INPUT_PASS

	if button.pressed:
		return _press(button)

	if _is_stroking:
		_finish_stroke()
		return AFTER_GUI_INPUT_STOP
	return AFTER_GUI_INPUT_PASS


func _forward_3d_draw_over_viewport(overlay: Control) -> void:
	if not is_instance_valid(_camera) or not _brush_point.is_finite() or _dock.get_tool() == &"select":
		return

	var radius := _dock.get_radius()
	var points := PackedVector2Array()
	for index in OUTLINE_POINTS + 1:
		var angle := TAU * index / OUTLINE_POINTS
		var x := _brush_point.x + cos(angle) * radius
		var z := _brush_point.z + sin(angle) * radius
		var local := Vector3(x, _terrain.get_surface_height(x, z), z)
		var world := _terrain.global_transform * local
		if _camera.is_position_behind(world):
			return
		points.append(_camera.unproject_position(world))

	overlay.draw_polyline(points, OUTLINE_COLOR, 2.0, true)


func _get_unsaved_status(for_scene: String) -> String:
	if not for_scene.is_empty() or not is_instance_valid(_terrain) or not _terrain.has_unsaved_changes():
		return ""
	return "The map has unsaved changes. Save them before closing?"


func _save_external_data() -> void:
	if is_instance_valid(_terrain) and _terrain.has_unsaved_changes():
		_save_map()


## Starts what the chosen tool does on a press: choosing a location, or a stroke.
func _press(button: InputEventMouseButton) -> int:
	var tool := _dock.get_tool()
	var location := _terrain.pick_location(_camera, button.position)

	if tool == &"select" or (tool == &"paint" and button.alt_pressed):
		if location >= 0:
			_choose_location(location)
		return AFTER_GUI_INPUT_STOP

	var point := _terrain.pick_point(_camera, button.position)
	if not point.is_finite():
		return AFTER_GUI_INPUT_PASS

	_flatten_level = _terrain.get_ground_level(point.x, point.z)
	_terrain.begin_stroke()
	_is_stroking = true
	_last_dab = Vector3(NAN, NAN, NAN)
	_dab_towards(point)
	return AFTER_GUI_INPUT_STOP


func _move_brush(screen_position: Vector2) -> void:
	_brush_point = _terrain.pick_point(_camera, screen_position)
	update_overlays()

	if _is_stroking and _brush_point.is_finite():
		_dab_towards(_brush_point)


## Dabs along the way from the last dab to `point`, so a quick stroke leaves no gaps.
func _dab_towards(point: Vector3) -> void:
	var spacing := maxf(_dock.get_radius() * DAB_SPACING, 0.5)
	if not _last_dab.is_finite():
		_dab(point)
		return

	var flat_from := Vector2(_last_dab.x, _last_dab.z)
	var flat_to := Vector2(point.x, point.z)
	var distance := flat_from.distance_to(flat_to)
	if distance < spacing:
		return

	var steps := int(distance / spacing)
	for step in range(1, steps + 1):
		var spot := flat_from.lerp(flat_to, float(step) / steps)
		_dab(Vector3(spot.x, 0.0, spot.y))


func _dab(point: Vector3) -> void:
	_last_dab = point
	var tool := _dock.get_tool()
	if tool == &"paint":
		_terrain.paint_location_at(point.x, point.z, _dock.get_radius(), _dock.get_location())
		return

	_terrain.sculpt_at(point.x, point.z, _dock.get_radius(), _dock.get_strength(), tool, _flatten_level)


func _finish_stroke() -> void:
	_is_stroking = false
	var edit := _terrain.end_stroke()
	if edit < 0:
		return

	var undo_redo := get_undo_redo()
	undo_redo.create_action("Paint locations" if _dock.get_tool() == &"paint" else "Shape the ground")
	undo_redo.add_do_method(_terrain, &"apply_edit", edit, true)
	undo_redo.add_undo_method(_terrain, &"apply_edit", edit, false)
	undo_redo.commit_action(false)
	_dock.show_status("Unsaved changes")


func _choose_location(location: int) -> void:
	_terrain.select_location(location)
	_dock.show_location(location, _fields_of(location))


func _fields_of(location: int) -> Dictionary[StringName, String]:
	var fields: Dictionary[StringName, String] = {}
	for field: StringName in MapLocationInspector.FIELDS:
		fields[field] = _terrain.get_location_field(location, field)
	return fields


## Shows the map mode that suits a tool while it is in hand.
func _show_tool(tool: StringName) -> void:
	if tool == &"paint":
		_terrain.preview_map_mode(&"locations")
	else:
		_terrain.clear_map_mode_preview()
	update_overlays()


func _save_map() -> void:
	var error := _terrain.save_map()
	if not error.is_empty():
		_dock.show_status(error)
		push_error("Map Editor: " + error)
		return

	var filesystem := EditorInterface.get_resource_filesystem()
	for file: String in MAP_FILES:
		filesystem.update_file(_terrain.map_directory.path_join(file))
	_dock.show_status("Saved")


func _refresh_dock(location: int) -> void:
	_dock.show_locations(_terrain.get_location_names(), _terrain.get_terrains())
	_choose_location(location)


func _on_dock_tool_changed(tool: StringName) -> void:
	if is_instance_valid(_terrain):
		_show_tool(tool)


func _on_dock_location_chosen(location: int) -> void:
	if is_instance_valid(_terrain):
		_choose_location(location)


func _on_dock_add_requested() -> void:
	if not is_instance_valid(_terrain):
		return

	var like := maxi(_dock.get_location(), 0)
	var name := "New location %d" % _terrain.get_location_names().size()
	var added := _terrain.add_location(name, like)
	if added < 0:
		_dock.show_status("Cannot add a location")
		return

	var undo_redo := get_undo_redo()
	undo_redo.create_action("Add location")
	undo_redo.add_do_method(_terrain, &"add_location", name, like)
	undo_redo.add_do_method(self, &"_refresh_dock", added)
	undo_redo.add_undo_method(_terrain, &"remove_last_location")
	undo_redo.add_undo_method(self, &"_refresh_dock", like)
	undo_redo.commit_action(false)
	_refresh_dock(added)
	_dock.show_status("Unsaved changes")


func _on_dock_field_edited(location: int, field: StringName, value: String) -> void:
	if not is_instance_valid(_terrain):
		return

	var before := _terrain.get_location_field(location, field)
	if before == value:
		return

	var error := _terrain.set_location_field(location, field, value)
	if not error.is_empty():
		_dock.show_status(error)
		_dock.show_location(location, _fields_of(location))
		return

	var undo_redo := get_undo_redo()
	undo_redo.create_action("Edit location %s" % field)
	undo_redo.add_do_method(_terrain, &"set_location_field", location, field, value)
	undo_redo.add_do_method(self, &"_refresh_dock", location)
	undo_redo.add_undo_method(_terrain, &"set_location_field", location, field, before)
	undo_redo.add_undo_method(self, &"_refresh_dock", location)
	undo_redo.commit_action(false)
	_refresh_dock(location)
	_dock.show_status("Unsaved changes")


func _on_dock_save_requested() -> void:
	if is_instance_valid(_terrain):
		_save_map()
