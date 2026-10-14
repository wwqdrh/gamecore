# 修仙手册（教程面板）逻辑：菜单选中 → 按 id 加载对应 .gdoc 渲染
# 目录/正文结构见 docsbook.gml；文档源文件在 docs/ 目录
extends Panel

## 菜单项 id → 文档路径（新增章节：gml 加 MenuNode + 这里补一行）
const DOC_FILES: Dictionary = {
	"basics": "res://example/demo/xiuxian/scenes/main/ui/docsbook/docs/basics.gdoc",
	"ability": "res://example/demo/xiuxian/scenes/main/ui/docsbook/docs/ability.gdoc",
}

const DEFAULT_DOC := "basics"

# 按 name 直取节点（name→节点映射由 rust 维护，免写长路径）
@onready var _menu: Control = GdUIManager.get_gml_node(self, "DocMenu")
@onready var _view: Control = GdUIManager.get_gml_node(self, "DocView")


func _ready() -> void:
	# 默认选中「基础知识」（不发信号，直接加载）
	_menu.call("select_item", DEFAULT_DOC, false)
	_load_doc(DEFAULT_DOC)


func _on_menu_selected(id: String) -> void:
	_load_doc(id)


func _load_doc(id: String) -> void:
	var path: String = DOC_FILES.get(id, "")
	if path.is_empty():
		push_warning("[DocsBook] 未登记的章节 id: %s" % id)
		return
	if not _view.call("load_doc", path):
		push_warning("[DocsBook] 文档加载失败: %s" % path)
