# 修仙角色信息面板 —— 多 GML 组合示例（壳场景：运行时加载 profile_panel.gml）
#
# 拆分与引用（GML 间直接 <Gml> 引用，无需脚本挂载）：
#   profile_panel.gml      骨架：组合 topbar / avatar / realm / skills / stats
#   profile_topbar.gml     标题 + 境界徽章 + 关闭按钮（@pressed 回退连到本脚本）
#   profile_avatar.gml     立绘占位 + 五行灵根（<script> 数据 + UIVList 数据驱动）
#   profile_realm.gml      境界突破区（圆环占位 + 突破按钮）
#   profile_skills.gml     功法列表（UIVList + skill_item 条目模板）
#   skill_item.gd          条目自带控制器（@export 契约注入 + locked 锁定态）
#
# 本脚本负责场景级回调 + 状态层数据绑定（GdBean，example/demo/xiuxian/state/）：
#   · XiuCharacterState（xiuxian_character）→ 当前/下一境界、突破进度、
#     战斗属性条（攻击/防御/气血/灵力，attack/defense 由属性加权派生）
#   · XiuItemState（xiuxian_item）→ 突破材料徽章（真实材料 have/need）
#   watch 注册即回调当前值，面板打开即展示存档状态
#   （五行灵根/功法列表暂为静态配置数据，与养成状态无关）
extends Control
## 面板关闭请求：独立 F6 运行时仅自身 hide()；组合进 <Modal> 时
## Modal 监听本信号整体关闭（见 ui_modal.rs 内容关闭联动）
signal s_close_requested

var _char_bean: GdBean
var _item_bean: GdBean


func _ready() -> void:
	# 状态层数据绑定（watch 注册即回调当前值，无需手动刷初始 UI）
	_char_bean = XiuCharacterState.ins()
	_item_bean = XiuItemState.ins()
	# 红线：watch 按回调哈希去重（同一 Callable 重复注册被静默忽略）；
	# 多 key 共用一个处理函数必须各包一层独立 lambda（分号路径是嵌套属性语法，非多 key）
	_char_bean.watch("level", func(v = null, m = null): _on_char_changed())
	_char_bean.watch("exp", func(v = null, m = null): _on_char_changed())
	_char_bean.watch("hp", func(v = null, m = null): _on_char_changed())
	_char_bean.watch("mp", func(v = null, m = null): _on_char_changed())
	_char_bean.watch("attrs", func(v = null, m = null): _on_char_changed())
	_item_bean.watch("bag_views", func(v = null, m = null): _refresh_material_chip())


## 境界突破区 + 战斗属性条：全部由人物状态派生
func _on_char_changed(_value: Variant = null, _metas: Variant = null) -> void:
	var now: Label = find_child("RealmNow", true, false)
	if now:
		now.text = _char_bean.get_realm_title()
	var next: Label = find_child("RealmNext", true, false)
	if next:
		next.text = _char_bean.get_realm_title_at(int(_char_bean.level) + 1)
	var pct: Label = find_child("RealmProgress", true, false)
	if pct:
		pct.text = "%d%%" % roundi(_char_bean.get_exp_progress() * 100.0)
	var attack: Label = find_child("AttackValue", true, false)
	if attack:
		attack.text = str(_char_bean.get_attack())
	var defense: Label = find_child("DefenseValue", true, false)
	if defense:
		defense.text = str(_char_bean.get_defense())
	var health: Label = find_child("HealthValue", true, false)
	if health:
		health.text = "%d/%d" % [int(_char_bean.hp), int(_char_bean.max_hp)]
	var mana: Label = find_child("ManaValue", true, false)
	if mana:
		mana.text = "%d/%d" % [int(_char_bean.mp), int(_char_bean.max_mp)]
	_refresh_material_chip()


## 突破材料徽章：真实突破材料清单（have/need），不再用背包丹药顶替——
## 回春丹等 pill 是治疗消耗品，与突破材料（breakthrough.json）无关
## 重入红线：本函数由 level/exp/attrs/bag_views watch 回调驱动，而
## get_breakthrough_materials 会经 XiuItemState.ins() 触发另一 Bean 的
## update（refresh_bag_views）——watch 回调可能正处在别的 update 借用内，
## 同步调用 = bind_mut 重入 panic，必须 call_deferred 出栈后再执行。
func _refresh_material_chip() -> void:
	_do_refresh_material_chip.call_deferred()


func _do_refresh_material_chip() -> void:
	var chip: Label = find_child("MaterialText", true, false)
	if chip == null:
		return
	var mats: Array = _char_bean.get_breakthrough_materials()
	if mats.is_empty():
		chip.text = "已臻圆满，无需突破"
		return
	var parts: PackedStringArray = []
	for m in mats:
		parts.append("%s %d/%d" % [
			str(m.get("name", "")), int(m.get("have", 0)), int(m.get("need", 0))])
	chip.text = "、".join(parts)


# ---------- 顶栏回调（profile_topbar.gml 的 @pressed，就近解析回退到场景脚本） ----------
func _on_close_pressed() -> void:
	print("[ProfilePanel] 关闭角色面板")
	s_close_requested.emit()
	hide()


# ---------- 中央突破按钮（profile_realm.gml 的 @pressed） ----------
func _on_breakthrough_pressed() -> void:
	# 突破流程（经验体系定义见 state/level/level.gjson）：
	#   阶内 10 段修为圆满（经验攒满）+ 突破材料齐备 → try_breakthrough
	#   扣材料进下一阶一段；任一条件不满足都在面板给出可见提示
	#   （此前失败分支只 print 控制台，表现为"点击没反应"）
	if _char_bean.is_rank_full():
		var lacking: Array = []
		for m in _char_bean.get_breakthrough_materials():
			if int(m.get("have", 0)) < int(m.get("need", 0)):
				lacking.append("%s(%d/%d)" % [
					str(m.get("name", "")), int(m.get("have", 0)),
					int(m.get("need", 0))])
		if not _char_bean.is_breakthrough_exp_ready():
			var need: int = _char_bean.exp_to_next(int(_char_bean.level))
			_hint("修为尚未圆满：经验 %d/%d，攒满后方可冲关突破。" % [int(_char_bean.exp), need])
		elif not lacking.is_empty():
			_hint("突破材料不足：%s。集齐后即可冲关。" % "、".join(lacking))
		elif _char_bean.try_breakthrough():
			_hint("突破成功！迈入「%s」。" % _char_bean.get_realm_title())
			print("[ProfilePanel] 突破成功 → ", _char_bean.get_realm_title())
		else:
			_hint("突破失败，条件异常，请重试。")
	elif _char_bean.get_exp_progress() >= 1.0:
		_char_bean.add_exp(_char_bean.exp_to_next(int(_char_bean.level)))
		_hint("升级成功 → %s。" % _char_bean.get_realm_title())
		print("[ProfilePanel] 升级成功 → ", _char_bean.get_realm_title())
	else:
		_hint("修炼进度 %d%%，未圆满——继续打坐攒修为。" % roundi(_char_bean.get_exp_progress() * 100.0))
		print("[ProfilePanel] 修炼进度 %d%%，未满" % roundi(_char_bean.get_exp_progress() * 100.0))


# ---------- 行内提示（BreakthroughHint，3 秒后还原；范式同 level_panel） ----------
var _hint_default := ""
var _hint_timer: Timer


func _hint(text: String) -> void:
	var lbl: Label = find_child("BreakthroughHint", true, false)
	if lbl == null:
		return
	if _hint_default.is_empty():
		_hint_default = lbl.text
	lbl.text = text
	if _hint_timer == null:
		_hint_timer = Timer.new()
		_hint_timer.one_shot = true
		_hint_timer.wait_time = 4.0
		_hint_timer.timeout.connect(func():
			if is_instance_valid(lbl):
				lbl.text = _hint_default)
		add_child(_hint_timer)
	_hint_timer.start()
