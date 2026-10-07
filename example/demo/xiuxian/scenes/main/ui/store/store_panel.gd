# 修仙坊市面板控制器 —— 由 store_panel.gml 的 <ui script="store_panel.gd"> 声明，
# 构建期自动挂载到 gml 根元素（StorePanel Panel）。无包装层：
# 编辑器生成的 store_panel.gml.tscn 直开运行即完整可用（信号绑定挂树自动连接）。
#
# 拆分与引用（GML 间直接 <Gml> 引用，无需脚本挂载）：
#   store_panel.gml        骨架：组合 topbar / tabs / featured / footer
#   store_topbar.gml       标题 + 双货币徽章 + 关闭按钮（@pressed 回退连到本脚本）
#   store_tabs.gml         TabContainer 四页签（每页 <Gml> 引用 store_goods.gml）
#   store_goods.gml        商品网格页（UIGrid 三列 + <script> 全量数据 + 分类过滤）
#   store_item.gd          条目自带控制器（@export 契约：品质/倒计时/划线价/售罄态）
#   store_featured.gml     右侧推荐商品卡
#   store_footer.gml       底部限购进度
#
# 本脚本只负责场景级回调（条目内回调在 store_item.gd，就近绑定）：
#   运行验证：godot --headless --path . -s res://example/demo/xiuxian/ui/store/check_store_ui.gd
extends Panel

## 面板关闭请求：独立 F6 运行时仅自身 hide()；组合进 <Modal> 时
## Modal 监听本信号整体关闭（见 ui_modal.rs 内容关闭联动）
signal s_close_requested


# ---------- 顶栏回调（store_topbar.gml 的 @pressed，就近解析回退到本脚本） ----------
func _on_close_pressed() -> void:
	print("[StorePanel] 关闭坊市")
	s_close_requested.emit()
	hide()


func _on_lingshi_plus() -> void:
	print("[StorePanel] 灵石充值入口")


func _on_lingyu_plus() -> void:
	print("[StorePanel] 灵玉充值入口")
