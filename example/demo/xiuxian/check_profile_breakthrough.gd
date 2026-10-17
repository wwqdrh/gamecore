# 校验脚本：角色面板突破流程（点击有可见反馈 + 真实材料判定）
# 运行：godot --headless --path . -s res://example/demo/xiuxian/check_profile_breakthrough.gd
# 背景（2026-10-17 bug）：斗之气十段点突破"没反应"——失败分支只 print 控制台；
# 材料徽章把背包首个 pill（回春丹）当突破材料展示，误导玩家以为回春丹可突破。
# 修复后：BreakthroughHint 行内提示四分支全覆盖；徽章显示真实材料 have/need。
extends SceneTree

var _fails := 0


func check(cond: bool, msg: String) -> void:
	if cond:
		print("[BTCheck] PASS ", msg)
	else:
		_fails += 1
		printerr("[BTCheck] FAIL ", msg)


func _initialize() -> void:
	_run()


func _run() -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	var character = XiuCharacterState.ins()
	var item = XiuItemState.ins()
	XiuGameState.ins().reset_demo()

	# 复现用户情境：斗之气十段 + 回春丹（治疗丹药，非突破材料）
	item.add_item("pill_hp", 2)
	var gained: int = character.add_exp(6000)
	check(character.level == 10 and character.is_rank_full(),
		"add_exp(6000) 应达斗之气十段阶满（got lv=%d gained=%d）" % [character.level, gained])
	check(int(character.exp) == 1000, "阶满经验应封顶 1000（got %d）" % int(character.exp))

	# 实例化角色面板（gml 运行时加载，控制器挂在根节点）
	var panel_scene: PackedScene = load(
		"res://example/demo/xiuxian/scenes/main/ui/profile/profile_panel.gml.tscn")
	check(panel_scene != null, "profile_panel.gml.tscn 应可加载")
	if panel_scene == null:
		load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
		quit(1)
		return
	var holder := Control.new()
	holder.size = Vector2(1152, 648)
	root.add_child(holder)
	var panel: Control = panel_scene.instantiate()
	holder.add_child(panel)
	for i in 5:
		await process_frame

	var hint: Label = panel.find_child("BreakthroughHint", true, false)
	var chip: Label = panel.find_child("MaterialText", true, false)
	check(hint != null, "BreakthroughHint 提示行应存在")
	check(chip != null, "MaterialText 材料徽章应存在")

	# 1. 材料徽章显示真实突破材料（不是回春丹）
	check(chip != null and str(chip.text).contains("聚气散") and str(chip.text).contains("0/3"),
		"材料徽章应显示 聚气散 0/3（got %s）" % (chip.text if chip else "<nil>"))
	check(chip != null and not str(chip.text).contains("回春丹"),
		"材料徽章不应再显示回春丹（got %s）" % (chip.text if chip else "<nil>"))

	# 2. 十段 + 修为圆满 + 无聚气散 → 可见提示材料不足（走真实按钮信号链路）
	var btn: Button = panel.find_child("BreakthroughBtn", true, false)
	check(btn != null and btn.pressed.get_connections().size() > 0,
		"BreakthroughBtn @pressed 应已连接（tscn 内嵌 profile_panel.gd）")
	if btn != null:
		btn.pressed.emit()
	check(str(hint.text).contains("聚气散") and str(hint.text).contains("(0/3)"),
		"缺材料点击应提示 聚气散(0/3)（got %s）" % hint.text)
	check(character.level == 10, "材料不足突破不应升阶")

	# 3. 十段 + 修为未圆满（清掉阶满经验）→ 可见提示修为未圆满
	character.update("exp", 200, {}, false)
	if btn != null:
		btn.pressed.emit()
	check(str(hint.text).contains("修为尚未圆满") and str(hint.text).contains("200/1000"),
		"修为未满点击应提示 经验 200/1000（got %s）" % hint.text)
	check(character.level == 10, "修为未满突破不应升阶")

	# 4. 备齐聚气散 → 突破成功可见提示 + 进斗者一段 + 材料被扣
	item.add_item("break_qizhe", 3)
	await process_frame  # 徽章刷新走 call_deferred（重入红线），等一帧
	check(str(chip.text).contains("聚气散 3/3"),
		"备料后徽章应显示 聚气散 3/3（got %s）" % chip.text)
	character.update("exp", 1000, {}, false)
	if btn != null:
		btn.pressed.emit()
	check(str(hint.text).contains("突破成功"),
		"突破成功应提示（got %s）" % hint.text)
	check(character.level == 11 and int(character.exp) == 0,
		"突破后应进斗者一段经验清零（got lv=%d exp=%d）" % [character.level, int(character.exp)])
	check(character.get_realm_title() == "斗者一段",
		"突破后头衔应为 斗者一段（got %s）" % character.get_realm_title())
	check(item.get_count("break_qizhe") == 0, "突破应扣除聚气散 x3")
	check(item.get_count("pill_hp") == 2, "回春丹不应被突破消耗")

	# 5. 阶内未满分支：清回低级 + 少量经验 → 可见提示修炼进度
	character.update("level", 3, {}, false)
	character.update("exp", 50, {}, false)
	if btn != null:
		btn.pressed.emit()
	check(str(hint.text).contains("修炼进度") and str(hint.text).contains("未圆满"),
		"阶内未满点击应提示修炼进度（got %s）" % hint.text)

	holder.queue_free()
	await process_frame
	print("[BTCheck] %s（fails=%d）" % ["ALL GREEN" if _fails == 0 else "FAILED", _fails])
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(0 if _fails == 0 else 1)
