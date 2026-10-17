# 弹幕技能接线验收（headless 手动跑）：
#   godot --headless --path . -s res://example/demo/xiuxian/check_danmaku.gd
#
# 链路：explore_skillbar 点选/键盘 1-6 → GdState mainhud.skill（选中语义）→
#   player.gd watch 收敛选中态；秘境内左键点击 → _cast_skill（SKILL_PATTERNS
#   描述）→ GdDanmaku.cast → GdBullet 弹幕（fan/ring/...）。
# 本脚本模拟状态键上行 + 直调发射器，验证两端接线与弹幕生成数量。
extends SceneTree

var ok := true


func _initialize() -> void:
	var wd := Timer.new()
	wd.wait_time = 60.0
	wd.one_shot = true
	wd.autostart = true
	wd.timeout.connect(func() -> void: quit(2))
	root.add_child(wd)

	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()
	XiuMovieState.ins().mark_played("prologue")  # 屏蔽 MovieTrigger 并发开播

	var app: Node = load("res://example/demo/xiuxian/index.tscn").instantiate()
	root.add_child(app)
	if not await _poll(func() -> bool: return _scene_id(app) == "xiuxian_title", 300):
		_check(false, "未进入 title")
		_finish(1)
		return
	app.call("change_scene", "xiuxian_main", {}, "", false, false)
	if not await _poll(func() -> bool: return _scene_id(app) == "xiuxian_main", 600):
		_check(false, "未切到 main")
		await _finish(1)
		return
	await _wait(10)

	var player: Node = get_first_node_in_group("player")
	_check(player != null, "player 在场")
	var dm: Node = player.get("danmaku")
	_check(dm != null, "player 挂载弹幕发射器")

	# ---- 1. 直调发射器：fan 7 发 ----
	var n0 := _count_bullets()
	var cast_ok: bool = dm.cast("fan", Vector2(1, 0), {"count": 7, "spread": 70.0,
		"bullet": {"speed": 500.0, "damage": 12.0}})
	_check(cast_ok, "fan 弹幕施放成功")
	await _wait(4)
	_check(_count_bullets() - n0 >= 7, "fan 应新增 ≥7 发（实际 +%d）" % (_count_bullets() - n0))

	# ---- 2. 状态键链路：skill "3" 选中语义 ----
	# headless 无真实鼠标，左键施放路径无法端到端（红线备忘）；弹幕数量已由
	# 用例 1 覆盖。此处验证选中链路：状态键 → watch → 选中态记录，且选中后
	# 状态键保持（选中语义，不再是「按键即施放」的一次性命令）
	Engine.get_singleton("GDSTATE").set_state("mainhud.skill", "3")
	await _wait(6)
	_check(str(Engine.get_singleton("GDSTATE").get_state("mainhud.skill")) == "3",
		"技能键 3 选中后状态键保持（不消费置空）")
	_check(str(player.get("_selected_skill")) == "3",
		"玩家选中态应为 3（watch 收敛本地选中）")
	# 再按同键 = 取消选中（toggle）
	player.call("_select_skill", "3")
	await _wait(4)
	_check(str(Engine.get_singleton("GDSTATE").get_state("mainhud.skill")) == ""
		and str(player.get("_selected_skill")) == "", "再按同键应取消选中")

	# ---- 3. 螺旋弹幕（万剑归宗）总量：duration 1.2/step 0.08 × arms 3 = 45 发 ----
	# 等 fan 的施放冷却（0.5s）走完再 cast
	var cd_done: bool = await _poll(func() -> bool:
		return dm.get_cooldown_left() <= 0.0, 120)
	_check(cd_done, "施放冷却应走完")
	var total := []
	dm.s_cast.connect(func(_shape, count): total.append(int(count)))
	var spiral_ok: bool = dm.cast("spiral", Vector2(1, 0), {"duration": 1.2,
		"step": 0.08, "arms": 3, "rotate": 26.0,
		"bullet": {"speed": 460.0, "damage": 8.0}})
	_check(spiral_ok, "spiral 弹幕施放成功")
	await _wait(2)
	_check(total.size() == 1 and total[0] == 45,
		"spiral 编排总数 45（15 轮 × 3 臂），实际 %s" % str(total))

	print("[DanmakuSkill] %s" % ("ALL PASS" if ok else "FAIL"))
	_finish(0 if ok else 1)


# ---- 工具 ----

func _count_bullets() -> int:
	var out := 0
	var stack := [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n.get_class() == "GdBullet":
			out += 1
		for c in n.get_children():
			stack.append(c)
	return out


func _scene_id(scene_root: Node) -> String:
	var cur: Node = scene_root.call("get_current_scene")
	return str(cur.call("get_scene_id")) if cur != null else "<null>"


func _check(cond: bool, what: String) -> void:
	if cond:
		print("[DanmakuSkill] PASS: ", what)
	else:
		ok = false
		print("[DanmakuSkill] FAIL: ", what)


func _finish(code: int) -> void:
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	quit(code)


func _wait(frames: int) -> void:
	for i in frames:
		await process_frame


func _poll(cond: Callable, max_frames: int) -> bool:
	for i in max_frames:
		if cond.call():
			return true
		await process_frame
	return cond.call()
