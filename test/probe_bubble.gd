extends SceneTree

## 排查探针：prologue 演出中小翠头顶气泡可见性采样
## 运行：perl -e 'alarm 90; exec @ARGV' godot --headless --path . -s res://test/probe_bubble.gd

const APP_SCENE := "res://example/demo/xiuxian/index.tscn"

var _maid: Node = null
var _bubble: Node = null
var _last_vis: bool = false
var _vis_changes: Array = []
var _start_ms := 0
var _next_log := 0
var _app: Node = null
var _entered := false
var _done := false


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
	if _done:
		return false
	if _maid == null or _bubble == null:
		var cur: Node = _app.call("get_current_scene")
		if cur == null or cur.call("get_scene_id") != "xiuxian_main":
			return false
		var found: Array[Node] = cur.find_children("xiao_maid", "", true, false)
		if found.is_empty():
			return false
		_maid = found[0]
		var bl: Array[Node] = _maid.find_children("__MovieBubble", "", true, false)
		_bubble = bl[0] if not bl.is_empty() else null
		if _bubble == null:
			return false
		_start_ms = Time.get_ticks_msec()
		print("[BProbe] 找到气泡，开始采样 pos=%s bubble_valid=%s" % [
			_maid.global_position, _bubble.is_instance_valid()])
	if not is_instance_valid(_bubble) or not is_instance_valid(_maid):
		print("[BProbe] t=%dms 节点失效" % (Time.get_ticks_msec() - _start_ms))
		_report(0)
		return true
	var vis: bool = _bubble.is_visible()
	var maid_pos: Vector2 = _maid.global_position
	if vis != _last_vis:
		print("[BProbe] t=%dms visible %s -> %s  maid_pos=%s bubble_gpos=%s" % [
			Time.get_ticks_msec() - _start_ms, _last_vis, vis,
			maid_pos, _bubble.global_position])
		_vis_changes.append(true)
		_last_vis = vis
	if Time.get_ticks_msec() - _start_ms >= _next_log:
		_next_log += 500
		print("[BProbe] t=%dms vis=%s maid=%s bubble_gpos=%s" % [
			Time.get_ticks_msec() - _start_ms, vis, maid_pos,
			_bubble.global_position])
	# 演出结束（气泡消失且小翠被释放）或可见性变化 ≥4 次后收尾
	if _vis_changes.size() >= 6:
		_report(0)
		return true
	return false


func _report(code: int) -> void:
	print("[BProbe] DONE code=", code)
	quit(code)


func _scene_id() -> String:
	var cur: Node = _app.call("get_current_scene")
	return str(cur.call("get_scene_id")) if cur != null else "<null>"
