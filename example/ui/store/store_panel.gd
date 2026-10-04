# 修仙坊市面板 —— 多 GML 组合示例（壳场景：运行时加载 store_panel.gml）
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
#   运行方式：编辑器中选中本场景 F6 运行，或在主场景中实例化本场景
extends GdGmlScene


func _ready() -> void:
	# gml_file 属性已配置时 GdGmlScene 自动加载（含全部 <Gml> 引用与 <script> 数据绑定）
	if not is_loaded():
		load_gml("res://example/ui/store/store_panel.gml")


# ---------- 顶栏回调（store_topbar.gml 的 @pressed，就近解析回退到场景脚本） ----------
func _on_close_pressed() -> void:
	print("[StorePanel] 关闭坊市")
	hide()


func _on_lingshi_plus() -> void:
	print("[StorePanel] 灵石充值入口")


func _on_lingyu_plus() -> void:
	print("[StorePanel] 灵玉充值入口")
