# 对话立绘链路检查 —— Rust 元数据注入 + DialogBox UI 显示
# 运行：godot --headless -s test/check_dialog_portrait.gd（项目根目录）
# 校验：
#   A. GdDialogue.register_role_meta 注册后，handle_line 行字典随行下发
#      display_name / portrait；未注册角色不注入
#   B. get_role_portrait / get_role_display_name 查询 API
#   C. DialogBox 立绘 TextureRect：有 portrait 显示加载贴图（尺寸/自适应/
#      鼠标穿透参数），无 portrait 隐藏；display_name 优先于角色名
#   D. XiuNpcBase 自动探测：elder 实例化后 Speaker 带上本目录立绘与展示名
extends SceneTree

const PORTRAIT_PATH := "res://example/demo/xiuxian/role/npc/elder/assets/dialog_basic.png"

var ok := true

func check(cond: bool, msg: String) -> void:
	if cond:
		print("  PASS: " + msg)
	else:
		printerr("  FAIL: " + msg)
		ok = false


class LineCatcher extends Node:
	var lines: Array = []
	func handle_line(line: Dictionary) -> void:
		lines.append(line)


func _init() -> void:
	print("[DialogPortrait] 开始检查")
	_check_dialogue_meta()
	_run_rest()


# await 分段收尾：RESULT 与 quit 必须等全部异步检查完成（否则退出码不完整）
func _run_rest() -> void:
	await _check_dialog_box_ui()
	await _check_npc_auto_portrait()
	print("[DialogPortrait] RESULT=%s" % ("PASS" if ok else "FAIL"))
	quit(0 if ok else 1)


# ---- A/B: Rust 侧元数据注册与随行下发 ----
func _check_dialogue_meta() -> void:
	print("[A/B] GdDialogue 元数据注入")
	var dia: GdDialogue = GdDialogue.new()
	dia.name = "Dialogue"
	root.add_child(dia)
	var catcher := LineCatcher.new()
	catcher.name = "Catcher"
	root.add_child(catcher)
	dia.set_dialogue_control_path(dia.get_path_to(catcher))

	dia.initial("[t]\n(执事长老)\n长老开口了。\n(云舟)\n玩家回应。\n")
	dia.register_role_meta("执事长老", "执事长老·特写", PORTRAIT_PATH)

	dia.next("")
	dia.next("")
	check(catcher.lines.size() == 2, "应回调 2 行（实际 %d）" % catcher.lines.size())
	if catcher.lines.size() == 2:
		var l0: Dictionary = catcher.lines[0]
		var l1: Dictionary = catcher.lines[1]
		check(str(l0.get("name", "")) == "执事长老", "第 1 行角色名应 执事长老")
		check(str(l0.get("display_name", "")) == "执事长老·特写",
			"第 1 行应携带 display_name")
		check(str(l0.get("portrait", "")) == PORTRAIT_PATH, "第 1 行应携带立绘路径")
		check(str(l1.get("name", "")) == "云舟" and not l1.has("portrait"),
			"未注册角色的行不应携带 portrait 键")
		check(not l1.has("display_name"), "未注册角色的行不应携带 display_name 键")

	check(dia.get_role_portrait("执事长老") == PORTRAIT_PATH, "get_role_portrait 应命中")
	check(dia.get_role_portrait("云舟") == "", "未注册角色 get_role_portrait 应为空")
	check(str(dia.get_role_display_name("执事长老")) == "执事长老·特写",
		"get_role_display_name 应命中")
	dia.queue_free()
	catcher.queue_free()


# ---- C: DialogBox 立绘 UI ----
func _check_dialog_box_ui() -> void:
	print("[C] DialogBox 立绘 UI")
	var script: GDScript = load("res://example/demo/xiuxian/ui/dialog/dialog_box.gd")
	var box: CanvasLayer = script.new()
	root.add_child(box)
	await process_frame

	var rect: TextureRect = box.portrait_rect
	check(rect != null, "应构建 portrait_rect")
	if rect == null:
		box.queue_free()
		return
	check(not rect.visible, "初始应隐藏")
	check(rect.expand_mode == TextureRect.EXPAND_IGNORE_SIZE,
		"expand_mode 应 EXPAND_IGNORE_SIZE（任意尺寸素材自适应）")
	check(rect.stretch_mode == TextureRect.STRETCH_KEEP_ASPECT_CENTERED,
		"stretch_mode 应 KEEP_ASPECT_CENTERED（等比居中）")
	check(rect.mouse_filter == Control.MOUSE_FILTER_IGNORE,
		"立绘应鼠标 IGNORE（不吞点击推进）")
	check(rect.size == Vector2(400, 696), "立绘槽位应 400x696（实际 %s）" % rect.size)

	# 有立绘的行：显示 + 贴图加载成功 + 展示名优先
	box.handle_line({
		"name": "执事长老", "text": "长老台词。",
		"display_name": "执事长老·特写", "portrait": PORTRAIT_PATH,
	})
	check(rect.visible, "有 portrait 的行应显示立绘")
	check(rect.texture != null, "立绘贴图应加载成功（素材已导入）")
	check(str(box.name_label.text) == "执事长老·特写", "展示名应优先于角色名显示")

	# 无立绘的行（玩家）：隐藏 + 回退角色名
	box.handle_line({"name": "云舟", "text": "玩家台词。"})
	check(not rect.visible, "无 portrait 的行应隐藏立绘")
	check(str(box.name_label.text) == "云舟", "无展示名应回退角色名")
	box.queue_free()


# ---- D: XiuNpcBase 立绘自动探测 ----
func _check_npc_auto_portrait() -> void:
	print("[D] XiuNpcBase 自动探测")
	var elder: Node = load("res://example/demo/xiuxian/role/npc/elder/elder.gd").new()
	elder.name = "ElderProbe"
	root.add_child(elder)
	await process_frame
	var speaker: Node = elder.get_node_or_null(NodePath("Speaker"))
	check(speaker != null, "elder 应装配 Speaker")
	if speaker != null:
		check(str(speaker.get("portrait")) == PORTRAIT_PATH,
			"Speaker 立绘应自动探测为本目录 dialog_basic.png（实际 %s）" % speaker.get("portrait"))
		check(str(speaker.get("display_name")) == "执事长老", "Speaker 应携带展示名")
		check(str(speaker.get("role_name")) == "执事长老", "Speaker 角色名不变")
	elder.queue_free()
