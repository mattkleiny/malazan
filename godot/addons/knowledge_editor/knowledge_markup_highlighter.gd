## Colours the links in knowledge markup: those leading to a concept, and those naming nothing known.

@tool
@icon("res://editor/icons/KnowledgeMarkupHighlighter.svg")
class_name KnowledgeMarkupHighlighter extends SyntaxHighlighter


func _get_line_syntax_highlighting(line: int) -> Dictionary:
	var edit := get_text_edit()
	var theme := EditorInterface.get_editor_theme()
	var plain := edit.get_theme_color(&"font_color")
	var known := theme.get_color(&"accent_color", &"Editor")
	var broken := theme.get_color(&"error_color", &"Editor")

	var colors: Dictionary = {0: {"color": plain}}
	for link: KnowledgeLink in KnowledgeBase.find_links(edit.get_line(line)):
		colors[link.start] = {"color": known if link.concept != null else broken}
		colors[link.end] = {"color": plain}
	return colors
