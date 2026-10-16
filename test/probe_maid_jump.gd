extends SceneTree

## 排查探针：prologue 演出中小翠位置突变检测（单帧跳 > 1.5 格即记录）
## 运行：perl -e 'alarm 90; exec @ARGV' godot --headless --path . -s res://test/probe_maid_jump.gd

const APP_SCENE := "res://example/demo/xiuxian/index.tscn"

var _maid: Node = null
var _last := Vector2.INF
var _jumps: Array = []
var _start_ms := 0
var _next_log := 0
var _app: Node = null
var _entered := false


func _initialize() -> void:
	var wd := Timer.new()
	wd.wait_time = 40.0
	wd.one_shot = true
	wd.autostart = true
	wd.timeout.connect(func() -> void: _report(2))
	root.add_child(wd)

	load("res://example/demo/xiuxian/state/test_save_guard.gd").isolate()

	_app = load(APP_SCENE).instantiate()
	root.add_child(_app)


func _process(_delta: float) -> bool:
	if not _entered:
		if _scene_id() != "xiuxian_title":
			return false
		_entered = true
		XiuMovieState.ins().reset_demo()
		_app.call("change_scene", "xiuxian_main", {}, "", false, false)
		return false
	if _scene_id() != "xiuxian_main":
		return false
	if _maid == null:
		var cur: Node = _app.call("get_current_scene")
		_maid = _find_maid(cur)
		if _maid == null:
			return false
		_start_ms = Time.get_ticks_msec()
		print("[Probe] 找到小翠，开始采样")
	if not is_instance_valid(_maid):
		_jumps.append({"t": Time.get_ticks_msec() - _start_ms, "what": "FREED"})
		_report(0)
		return true
	var pos: Vector2 = _maid.global_position
	if _last != Vector2.INF and pos.distance_to(_last) > 96.0:
		var brain: Node = _maid.get("brain")
		var binfo := "<null>"
		if brain != null and is_instance_valid(brain):
			binfo = "phase=%s home=%s target=%s" % [
				brain.call("get_ai_state"), brain.call("get_home"),
				brain.get("current_target")]
		print("[Probe] JUMP t=%dms from=%s to=%s npc_cell=%s snapped=%s" % [
			Time.get_ticks_msec() - _start_ms, _last, pos, _maid.get("npc_cell"),
			_maid.get("_snapped")])
		print("[Probe]       iid=%d parent=%s schedule=%s brain=%s" % [
			_maid.get_instance_id(), _maid.get_parent().name,
			_maid.get("schedule") != null, binfo])
		_report(0)
		return true
	_last = pos
	if Time.get_ticks_msec() - _start_ms >= _next_log:
		_next_log += 400
		print("[Probe] t=%dms pos=%s path_active=%s paused=%s" % [
			Time.get_ticks_msec() - _start_ms, pos,
			_maid.call("is_grid_path_active"), _maid.call("is_paused")])
	return false


func _find_maid(cur: Node) -> Node:
	var found: Array[Node] = cur.find_children("xiao_maid", "", true, false)
	return found[0] if not found.is_empty() else null


func _scene_id() -> String:
	var cur: Node = _app.call("get_current_scene")
	return str(cur.call("get_scene_id")) if cur != null else "<null>"


func _report(code: int) -> void:
	print("[Probe] 位置突变记录 (", _jumps.size(), " 条):")
	for j in _jumps:
		print("  ", j)
	print("[Probe] DONE code=", code)
	quit(code)
