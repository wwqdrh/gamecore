# 修仙角色信息面板 —— 多 GML 组合示例（壳场景：运行时加载 profile_panel.gml）
#
# 拆分与引用（GML 间直接 <Gml> 引用，无需脚本挂载）：
#   profile_panel.gml      骨架：组合 topbar / avatar / realm / skills / stats
#   profile_topbar.gml     标题 + 境界徽章 + 关闭按钮（@pressed 回退连到本脚本）
#   profile_avatar.gml     立绘占位 + 五行灵根（<script> 数据 + UIVList 数据驱动）
#   profile_realm.gml      境界突破区（62% 圆环占位 + 突破按钮）
#   profile_skills.gml     功法列表（UIVList + skill_item 条目模板）
#   skill_item.gd          条目自带控制器（@export 契约注入 + locked 锁定态）
#
# 本脚本只负责场景级回调（条目内回调在 skill_item.gd，就近绑定）：
#   运行方式：编辑器中选中本场景 F6 运行，或在主场景中实例化本场景
extends Control
#
#
#func _ready() -> void:
	## gml_file 属性已配置时 GdGmlScene 自动加载（含全部 <Gml> 引用与 <script> 数据绑定）
	#if not is_loaded():
		#load_gml("res://example/ui/profile/profile_panel.gml")
#

# ---------- 顶栏回调（profile_topbar.gml 的 @pressed，就近解析回退到场景脚本） ----------
func _on_close_pressed() -> void:
	print("[ProfilePanel] 关闭角色面板")
	hide()


# ---------- 中央突破按钮（profile_realm.gml 的 @pressed） ----------
func _on_breakthrough_pressed() -> void:
	# 突破演示：进度 62% → 100% 后由业务数据驱动境界提升，这里仅打印
	print("[ProfilePanel] 突破！炼气三层 -> 炼气四层")
