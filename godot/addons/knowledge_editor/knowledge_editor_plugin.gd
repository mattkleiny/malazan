## Writes concepts in the inspector: links them as you type, follows links between them, and makes the concepts links name.

@tool
extends EditorPlugin

var _inspector_plugin: KnowledgeInspectorPlugin


func _enter_tree() -> void:
	_inspector_plugin = KnowledgeInspectorPlugin.new()
	add_inspector_plugin(_inspector_plugin)
	EditorInterface.get_resource_filesystem().filesystem_changed.connect(_on_filesystem_changed)
	resource_saved.connect(_on_resource_saved)


func _exit_tree() -> void:
	remove_inspector_plugin(_inspector_plugin)
	EditorInterface.get_resource_filesystem().filesystem_changed.disconnect(_on_filesystem_changed)
	resource_saved.disconnect(_on_resource_saved)


func _on_filesystem_changed() -> void:
	KnowledgeBase.reload()


func _on_resource_saved(resource: Resource) -> void:
	if resource is KnowledgeConcept:
		KnowledgeBase.reload()
