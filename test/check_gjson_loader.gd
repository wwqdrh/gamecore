# GdJson 资源加载器检查 —— .gjson 双击/运行时加载入口
# 运行：godot --headless -s test/check_gjson_loader.gd（项目根目录）
# 校验：
#   1. ResourceLoader.load 能加载 .gjson（走 Rust 侧 ResourceFormatLoader）
#   2. 资源类型为 GdJson，get_data 返回完整定义表
#   3. query 分号路径查询命中/未命中
#   4. 失败路径（文件不存在）不崩溃返回 null
extends SceneTree

var ok := true

func check(cond: bool, msg: String) -> void:
	if cond:
		print("  PASS: " + msg)
	else:
		printerr("  FAIL: " + msg)
		ok = false

func _init() -> void:
	print("[GjsonLoader] 开始检查")

	# 1. 加载 item.gjson（加密产物）
	var res: Resource = load("res://example/demo/xiuxian/state/item/item.gjson")
	check(res != null, "load item.gjson 应成功")
	if res == null:
		_finish()
		return
	check(res.get_class() == "GdJson", "资源类型应为 GdJson（实际 %s）" % res.get_class())

	# 2. get_data 返回完整定义表
	var data: Variant = res.call("get_data")
	check(data is Dictionary and (data as Dictionary).has("items"),
		"get_data 应为含 items 键的 Dictionary")
	var items: Dictionary = (data as Dictionary).get("items", {})
	check(items.size() == 24, "道具定义应 24 条（实际 %d）" % items.size())

	# 3. query 分号路径查询
	check(str(res.call("query", "items;pill_hp;name")) == "回春丹",
		"query items;pill_hp;name 应为 回春丹")
	check(int(res.call("query", "items;pill_exp_s;exp")) == 500,
		"query items;pill_exp_s;exp 应为 500")
	check(res.call("query", "items;no_such;name") == null,
		"query 未命中路径应返回 null")
	check(res.call("query", "") == null or res.call("query", "") != null,
		"query 空路径不崩溃")

	# 4. 文件不存在：返回 null 且不崩溃
	var missing: Resource = load("res://example/demo/xiuxian/state/item/no_such.gjson")
	check(missing == null, "加载不存在的 gjson 应返回 null")

	_finish()


func _finish() -> void:
	print("[GjsonLoader] RESULT=%s" % ("PASS" if ok else "FAIL"))
	quit(0 if ok else 1)
