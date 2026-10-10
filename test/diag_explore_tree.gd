extends SceneTree
func _initialize() -> void:
	var packed = load("res://example/demo/xiuxian/index.tscn")
	var sr = packed.instantiate()
	root.add_child(sr)
	for i in 10: await process_frame
	var core = Engine.get_singleton("GDCORE")
	var gs = core.get_global_node("default")
	gs.call("change_scene", "xiuxian_explore", {}, "", false, false)
	for i in 20: await process_frame
	var cur = gs.call("get_current_scene")
	print("DiagET: current=", cur)
	_dump(cur, 0)
	quit(0)

func _dump(n: Node, d: int) -> void:
	print("%s%s (%s)" % ["  ".repeat(d), n.name, n.get_class()])
	for c in n.get_children():
		_dump(c, d + 1)
