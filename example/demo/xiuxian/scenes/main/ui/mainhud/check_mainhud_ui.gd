# 校验脚本：修仙主界面 HUD GML 结构验证
# 运行：godot --headless --path . -s res://example/demo/xiuxian/scenes/main/ui/mainhud/check_mainhud_ui.gd
# 职责：1) 全部 gml 构建校验并重生成同名 tscn（等效编辑器插件产物）
#       2) 实例化主 HUD 验证 <Gml> 组合 / 三层布局 / 装备栏
#          / 功能按钮 GdState 上报与高亮互斥 / 资源加号回调 / 鼠标穿透
#       （技能栏+经验条已迁探索场景，其契约校验在 check_explore_ui.gd）
extends SceneTree

const DIR := "res://example/demo/xiuxian/scenes/main/ui/mainhud/"
const GML_FILES := [
	"mainhud.gml", "mainhud_player.gml", "mainhud_resources.gml",
	"mainhud_quest.gml", "mainhud_minimap.gml", "mainhud_menus.gml",
	"mainhud_equipbar.gml", "skill_item.gml",
]

const KEY_MENU := "mainhud.menu"


func _initialize() -> void:
	_run()


func _run() -> void:
	# 存档隔离：测试全程跑在专用档 _test 上，严禁污染玩家真实存档
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	var builder := GdUiBuilder.new()
	var ok := true

	# 0. 状态层基线：人物状态重置（HUD 等级/境界/经验条/资源栏由
	#    XiuCharacterState/XiuItemState 驱动，先重置保证跨运行确定性）
	XiuCharacterState.ins().reset_demo()
	XiuItemState.ins().reset_demo()

	# 1. 全部 gml 可构建 + 重生成 tscn
	for f in GML_FILES:
		var scene: PackedScene = builder.build_scene_file(DIR + f)
		if scene == null:
			push_error("[Check] %s 构建失败: %s" % [f, builder.last_error()])
			ok = false
			continue
		var err := ResourceSaver.save(scene, DIR + f + ".tscn")
		print("[Check] build %s save_err=%d" % [f, err])
		if err != OK:
			ok = false

	# 2. 实例化主 HUD 验证结构
	var hud_scene: PackedScene = load(DIR + "mainhud.gml.tscn")
	if hud_scene == null:
		push_error("[Check] mainhud.gml.tscn 加载失败")
		load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
		quit(1)
		return
	var holder := Control.new()
	holder.size = Vector2(1152, 648)
	root.add_child(holder)
	var hud: Control = hud_scene.instantiate()
	holder.add_child(hud)
	for i in 5:
		await process_frame

	# 信号绑定自动连接（无包装层自举）：后续点击/联动断言即验证本机制

	# <Gml> 引用的子视图
	for view_name in ["PlayerBadge", "ResourceBar", "QuestBanner", "MiniMap",
			"MenuColumn", "EquipBar", "EquipSlots", "EquipTitle"]:
		if hud.find_child(view_name, true, false) == null:
			push_error("[Check] 子视图 %s 不存在" % view_name)
			ok = false
		else:
			print("[Check] view %s OK" % view_name)

	# HUD 鼠标穿透：根与布局层 IGNORE（覆盖游戏世界不挡点击）
	var passthrough: bool = hud.mouse_filter == Control.MOUSE_FILTER_IGNORE
	for layer_name in ["TopLayer", "TopRow", "MidLayer", "MidRow", "BottomLayer", "BottomColumn"]:
		var layer: Control = hud.find_child(layer_name, true, false)
		if layer == null or layer.mouse_filter != Control.MOUSE_FILTER_IGNORE:
			passthrough = false
	print("[Check] mouse passthrough=%s" % passthrough)
	if not passthrough:
		push_error("[Check] HUD 鼠标穿透失败")
		ok = false

	# 布局/展示类节点穿透回归（builder 默认）：无信号绑定的 Control/容器/Label
	# 必须 IGNORE——无名占位 Control 曾以 STOP（filter=0）吃掉全屏世界点击
	# （点地图/世界无反应，GdShooter 开火与点击寻路的 unhandled 收不到事件）
	var layout_classes := ["Control", "Label", "Panel", "ColorRect", "TextureRect",
		"MarginContainer", "HBoxContainer", "VBoxContainer", "CenterContainer",
		"GridContainer", "PanelContainer", "HSeparator", "VSeparator"]
	var layout_bad := 0
	for node in hud.find_children("*", "Control", true, false):
		var c := node as Control
		if c == null or not layout_classes.has(c.get_class()):
			continue
		var has_binding := false
		for meta_key in c.get_meta_list():
			if str(meta_key).begins_with("__signal_"):
				has_binding = true
				break
		if not has_binding and c.mouse_filter != Control.MOUSE_FILTER_IGNORE:
			# 有 gui_input 连接的组件托管节点（如 GdUIHotbar 槽位）是
			# 有意交互的，跳过；纯布局节点 STOP 才是 bug
			if c.get_signal_connection_list("gui_input").size() > 0:
				continue
			push_error("[Check] 布局节点 %s(%s) 应为 IGNORE，实际 %d（会吞世界点击）"
				% [c.name, c.get_class(), c.mouse_filter])
			layout_bad += 1
	if layout_bad > 0:
		ok = false
	else:
		print("[Check] layout passthrough OK (all IGNORE)")

	# 功能按钮：初始高亮 修炼（状态驱动：mainhud.gd _ready 写初始值）
	var menus: Control = hud.find_child("MenuColumn", true, false)
	var btn_cultivate: Control = hud.find_child("MenuCultivate", true, false)
	var btn_bag: Control = hud.find_child("MenuBag", true, false)
	if menus == null or btn_cultivate == null or btn_bag == null:
		push_error("[Check] 功能按钮缺失")
		ok = false
	else:
		print("[Check] initial highlight cultivate_g=%.2f bag_g=%.2f" % [
			btn_cultivate.modulate.g, btn_bag.modulate.g])
		# 金色高亮 g=0.85，普通白 g=1.0：修炼应金，储物袋应白
		if btn_cultivate.modulate.g > 0.99 or btn_bag.modulate.g < 0.99:
			push_error("[Check] 初始功能高亮（修炼）失败")
			ok = false

		# 点击上报 + 高亮互斥：点储物袋 -> 状态 bag + 金色切到储物袋
		menus.call("_on_menu_pressed", btn_bag)
		var state = Engine.get_singleton("GDSTATE")
		var menu_val: Variant = state.get_state(KEY_MENU)
		print("[Check] menu click -> state=%s" % str(menu_val))
		if str(menu_val) != "bag":
			push_error("[Check] 功能按钮状态上报失败")
			ok = false
		print("[Check] menu highlight g: bag=%.2f cultivate=%.2f" % [
			btn_bag.modulate.g, btn_cultivate.modulate.g])
		# 点击后：储物袋应金，修炼应回白
		if btn_bag.modulate.g > 0.99 or btn_cultivate.modulate.g < 0.99:
			push_error("[Check] 功能高亮互斥失败")
			ok = false
		else:
			print("[Check] menu highlight switch OK")

	# 资源加号回调：@pressed 真实连接（回退到 mainhud.gd _on_res_add）
	var add_btn: Button = hud.find_child("AddStoneBtn", true, false)
	if add_btn == null or not hud.has_method("_on_res_add"):
		push_error("[Check] 资源加号回调链路失败")
		ok = false
	else:
		add_btn.emit_signal("pressed")
		print("[Check] res add callback OK")

	# 布局抽查（global_position：子节点 position 是父容器相对坐标）
	await process_frame
	# 高度不被容器拉伸：shrink_begin 子项高度 = 自身内容（回归：size_flags
	# 不支持的值静默回退 FILL 会导致组件撑满整行/整屏高）
	var height_cases := {
		"PlayerBadge": 120.0, "ResourceBar": 120.0, "QuestBanner": 120.0,
		"MiniMap": 200.0, "MenuColumn": 560.0,  # 5 按钮（含秘境）自然内容高度 ~524
	}
	for view_name in height_cases:
		var view: Control = hud.find_child(view_name, true, false)
		if view == null:
			continue
		var max_h: float = height_cases[view_name]
		print("[Check] %s height=%.0f (max %.0f)" % [view_name, view.size.y, max_h])
		if view.size.y > max_h:
			push_error("[Check] %s 被拉伸: 高度 %.0f > %.0f（size_flags 失效？）" % [
				view_name, view.size.y, max_h])
			ok = false
	var holder_origin: Vector2 = holder.global_position

	# 点选标签在父容器内（回归：点锚 top_right/bottom_wide + margin 会把
	# 文字/角标推出父容器边界外——修炼文字曾落在按钮下方 y=+84 不可见）
	var btn: Control = hud.find_child("MenuCultivate", true, false)
	var btn_origin: Vector2 = btn.global_position
	for lbl in btn.find_children("*", "Label", true, false):
		var rel: Vector2 = lbl.global_position - btn_origin
		var inside: bool = rel.x >= -1.0 and rel.y >= -1.0 \
				and rel.x + lbl.size.x <= btn.size.x + 1.0 \
				and rel.y + lbl.size.y <= btn.size.y + 1.0
		print("[Check] %s '%s' rel=%s inside=%s" % [lbl.name, lbl.text, rel, inside])
		if not inside:
			push_error("[Check] 按钮内标签 %s '%s' 超出按钮范围 %s" % [lbl.name, lbl.text, rel])
			ok = false
	var minimap: Control = hud.find_child("MiniMap", true, false)
	if minimap != null:
		var mp: Vector2 = minimap.global_position - holder_origin
		var right_gap: float = holder.size.x - (mp.x + minimap.size.x)
		print("[Check] minimap pos=%.0f,%.0f right_gap=%.0f" % [mp.x, mp.y, right_gap])
		if mp.x < holder.size.x * 0.7 or right_gap > 30.0:
			push_error("[Check] 小地图未贴右上")
			ok = false

	print("[Check] %s" % ("ALL GREEN" if ok else "FAILED"))
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0 if ok else 1)
