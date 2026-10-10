# 存档选择弹窗控制器 —— save_modal.gml 根 <Panel> 挂载（<ui script> 声明），
# 被 title_menu.gml 的 <Modal ui_id="SaveModal" content_margin="56"> 包裹。
#
# 槽位约定（3 槽）：
#   槽一 = 默认档 user://coredata.data（save_id ""，兼容历史进度）
#   槽二 = user://coredata_2.data（save_id "2"）
#   槽三 = user://coredata_3.data（save_id "3"）
#
# 行为：空档按钮「新游戏」，有档显示 境界/天数/游玩时长 + 「继续游戏」。
# 进入存档：GDCORE.set_save_id(id)（GdCoreData.build force=false 读旧档 /
#           空档建新库；Bean do_switch_core 空档重置默认值、有档读入）
#           → 发 s_close_requested 关 Modal → change_scene("xiuxian_main")。
#
# 槽位摘要读取：临时 GdCoreData 直查存档文件（不影响当前档缓存）：
#   level = init;xiuxian_character;level / day = init;xiuxian_calendar;day
#   playtime = playtime;total
extends Control

## 面板关闭请求：<Modal> 监听本信号整体关闭（同 dungeon_settle 面板约定）
signal s_close_requested

## 槽位 -> 存档 id（"" = 默认档 coredata.data）
const SLOT_IDS := ["", "2", "3"]


func _ready() -> void:
	refresh_slots()


# ---------------------------------------------------------------- 查询

## 存档文件路径
func _save_path(id: String) -> String:
	if id.is_empty():
		return "user://coredata.data"
	return "user://coredata_%s.data" % id


## 读单个槽位摘要：空档返回空字典；有档返回 {level, realm, day, playtime}
func read_slot(id: String) -> Dictionary:
	var path := _save_path(id)
	if not FileAccess.file_exists(path):
		return {}
	var data: GdCoreData = GdCoreData.build(path, "{}", false, "init")
	var level_f: float = data.value("xiuxian_character;level", 1.0, "init")
	var day_f: float = data.value("xiuxian_calendar;day", 1.0, "init")
	var secs_f: float = data.value("total", 0.0, "playtime")
	var level := int(level_f)
	return {
		"level": level,
		"realm": XiuLevelTable.get_rank_title_at(level),
		"day": int(day_f),
		"playtime": secs_f,
	}


## 秒数 → 「x 小时 x 分 / x 分 x 秒 / x 秒」
func _format_time(secs: float) -> String:
	var total := int(secs)
	if total < 60:
		return "%d 秒" % total
	if total < 3600:
		var m := int(total / 60.0)
		var s := total - m * 60
		return "%d 分 %d 秒" % [m, s]
	var h := int(total / 3600.0)
	var m2 := int((total - h * 3600) / 60.0)
	return "%d 小时 %d 分" % [h, m2]


# ---------------------------------------------------------------- 刷新

## 重读三个槽位并刷新行内容（打开弹窗前由 title_menu 调用，_ready 也会刷一次）
func refresh_slots() -> void:
	for i in SLOT_IDS.size():
		var n := i + 1
		var info := read_slot(SLOT_IDS[i])
		var line1: Label = find_child("Slot%dLine1" % n, true, false)
		var line2: Label = find_child("Slot%dLine2" % n, true, false)
		var btn: Button = find_child("Slot%dBtn" % n, true, false)
		if btn == null:
			continue
		if info.is_empty():
			if line1:
				line1.text = "暂无存档"
			if line2:
				line2.text = "点击开始新的修仙之旅"
			btn.text = "新游戏"
		else:
			if line1:
				line1.text = "境界：%s · 第 %d 天" % [info["realm"], info["day"]]
			if line2:
				line2.text = "游玩时长：%s" % _format_time(info["playtime"])
			btn.text = "继续游戏"


# ---------------------------------------------------------------- 进入存档

func _on_slot1_pressed() -> void:
	_enter_save(0)


func _on_slot2_pressed() -> void:
	_enter_save(1)


func _on_slot3_pressed() -> void:
	_enter_save(2)


## 切到目标存档并进入主场景（新游戏/继续共用：空档自然以默认值开局）
func _enter_save(idx: int) -> void:
	if idx < 0 or idx >= SLOT_IDS.size():
		return
	if not Engine.has_singleton("GDCORE"):
		push_warning("SaveModal: GDCORE 单例不存在，无法切档")
		return
	Engine.get_singleton("GDCORE").set_save_id(SLOT_IDS[idx])
	# 先关弹窗（Modal 收到 s_close_requested 播放关闭动画），再切场景
	emit_signal("s_close_requested")
	var scene := _find_scene()
	if scene != null:
		var entered: bool = scene.call("change_scene", "xiuxian_main", {})
		if not entered:
			push_warning("SaveModal: change_scene(xiuxian_main) 失败（场景未注册或转场中？）")
	else:
		push_warning("SaveModal: 未找到 GdScene 场景根，无法切换场景")


## 返回按钮：只关弹窗（留在标题页）
func _on_back_pressed() -> void:
	emit_signal("s_close_requested")


# ---------------------------------------------------------------- 查找

## 沿父链向上找场景根（GdScene，具备 change_scene 方法）
func _find_scene() -> Node:
	var node: Node = get_parent()
	while node != null:
		if node.has_method("change_scene"):
			return node
		node = node.get_parent()
	return null
