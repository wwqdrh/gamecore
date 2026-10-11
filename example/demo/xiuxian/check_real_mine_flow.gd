# 实机路径全场景复现 —— 图标 + 挖矿持久化 一次性钉死
# 运行：godot --headless --path . -s res://example/demo/xiuxian/check_real_mine_flow.gd
#
# 覆盖（与用户实机操作逐环对应）：
#   A. 图标链路：main.gml 组合根（实机加载的同一产物链）构建 → 打开 BagModal
#      → 网格首格图标应为注册表解析结果（灵芝 🍄 / 寒铁矿 🪨），格子 58 高
#   B. 定义表：XiuItemTable 直查 icon 字段应为逻辑图片 id（img_*）
#   C. 挖矿全链：实例化真实 xiaozhai.tscn（含 DecoLayer+DecoHarvest）
#      → 真实 GdMelee 挥击（mask=16，与玄铁镐一致）3 次
#      → 采集回调宿主 on_ore_harvested → XiuItemState.add_item
#   D. 持久化：切走 save_id 再切回（强制从磁盘重读 bean），数量应保持
extends SceneTree

const MAIN_GML := "res://example/demo/xiuxian/scenes/main/ui/main.gml"
const MAP_SCENE := "res://example/demo/xiuxian/map/xiaozhai.tscn"


func _initialize() -> void:
	_run()


func _check(cond: bool, msg: String) -> void:
	if cond:
		print("[Mine] OK  ", msg)
	else:
		push_error("[Mine] FAIL  " + msg)
		ok = false


var ok := true


func _run() -> void:
	# 存档隔离：测试全程跑专用档 _test，严禁污染玩家真实存档
	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()

	# ---- B. 定义表 icon 字段 = 逻辑图片 id ----
	var it := XiuItemState.ins()
	it.reset_demo()
	print("[Mine] item.gjson herb_lingzhi.icon=%s ore_coldiron.icon=%s" % [
		it.get_item("herb_lingzhi").get("icon", "<missing>"),
		it.get_item("ore_coldiron").get("icon", "<missing>")])
	_check(str(it.get_item("herb_lingzhi").get("icon")) == "img_herb_lingzhi",
		"定义表 icon 应为逻辑图片 id img_herb_lingzhi")

	# ---- A. main.gml 组合根（实机产物链）→ BagModal 图标 ----
	it.add_item("herb_lingzhi", 1)
	it.add_item("ore_coldiron", 2)
	var builder := GdUiBuilder.new()
	var scene: PackedScene = builder.build_scene_file(MAIN_GML)
	if scene == null:
		push_error("[Mine] main.gml 构建失败: %s" % builder.last_error())
		load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
		quit(1)
		return
	var holder: Control = Control.new()
	holder.size = Vector2(1920, 1080)
	root.size = Vector2(1920, 1080)
	root.add_child(holder)
	var demo: Control = scene.instantiate()
	holder.add_child(demo)
	for i in 5:
		await process_frame

	var modal: Control = demo.find_child("BagModal", true, false)
	if modal == null:
		push_error("[Mine] 组合根中找不到 BagModal")
		load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
		quit(1)
		return
	modal.call("open")
	for i in 4:
		await process_frame

	var grid: Container = demo.find_child("GoodsGrid", true, false)
	if grid == null:
		push_error("[Mine] BagModal 内容中找不到 GoodsGrid")
		load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
		quit(1)
		return
	for i in 3:
		await process_frame  # 数据绑定 + 网格刷新
	var n: int = grid.get_child_count() - 1
	print("[Mine] grid items=%d" % n)
	_check(n == 2, "背包应显示 2 种物品（灵芝/寒铁矿），实际 %d" % n)
	if n >= 2:
		var c0: Control = grid.get_at(0)
		var ic0: Label = c0.find_child("CellIcon", true, false)
		var c1: Control = grid.get_at(1)
		var ic1: Label = c1.find_child("CellIcon", true, false)
		print("[Mine] item0=%s icon=%s size=%s" % [c0.name, ic0.text, c0.size])
		print("[Mine] item1=%s icon=%s" % [c1.name, ic1.text])
		_check(ic0.text == "🍄", "灵芝图标应为注册表 🍄（实机产物链），实际 %s" % ic0.text)
		_check(ic1.text == "🪨", "寒铁矿图标应为注册表 🪨（实机产物链），实际 %s" % ic1.text)
		_check(absf(c0.size.y - 80.0) < 2.0, "格子高应 80（模板 min，容纳贴图+角标），实际 %.0f" % c0.size.y)
	modal.call("close")

	# ---- C. 真实 xiaozhai.tscn + 真实挥击挖矿 ----
	var map: Node = load(MAP_SCENE).instantiate()
	root.add_child(map)
	await process_frame
	await process_frame  # ready + call_deferred spawn_deco
	var layer: Node = map.find_child("DecoLayer", true, false)
	_check(layer != null, "xiaozhai.tscn 应含 DecoLayer")
	if layer == null:
		load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
		quit(1)
		return
	var ores: Array = []
	for child in layer.get_children():
		if child.has_meta("deco_item_id"):
			ores.append(child)
	print("[Mine] ores spawned=%d" % ores.size())
	_check(ores.size() > 0, "xiaozhai 应铺出矿石（seed=20261001）")
	if ores.is_empty():
		load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
		quit(1)
		return

	var ore: Node = ores[0]
	var ore_id := String(ore.get_meta("deco_item_id"))
	print("[Mine] target ore id=%s pos=%s" % [ore_id, ore.position])

	# 挥击者配置与 demo 玄铁镐一致（mask=16 / 单次 1 点）
	var attacker := Node2D.new()
	attacker.position = ore.position
	map.add_child(attacker)
	var melee := GdMelee.new()
	melee.target_mask = 16
	melee.attack_damage = 1.0
	melee.attack_cooldown = 0.05
	melee.swing_window = 0.15
	melee.attack_range = 40.0
	melee.attack_width = 40.0
	attacker.add_child(melee)
	await physics_frame
	await physics_frame

	var count0: int = XiuItemState.ins().get_count(ore_id)
	for i in 3:
		var swung: bool = melee.call("swing", Vector2.RIGHT)
		_check(swung, "第 %d 次挥击应被接受" % (i + 1))
		for j in 8:
			await physics_frame
	await process_frame
	var count1: int = XiuItemState.ins().get_count(ore_id)
	print("[Mine] bag %s: %d -> %d" % [ore_id, count0, count1])
	_check(count1 == count0 + 1, "真实挥击 3 次应采集入包 +1（%d -> %d）" % [count0, count1])

	# ---- D. 持久化：切走再切回 = 从磁盘重读 ----
	var core: Object = Engine.get_singleton("GDCORE")
	core.set_save_id("")
	core.set_save_id("_test")
	var reloaded: int = XiuItemState.ins().get_count(ore_id)
	print("[Mine] cold-reload count=%d" % reloaded)
	_check(reloaded == count1, "切档冷读后数量应保持（磁盘已落地）")

	map.free()
	load("res://example/demo/xiuxian/state/test_save_guard.gd").restore()
	print("[Mine] RESULT=%s" % ("ALL GREEN" if ok else "FAILED"))
	quit(0 if ok else 1)
