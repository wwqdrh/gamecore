# 校验脚本：秘境探索 HUD GML 结构验证（scenes/explore/ui/）
# 运行：godot --headless --path . -s res://example/demo/xiuxian/scenes/explore/ui/check_explore_ui.gd
# 职责：1) explore 系列 gml 构建校验并重生成同名 tscn（等效编辑器插件产物）
#       2) 实例化 explore HUD 验证技能栏契约（6 格 + 键位角标 + 冷却）/ 经验条
#          / 层数横幅 watch 刷新 / 离开按钮 / 鼠标穿透（自 mainhud 迁移后接管）
extends SceneTree

const DIR := "res://example/demo/xiuxian/scenes/explore/ui/"
const GML_FILES := [
	"explore.gml", "explore_skillbar.gml", "skill_item.gml",
	"dungeon/dungeon_settle.gml",
]

const KEY_SKILL := "mainhud.skill"


func _initialize() -> void:
	_run()


func _run() -> void:
	# 存档隔离：测试全程跑在专用档 _test 上，严禁污染玩家真实存档
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	var builder := GdUiBuilder.new()
	var ok := true

	# 0. 状态层基线（横幅 watch / 经验条由 XiuDungeonState/XiuCharacterState 驱动）
	XiuDungeonState.ins().reset_demo()
	XiuCharacterState.ins().reset_demo()

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

	# 2. 实例化 explore HUD 验证结构
	var hud_scene: PackedScene = load(DIR + "explore.gml.tscn")
	if hud_scene == null:
		push_error("[Check] explore.gml.tscn 加载失败")
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

	# 关键子节点
	for view_name in ["TopLayer", "TopRow", "FloorBanner", "FloorText", "LeaveBtn",
			"BottomLayer", "BottomColumn", "SkillBar", "XpBar", "SkillList"]:
		if hud.find_child(view_name, true, false) == null:
			push_error("[Check] 子节点 %s 不存在" % view_name)
			ok = false
		else:
			print("[Check] view %s OK" % view_name)

	# HUD 鼠标穿透：根与布局层 IGNORE（覆盖游戏世界不挡点击/开火/寻路）
	var passthrough: bool = hud.mouse_filter == Control.MOUSE_FILTER_IGNORE
	for layer_name in ["TopLayer", "TopRow", "BottomLayer", "BottomColumn"]:
		var layer: Control = hud.find_child(layer_name, true, false)
		if layer == null or layer.mouse_filter != Control.MOUSE_FILTER_IGNORE:
			passthrough = false
	print("[Check] mouse passthrough=%s" % passthrough)
	if not passthrough:
		push_error("[Check] explore HUD 鼠标穿透失败")
		ok = false

	# 技能栏契约：6 格 + 键位角标 1~6 + 冷却态（第 3 格 8s，其余就绪）
	var skill_list: Control = hud.find_child("SkillList", true, false)
	if skill_list == null:
		push_error("[Check] SkillList 不存在")
		load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
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

	# 经验条：状态层基线（reset 后 1 级 0 经验）→ 文本 0/100 + 填充条
	var xp_fill: Control = hud.find_child("XpFill", true, false)
	var xp_texts := 0
	for node in hud.find_children("*", "Label", true, false):
		if node.text == "0/100":
			xp_texts += 1
	if xp_fill == null or xp_texts != 1:
		push_error("[Check] 经验条填充/文本失败")
		ok = false
	else:
		print("[Check] xp bar OK (0/100 + fill)")

	# 层数横幅 watch：写状态 → 横幅文案响应式刷新（含 boss 层提示）
	var dungeon := XiuDungeonState.ins()
	dungeon.start_run("xiuxian_xiaozhai", Vector2i(10, 8))
	var banner: Label = hud.find_child("FloorText", true, false)
	await process_frame
	await process_frame
	if banner == null or "第 1/3 层" not in banner.text:
		push_error("[Check] 层数横幅未随状态刷新: %s" % (banner.text if banner else "null"))
		ok = false
	else:
		print("[Check] floor banner OK: %s" % banner.text)
	dungeon.advance_floor()
	await process_frame
	await process_frame
	if banner == null or "第 2/3 层" not in banner.text:
		push_error("[Check] 层数横幅未随换层刷新: %s" % (banner.text if banner else "null"))
		ok = false
	else:
		print("[Check] floor banner advance OK: %s" % banner.text)
	dungeon.reset_demo()

	# 技能点选上报：点第 3 格（键位 3，含冷却）-> 状态 "3"
	var slot3: Control = skill_list.get_at(2)
	skill_list.emit_signal("s_click_item", slot3)
	var skill_val: Variant = Engine.get_singleton("GDSTATE").get_state(KEY_SKILL)
	print("[Check] skill click -> state=%s" % str(skill_val))
	if str(skill_val) != "3":
		push_error("[Check] 技能点选状态上报失败")
		ok = false

	print("[Check] %s" % ("ALL GREEN" if ok else "FAILED"))
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0 if ok else 1)
