# GdNpcGreet 验收 —— 玩家靠近主动问候（浮动对话）
#
# 运行：perl -e 'alarm 60; exec @ARGV' godot --headless --path . \
#   -s res://test/check_npc_greet.gd
#
# 覆盖（fixture 鸭子宿主）：
#   A. 边沿触发：玩家远处无问候，靠近触发一次，内容来自 lines
#   B. 冷却：退出再进入不重复问候
#   C. resolve_greeting 鸭子回调优先（重逢问候）
#   D. 宿主忙（赶路/对话中）不触发；忙结束+重新靠近才触发
#   E. 宿主无 show_bubble → 自建 Label 显示并按 line_secs 收起
extends SceneTree

var ok := true


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[NpcGreet] " + msg)
		ok = false


func _initialize() -> void:
	_run()


# ---- fixture：鸭子宿主（有 show_bubble，捕获问候文案） ----
class GreetHost extends Node2D:
	var paused := false
	var path_active := false
	var bubbles: Array = []
	var greeting := ""  # 非空时经 resolve_greeting 返回
	func is_paused() -> bool:
		return paused
	func set_paused(p: bool) -> void:
		paused = p
	func is_grid_path_active() -> bool:
		return path_active
	func show_bubble(text: String, _secs := 3.0) -> void:
		bubbles.append(text)
	func resolve_greeting() -> String:
		return greeting

# ---- fixture：无 show_bubble 的朴素宿主（走自建 Label 路径） ----
class PlainHost extends Node2D:
	func is_paused() -> bool:
		return false
	func is_grid_path_active() -> bool:
		return false


func _make_player(pos: Vector2) -> Node2D:
	var player := Node2D.new()
	player.name = "Player"
	root.add_child(player)
	player.add_to_group("player")
	player.position = pos
	return player


func _run() -> void:
	# ---- A. 边沿触发 ----
	var host := GreetHost.new()
	root.add_child(host)
	host.position = Vector2(500, 500)
	var greet := GdNpcGreet.new()
	greet.name = "Greet"
	greet.lines = PackedStringArray(["客官安好。", "又见面了。"])
	greet.radius = 100.0
	greet.cooldown = 50.0
	greet.line_secs = 0.3
	greet.initial_cooldown = 0.0
	host.add_child(greet)
	var player := _make_player(Vector2(2000, 2000))
	await create_timer(0.5).timeout
	check(host.bubbles.is_empty(), "玩家远处不应问候")
	player.position = Vector2(540, 500)  # 走近（半径内）
	await create_timer(0.5).timeout
	check(host.bubbles.size() == 1, "靠近应触发一次问候（实际 %d 次）" % host.bubbles.size())
	if host.bubbles.size() > 0:
		check(host.bubbles[0] == "客官安好。" or host.bubbles[0] == "又见面了。",
			"问候内容应来自 lines（实际 %s）" % str(host.bubbles[0]))

	# ---- B. 冷却：退出再进入不重复 ----
	player.position = Vector2(2000, 2000)
	await create_timer(0.5).timeout
	player.position = Vector2(540, 500)
	await create_timer(0.5).timeout
	check(host.bubbles.size() == 1, "冷却内重复靠近不应再次问候（实际 %d 次）" % host.bubbles.size())

	# ---- C. resolve_greeting 优先（重逢问候） ----
	var host2 := GreetHost.new()
	root.add_child(host2)
	host2.position = Vector2(800, 500)
	host2.greeting = "故人重逢，别来无恙。"
	var greet2 := GdNpcGreet.new()
	greet2.lines = PackedStringArray(["客官安好。"])
	greet2.radius = 100.0
	greet2.cooldown = 50.0
	greet2.line_secs = 0.3
	greet2.initial_cooldown = 0.0
	host2.add_child(greet2)
	player.position = Vector2(840, 500)
	await create_timer(0.5).timeout
	check(host2.bubbles.size() == 1 and host2.bubbles[0] == "故人重逢，别来无恙。",
		"resolve_greeting 非空应优先（实际 %s）" % str(host2.bubbles))

	# ---- D. 宿主忙（赶路/对话中）不触发 ----
	var host3 := GreetHost.new()
	root.add_child(host3)
	host3.position = Vector2(1100, 500)
	host3.path_active = true  # 赶路中
	var greet3 := GdNpcGreet.new()
	greet3.lines = PackedStringArray("忙完再说。".split(","))
	greet3.radius = 100.0
	greet3.cooldown = 50.0
	greet3.line_secs = 0.3
	greet3.initial_cooldown = 0.0
	host3.add_child(greet3)
	player.position = Vector2(1140, 500)
	await create_timer(0.5).timeout
	check(host3.bubbles.is_empty(), "宿主赶路中不应问候")
	host3.path_active = false
	host3.paused = true  # 对话中
	player.position = Vector2(2000, 2000)
	await create_timer(0.5).timeout
	player.position = Vector2(1140, 500)
	await create_timer(0.5).timeout
	check(host3.bubbles.is_empty(), "宿主对话中不应问候")
	host3.paused = false  # 忙结束 + 玩家先离开再靠近 → 边沿重新武装
	player.position = Vector2(2000, 2000)
	await create_timer(0.5).timeout
	player.position = Vector2(1140, 500)
	await create_timer(0.5).timeout
	check(host3.bubbles.size() == 1, "忙结束且重新靠近应触发问候（实际 %d 次）" % host3.bubbles.size())

	# ---- E. 无 show_bubble 宿主 → 自建 Label 显示并按时长收起 ----
	var host4 := PlainHost.new()
	root.add_child(host4)
	host4.position = Vector2(1400, 500)
	var greet4 := GdNpcGreet.new()
	greet4.lines = PackedStringArray(["(点头示意)"])
	greet4.radius = 100.0
	greet4.cooldown = 50.0
	greet4.line_secs = 0.3
	greet4.initial_cooldown = 0.0
	host4.add_child(greet4)
	player.position = Vector2(1440, 500)
	await create_timer(0.5).timeout
	var bubble: Node = greet4.get_node_or_null(^"__GreetBubble")
	check(bubble != null, "无 show_bubble 宿主应自建气泡标签")
	if bubble != null:
		check(bubble.visible and bubble.text == "(点头示意)",
			"自建气泡应可见且内容正确（实际 visible=%s text=%s）" % [str(bubble.visible), bubble.text])
		await create_timer(1.0).timeout
		check(not bubble.visible, "line_secs 到时自建气泡应收起")

	print("[NpcGreet] %s" % ("ALL PASS" if ok else "FAILED"))
	quit(0 if ok else 1)
