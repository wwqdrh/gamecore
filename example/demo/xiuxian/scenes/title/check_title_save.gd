# 校验脚本：标题页存档选择弹窗（scenes/title/ui/）
# 运行：godot --headless --path . -s res://example/demo/xiuxian/scenes/title/check_title_save.gd
#
# 覆盖：1) title_menu/save_modal gml 构建并重生成 tscn（等效编辑器插件产物）
#       2) Modal 结构与初始隐藏；点「开始游戏」打开弹窗
#       3) 槽位摘要：空档「新游戏」、有档「境界·天数 + 继续游戏」
#       4) 切档语义：空档 Bean 重置默认值（新游戏），有档读回进度（继续）
#       5) 点空档槽位 → GDCORE.set_save_id + change_scene 进主场景全链路
#
# 零污染约定（2026-10-11）：本测试是存档 UI 专项，不能用 _test 守卫
# （被测对象就是真实槽位路径），因此**只写槽二/三的测试档并全部清理**，
# 对槽一（save_id ""，玩家真实档）只读不断言固定内容、不写入任何字段。
extends SceneTree

const DIR := "res://example/demo/xiuxian/scenes/title/ui/"
const GML_FILES := ["title_menu.gml", "save_modal.gml"]
## 槽一（默认档）文件路径——只读，用于条件断言
const SLOT1_PATH := "user://coredata.data"

var fails := 0


func check(cond: bool, msg: String) -> void:
	if cond:
		print("[PASS] ", msg)
	else:
		fails += 1
		printerr("[FAIL] ", msg)


func _initialize() -> void:
	_run()


func _run() -> void:
	# 1. gml 构建 + 重生成 tscn
	var builder := GdUiBuilder.new()
	for f in GML_FILES:
		var scene: PackedScene = builder.build_scene_file(DIR + f)
		if scene == null:
			printerr("[Check] %s 构建失败: %s" % [f, builder.last_error()])
			fails += 1
			continue
		var err := ResourceSaver.save(scene, DIR + f + ".tscn")
		check(err == OK, "gml 产物保存 %s" % f)
	if fails > 0:
		quit(1)
		return

	# 2. 环境清场：槽二/三删档；槽二制造「有档」受控样本（level=5）
	var core: Object = Engine.get_singleton("GDCORE")
	core.set_save_id("")
	var ch := XiuCharacterState.ins()
	var slot1_exists := FileAccess.file_exists(SLOT1_PATH)
	for id in ["2", "3"]:
		var p := "user://coredata_%s.data" % id
		if FileAccess.file_exists(p):
			DirAccess.remove_absolute(p)
	core.set_save_id("2")
	ch.update("level", 5, {}, true)
	check(FileAccess.file_exists("user://coredata_2.data"), "槽二测试档已生成（受控有档样本）")

	await process_frame
	await process_frame

	# 3. 实例化标题场景
	var title_scene: PackedScene = load("res://example/demo/xiuxian/scenes/title/index.tscn")
	var title_root: Node = title_scene.instantiate()
	root.add_child(title_root)
	await process_frame
	await process_frame

	var modal: Node = title_root.find_child("SaveModal", true, false)
	check(modal != null, "SaveModal 存在于标题场景")
	if modal == null:
		quit(1)
		return
	check(not modal.is_modal_open(), "弹窗初始隐藏")
	var save_root: Node = modal.find_child("SaveRoot", true, false)
	check(save_root != null, "弹窗内容 SaveRoot 存在")
	if save_root == null:
		quit(1)
		return

	# 4. 点「开始游戏」→ 打开存档弹窗
	var start_btn: Button = title_root.find_child("BtnStart", true, false)
	check(start_btn != null, "开始按钮存在")
	if start_btn:
		start_btn.emit_signal("pressed")
	await process_frame
	await process_frame
	check(modal.is_modal_open(), "点击开始游戏后弹窗打开")

	# 5. 槽位摘要：槽一按真实文件存在性断言（只读）；槽二有档；槽三空档
	save_root.call("refresh_slots")
	await process_frame
	var btn1: Button = save_root.find_child("Slot1Btn", true, false)
	var btn2: Button = save_root.find_child("Slot2Btn", true, false)
	var btn3: Button = save_root.find_child("Slot3Btn", true, false)
	check(btn1 != null and btn1.text == ("继续游戏" if slot1_exists else "新游戏"),
		"槽一（默认档%s）按钮文案正确" % ("有存档" if slot1_exists else "空"))
	var line1: Label = save_root.find_child("Slot1Line1", true, false)
	if slot1_exists:
		check(line1 != null and line1.text.contains("境界") and line1.text.contains("天"),
			"槽一摘要含境界与天数（%s）" % (line1.text if line1 else "-"))
	check(btn2 != null and btn2.text == "继续游戏", "槽二（受控有档）显示继续游戏")
	var line2: Label = save_root.find_child("Slot2Line1", true, false)
	check(line2 != null and line2.text.contains("境界"), "槽二摘要含境界（%s）" % (line2.text if line2 else "-"))
	check(btn3 != null and btn3.text == "新游戏", "槽三空档显示新游戏")

	# 6. 切档语义：有档 = 继续（读回进度）；空档 = 新游戏（重置默认值）
	core.set_save_id("")
	await process_frame
	var level_before := ch.level
	core.set_save_id("2")
	await process_frame
	check(int(ch.level) == 5, "切到槽二读回受控进度（继续游戏语义）")
	core.set_save_id("3")
	await process_frame
	check(int(ch.level) == 1, "切到空槽三角色重置默认等级（新游戏语义）")
	core.set_save_id("")
	await process_frame
	check(int(ch.level) == level_before, "切回默认档恢复原值（真实档未被污染）")

	# 7. 点空档（槽三）进入主场景：切档 + 关弹窗 + change_scene 全链路
	#    （change_scene 会整体替换场景根，旧 title_root/modal 随之释放——
	#      轮询断言一律先 is_instance_valid 兜底）
	save_root.call("_enter_save", 2)
	var switched := false
	for i in 600:
		await process_frame
		if not is_instance_valid(title_root) \
				or title_root.find_child("BtnStart", true, false) == null:
			switched = true
			break
	check(switched, "选择空档槽位后切换进入主场景")
	check(str(core.get_save_id()) == "3", "进入存档三（save_id=3）")
	check(int(ch.level) == 1, "新开局角色为默认等级")
	check(root.find_child("DungeonManager", true, false) != null
			and root.find_child("CalendarManager", true, false) != null,
		"主场景组件已接线（DungeonManager/CalendarManager）")

	# 8. 清理：切回默认档、删测试槽位档（真实档全程零写入）
	core.set_save_id("")
	for id in ["2", "3"]:
		var p2 := "user://coredata_%s.data" % id
		if FileAccess.file_exists(p2):
			DirAccess.remove_absolute(p2)

	print("RESULT: fails=%d" % fails)
	quit(1 if fails > 0 else 0)
