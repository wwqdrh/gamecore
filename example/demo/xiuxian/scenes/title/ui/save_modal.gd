# 存档选择弹窗控制器 —— save_modal.gml 根 <Panel> 挂载（<ui script> 声明），
# 被 title_menu.gml 的 <Modal ui_id="SaveModal" content_margin="56"> 包裹。
#
# 槽位约定（3 槽）：
#   槽一 = 默认档 user://coredata.data（save_id ""，兼容历史进度）
#   槽二 = user://coredata_2.data（save_id "2"）
#   槽三 = user://coredata_3.data（save_id "3"）
#
# 行为：空档按钮「新游戏」，有档显示 境界/天数/游玩时长 + 「继续游戏 + 删除存档」。
# 删除存档 = 两步确认（首点进确认态 3 秒，再点执行 GDCORE.delete_save），
# 删后 GdUIReactive.set_var(slotN_has_save=false) 响应式隐藏删除按钮、
# 主按钮自动回「新游戏」（show_if 条件显隐，gml 框架 rust 侧提供）。
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

## 删除确认时限（毫秒，墙钟计时——无头测试帧率不节流，帧数≠时间）
const CONFIRM_MS := 3000
const DEL_TEXT := "删除存档"
const CONFIRM_TEXT := "确认删除?"

## 两步删除确认状态：待确认槽位索引（-1 = 无）
var _confirm_idx := -1
## 确认态过期时刻（Time.get_ticks_msec 墙钟）
var _confirm_until_ms := 0


func _ready() -> void:
	refresh_slots()


func _process(_delta: float) -> void:
	# 删除确认态超时自动复原（墙钟计时，帧率无关）
	if _confirm_idx >= 0 and Time.get_ticks_msec() >= _confirm_until_ms:
		_reset_confirm()


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
## 删除按钮显隐走响应式条件（gml show_if="slotN_has_save"）：
## GdUIReactive.set_var 变更即重算，无需手动操作节点
func refresh_slots() -> void:
	_reset_confirm()
	for i in SLOT_IDS.size():
		var n := i + 1
		var info := read_slot(SLOT_IDS[i])
		var line1: Label = find_child("Slot%dLine1" % n, true, false)
		var line2: Label = find_child("Slot%dLine2" % n, true, false)
		var btn: Button = find_child("Slot%dBtn" % n, true, false)
		var del_btn: Button = GdUIManager.get_gml_node(self, "Slot%dDelBtn" % n)
		if del_btn:
			del_btn.text = DEL_TEXT
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
		# 响应式变量变更：驱动该槽删除按钮 show_if 即时显隐
		GdUIReactive.set_var(self, "slot%d_has_save" % n, not info.is_empty())


# ---------------------------------------------------------------- 删除存档

func _on_slot1_del_pressed() -> void:
	_on_slot_del_pressed(0)


func _on_slot2_del_pressed() -> void:
	_on_slot_del_pressed(1)


func _on_slot3_del_pressed() -> void:
	_on_slot_del_pressed(2)


## 两步确认删除：首点进入确认态（按钮变「确认删除?」，CONFIRM_MS 内有效），
## 再点执行 GDCORE.delete_save（正规删档途径：文件+缓存+Bean 全重置）。
## 删后 refresh_slots → set_var(false) → 删除按钮响应式隐藏、主按钮回「新游戏」。
## 点其他槽位删除键或超时会自动复原，避免误删。
func _on_slot_del_pressed(idx: int) -> void:
	if idx < 0 or idx >= SLOT_IDS.size():
		return
	if _confirm_idx != idx:
		# 首次点击（或从其他槽位切来）：进入确认态
		_reset_confirm()
		_confirm_idx = idx
		_confirm_until_ms = Time.get_ticks_msec() + CONFIRM_MS
		var btn := _del_btn(idx)
		if btn:
			btn.text = CONFIRM_TEXT
		return
	# 二次点击：执行删除
	_confirm_idx = -1
	if not Engine.has_singleton("GDCORE"):
		push_warning("SaveModal: GDCORE 单例不存在，无法删除存档")
		return
	var ok: bool = Engine.get_singleton("GDCORE").delete_save(SLOT_IDS[idx])
	if not ok:
		push_warning("SaveModal: 删除存档失败（slot %s）" % SLOT_IDS[idx])
	refresh_slots()


## 退出删除确认态：按钮文案复原
func _reset_confirm() -> void:
	if _confirm_idx < 0:
		return
	var idx := _confirm_idx
	_confirm_idx = -1
	var btn := _del_btn(idx)
	if btn:
		btn.text = DEL_TEXT


## 按槽位索引取删除按钮（gml name 注册表直取）
func _del_btn(idx: int) -> Button:
	var node: Object = GdUIManager.get_gml_node(self, "Slot%dDelBtn" % (idx + 1))
	return node as Button


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
