# 秘境探索 HUD 控制器 —— 由 explore.gml 的 <ui script="explore.gd"> 声明，
# 无壳模式：gml 根 Control 被提升合并，脚本挂 explore.gml.tscn 根节点
# （index.tscn 中实例名 ExploreUI）。直开运行即完整可用（信号绑定挂树自动连接）。
#
# 拆分与引用（GML 间直接 <Gml> 引用，无需脚本挂载）：
#   explore.gml           骨架：层数横幅 / 离开按钮 / 结算弹窗
#   explore_skillbar.gml+gd 底部技能栏 + 灵气经验条（自 mainhud 迁移）
#   skill_item.gml+gd     技能格条目契约（图标/键位/冷却）
#
# 职责：
#   · 层数横幅：watch XiuDungeonState（floor/total_floors/status）响应式刷新
#   · 离开秘境按钮：经组件表调 ExploreManager.abandon()（跨组件查找规范）
extends Control

var _dungeon_bean: XiuDungeonState


func _ready() -> void:
	# 自适应分辨率（headless 测试自动跳过；GdSceneRoot 流程已应用，此处覆盖直开组合根路径）
	GdDisplayFit.apply_display_fit(false)
	# HUD 覆盖游戏世界：根与布局容器鼠标穿透（不影响子交互节点）
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	for layer in ["TopLayer", "TopRow", "BottomLayer", "BottomColumn"]:
		var c := find_child(layer, true, false) as Control
		if c:
			c.mouse_filter = Control.MOUSE_FILTER_IGNORE
	# 层数横幅（watch 注册即回调当前值；中途放弃/通关回空也刷新文案）。
	# 红线：watch 按回调哈希去重（同一 Callable 重复注册被静默忽略，仅首个 key 生效）；
	# 多 key 共用一个处理函数必须各包一层独立 lambda（分号路径是嵌套属性语法，非多 key）
	_dungeon_bean = XiuDungeonState.ins()
	_dungeon_bean.watch("floor", func(v = null, m = null): _on_floor_changed())
	_dungeon_bean.watch("total_floors", func(v = null, m = null): _on_floor_changed())
	_dungeon_bean.watch("status", func(v = null, m = null): _on_floor_changed())
	# 结算弹窗遮罩：灰色透明（默认黑 55% 太沉）。运行期覆盖组件导出值——
	# 不走 <theme>（builder 主题默认值在属性之后应用，会连带改写全场景字号）
	var modal: Control = find_child("DungeonSettleModal", true, false)
	if modal != null:
		modal.set("overlay_color", Color(0.54, 0.54, 0.54, 0.4))


## 状态下行：层数横幅文案（running → 第 X/N 层；boss 层加妖王提示）
func _on_floor_changed(_value: Variant = null, _metas: Variant = null) -> void:
	var lbl: Label = find_child("FloorText", true, false)
	if lbl == null:
		return
	if str(_dungeon_bean.status) == XiuDungeonState.STATUS_RUNNING:
		var tag := " · 妖王镇守！" if _dungeon_bean.is_boss_floor() else ""
		lbl.text = "落霞秘境 · 第 %d/%d 层%s" % [
			int(_dungeon_bean.floor), int(_dungeon_bean.total_floors), tag]
	else:
		lbl.text = "落霞秘境"


# ---------- 离开秘境（explore.gml LeaveBtn 的 @pressed，就近解析回退到本脚本） ----------
## 经组件表调 ExploreManager.abandon()：进度作废 → 回主场景（送回入口）
func _on_leave_pressed(_btn: Control) -> void:
	var scene_root: Node = _find_scene_root()
	if scene_root == null:
		return
	var mgr: Node = scene_root.get_component("ExploreManager")
	if mgr != null and mgr.has_method("abandon"):
		mgr.abandon()


## 场景根（GdSceneRoot）：经 GDCORE 全局节点表获取（同 explore_manager.gd）
func _find_scene_root() -> Node:
	if not Engine.has_singleton("GDCORE"):
		return null
	var node: Object = Engine.get_singleton("GDCORE").get_global_node("default")
	return node if node is Node else null
