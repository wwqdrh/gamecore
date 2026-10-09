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
#   · XiuItemState（xiuxian_item）→ 突破材料徽章（背包首个消耗类持有）
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
	_char_bean.watch("level", _on_char_changed)
	_char_bean.watch("exp", _on_char_changed)
	_char_bean.watch("hp", _on_char_changed)
	_char_bean.watch("mp", _on_char_changed)
	_char_bean.watch("attrs", _on_char_changed)
	_item_bean.watch("bag_views", _on_bag_changed)


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


## 突破材料徽章：背包首个消耗类道具（无持有则显示提示）
func _on_bag_changed(_value: Variant = null, _metas: Variant = null) -> void:
	var chip: Label = find_child("MaterialText", true, false)
	if chip == null:
		return
	for v in _item_bean.bag_views:
		if str(v.get("category", "")) == "pill":
			chip.text = "%s %s" % [str(v.get("name", "")), str(v.get("count", ""))]
			return
	chip.text = "暂无突破材料"


# ---------- 顶栏回调（profile_topbar.gml 的 @pressed，就近解析回退到场景脚本） ----------
func _on_close_pressed() -> void:
	print("[ProfilePanel] 关闭角色面板")
	s_close_requested.emit()
	hide()


# ---------- 中央突破按钮（profile_realm.gml 的 @pressed） ----------
func _on_breakthrough_pressed() -> void:
	# 突破流程（经验体系定义见 state/level/level.gjson）：
	#   阶内 10 段经验攒满（进度 >= 100%）→ 需要等阶突破：
	#   突破材料齐备 → try_breakthrough 扣材料进下一阶一段；材料不足给提示
	#   阶内未满 → 经验不足以升级，仅提示进度
	var pct: float = _char_bean.get_exp_progress()
	if _char_bean.is_rank_full():
		if _char_bean.try_breakthrough():
			print("[ProfilePanel] 突破成功 → ", _char_bean.get_realm_title())
		else:
			var lacking: Array = []
			for m in _char_bean.get_breakthrough_materials():
				if int(m.get("have", 0)) < int(m.get("need", 0)):
					lacking.append("%s(%d/%d)" % [
						str(m.get("name", "")), int(m.get("have", 0)),
						int(m.get("need", 0))])
			print("[ProfilePanel] 突破材料不足：%s" % "、".join(lacking))
	elif pct >= 1.0:
		_char_bean.add_exp(_char_bean.exp_to_next(int(_char_bean.level)))
		print("[ProfilePanel] 升级成功 → ", _char_bean.get_realm_title())
	else:
		print("[ProfilePanel] 修炼进度 %d%%，未满" % roundi(pct * 100.0))
