# 校验脚本：修仙主界面 HUD GML 结构验证
# 运行：godot --headless --path . -s res://example/demo/xiuxian/ui/mainhud/check_mainhud_ui.gd
# 职责：1) 全部 gml 构建校验并重生成同名 tscn（等效编辑器插件产物）
#       2) 实例化主 HUD 验证 <Gml> 组合 / 三层布局 / 技能栏数据驱动（6 格 + 冷却）
#          / 功能按钮 GdState 上报与高亮互斥 / 技能点选上报 / 资源加号回调 / 鼠标穿透
extends SceneTree

const DIR := "res://example/demo/xiuxian/ui/mainhud/"
const GML_FILES := [
	"mainhud.gml", "mainhud_player.gml", "mainhud_resources.gml",
	"mainhud_quest.gml", "mainhud_minimap.gml", "mainhud_menus.gml",
	"mainhud_skillbar.gml", "skill_item.gml",
]

const KEY_MENU := "mainhud.menu"
const KEY_SKILL := "mainhud.skill"


func _initialize() -> void:
	_run()


func _run() -> void:
	var builder := GdUiBuilder.new()
	var ok := true

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
			"MenuColumn", "SkillBar", "XpBar"]:
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

	# 技能栏：6 格 + 键位角标 1~6 + 冷却态（第 3 格 8s，其余就绪）
	var skill_list: Control = hud.find_child("SkillList", true, false)
	if skill_list == null:
		push_error("[Check] SkillList 不存在")
		quit(1)
		return
	var n_slots: int = skill_list.get_child_count() - 1
	print("[Check] skill slots=%d" % n_slots)
	if n_slots != 6:
		push_error("[Check] 期望 6 技能格，实际 %d" % n_slots)
		ok = false
	for i in 6:
		var slot: Control = skill_list.get_at(i)
		var key_label: Label = slot.find_child("KeyBadge", true, false)
		var cd_label: Label = slot.find_child("CdText", true, false)
		var cd_mask: Control = slot.find_child("CdMask", true, false)
		if key_label == null or cd_label == null or cd_mask == null:
			push_error("[Check] 技能格 %d 缺少键位/冷却节点" % i)
			ok = false
			continue
		var expect_key := str(i + 1)
		var expect_cd := "8s" if i == 2 else ""
		if key_label.text != expect_key or cd_label.text != expect_cd \
				or cd_label.visible != (i == 2) or cd_mask.visible != (i == 2):
			push_error("[Check] 技能格 %d 数据错误: key=%s cd=%s vis=%s" % [
				i, key_label.text, cd_label.text, cd_label.visible])
			ok = false
	print("[Check] skill slot contracts OK (slot3 cd=8s)")

	# 经验条：320/600 文本 + 填充条
	var xp_fill: Control = hud.find_child("XpFill", true, false)
	var xp_texts := 0
	for node in hud.find_children("*", "Label", true, false):
		if node.text == "320/600":
			xp_texts += 1
	if xp_fill == null or xp_texts != 1:
		push_error("[Check] 经验条填充/文本失败")
		ok = false
	else:
		print("[Check] xp bar OK (fill + 320/600)")

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

	# 技能点选上报：点第 3 格（键位 3，含冷却）-> 状态 "3"
	var slot3: Control = skill_list.get_at(2)
	skill_list.emit_signal("s_click_item", slot3)
	var skill_val: Variant = Engine.get_singleton("GDSTATE").get_state(KEY_SKILL)
	print("[Check] skill click -> state=%s" % str(skill_val))
	if str(skill_val) != "3":
		push_error("[Check] 技能点选状态上报失败")
		ok = false

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
		"MiniMap": 200.0, "MenuColumn": 450.0, "SkillBar": 150.0,
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
	var skillbar: Control = hud.find_child("SkillBar", true, false)
	if minimap != null:
		var mp: Vector2 = minimap.global_position - holder_origin
		var right_gap: float = holder.size.x - (mp.x + minimap.size.x)
		print("[Check] minimap pos=%.0f,%.0f right_gap=%.0f" % [mp.x, mp.y, right_gap])
		if mp.x < holder.size.x * 0.7 or right_gap > 30.0:
			push_error("[Check] 小地图未贴右上")
			ok = false
	if skillbar != null:
		var sp: Vector2 = skillbar.global_position - holder_origin
		var center_off: float = absf((sp.x + skillbar.size.x / 2.0) - holder.size.x / 2.0)
		var bottom_gap: float = holder.size.y - (sp.y + skillbar.size.y)
		print("[Check] skillbar center_off=%.0f bottom_gap=%.0f" % [center_off, bottom_gap])
		if center_off > 30.0 or bottom_gap > 40.0:
			push_error("[Check] 技能栏未居中贴底")
			ok = false

	# 技能格键位角标在格子内（bottom_wide + 负 top margin）
	var slot0: Control = skill_list.get_at(0)
	var key0: Label = slot0.find_child("KeyBadge", true, false)
	var rel_key: Vector2 = key0.global_position - slot0.global_position
	print("[Check] KeyBadge rel=%s (slot %s)" % [rel_key, slot0.size])
	if rel_key.y < 0.0 or rel_key.y + key0.size.y > slot0.size.y + 1.0 \
			or rel_key.x < 0.0 or rel_key.x + key0.size.x > slot0.size.x + 1.0:
		push_error("[Check] 键位角标超出技能格范围")
		ok = false

	print("[Check] %s" % ("ALL GREEN" if ok else "FAILED"))
	quit(0 if ok else 1)
