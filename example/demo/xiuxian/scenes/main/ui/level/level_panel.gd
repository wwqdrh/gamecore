# 修炼/功法面板控制器 —— 由 level_panel.gml 的 <ui script="level_panel.gd"> 声明，
# 挂载到面板根节点（LevelPanel）。
#
# 职责：
#   1. 恢复：进入面板时按 XiuCharacterState.skill_tree 快照恢复功法树状态
#      （GdUITree.restore_state，JSON 按 id 匹配；首次进入根节点「道基」已领悟）
#   2. 领悟：点击节点 → 校验（已学/锁定/技能点）→ 扣点 → tone=done
#      → 解锁直接子节点 → 应用属性加成 → dump_state 存回 Bean
#   3. 展示：顶栏境界/技能点/经验随状态刷新
#
# 主流技能树规则：前置已学才可学（锁定态视觉置灰，点击给提示）、
# 技能点消耗 1 点/节点、已学节点绿色（done）、当前选中金色边框。
extends Panel

## 每个节点的领悟效果（一次性入账到 XiuCharacterState）
const BONUS := {
	"gongfa1": {"max_mp": 20},
	"gongfa2": {"max_mp": 40},
	"gongfa3": {"max_mp": 80},
	"gongfa4": {"max_mp": 160},
	"body1": {"max_hp": 50},
	"body2": {"max_hp": 100},
	"body3": {"max_hp": 200},
	"body4": {"max_hp": 400},
	"soul1": {"attr": ["悟性", 2]},
	"soul2": {"attr": ["悟性", 3]},
	"soul3": {"attr": ["悟性", 5]},
	"soul4": {"attr": ["悟性", 8]},
}

var _char: GdBean


func _ready() -> void:
	_char = XiuCharacterState.ins()
	var tree := _tree()
	if tree != null:
		tree.connect("s_node_clicked", _on_node_clicked)
	_restore()
	_refresh()


func _tree() -> Node:
	return get_node_or_null(^"MainMargin/MainColumn/TreeScroll/GongfaTree")


## 恢复功法树状态；首次进入（无快照）标记根节点已领悟并解锁第一层
func _restore() -> void:
	var tree := _tree()
	if tree == null:
		return
	var saved := str(_char.skill_tree)
	if saved.is_empty():
		tree.set_node_state("root", {"learned": true, "tone": "done"})
		_unlock_children("root")
	else:
		tree.restore_state(saved)


## 点击节点：领悟入口（锁定/已学/点数不足均给出行内提示）
func _on_node_clicked(id: String) -> void:
	var tree := _tree()
	if tree == null:
		return
	# 已领悟：只提示，不重复消耗
	var st: Dictionary = tree.get_node_state(id)
	if st.get("learned", false):
		_hint("%s 已领悟，温故而知新。" % _node_text(tree, id))
		return
	# 锁定：前置未领悟
	if tree.is_node_locked(id):
		var parent_id := str(tree.get_node_parent(id))
		_hint("前置功法「%s」尚未领悟。" % _node_text(tree, parent_id))
		return
	# 技能点不足
	if _char.skill_points < 1:
		_hint("技能点不足——提升境界可获得技能点（每升 1 级 +1，突破 +2）。")
		return
	# 领悟：扣点 → 标记 → 解锁子层 → 属性入账 → 存档
	if not _char.spend_skill_point(1):
		return
	tree.set_node_state(id, {"learned": true, "tone": "done"})
	_unlock_children(id)
	_apply_bonus(id)
	_char.save_skill_tree(str(tree.dump_state()))
	_hint("领悟成功：「%s」！" % _node_text(tree, id))
	_refresh()


## 解锁直接子节点（父已领悟即可学）
func _unlock_children(id: String) -> void:
	var tree := _tree()
	if tree == null:
		return
	for child_id in tree.get_node_children(id):
		tree.set_node_locked(child_id, false)


## 领悟效果入账（max_hp/max_mp 直改上限并同步当前值；attrs 走 set_attr）
func _apply_bonus(id: String) -> void:
	var bonus: Dictionary = BONUS.get(id, {})
	if bonus.is_empty():
		return
	if bonus.has("max_hp"):
		var add := int(bonus["max_hp"])
		_char.update("max_hp", _char.max_hp + add, {}, false)
		_char.update("hp", _char.hp + add, {}, false)
	if bonus.has("max_mp"):
		var add_mp := int(bonus["max_mp"])
		_char.update("max_mp", _char.max_mp + add_mp, {}, false)
		_char.update("mp", _char.mp + add_mp, {}, false)
	if bonus.has("attr"):
		var pair: Array = bonus["attr"]
		_char.set_attr(str(pair[0]), int(_char.attrs.get(str(pair[0]), 0)) + int(pair[1]))


func _node_text(tree: Node, id: String) -> String:
	var t := str(tree.get_node_text(id))
	return id if t.is_empty() else t


## 顶栏刷新：境界 / 技能点 / 经验
func _refresh() -> void:
	var realm: Label = get_node_or_null(^"MainMargin/MainColumn/TopBar/MainMargin/TopRow/RealmLabel")
	var points: Label = get_node_or_null(^"MainMargin/MainColumn/TopBar/MainMargin/TopRow/PointsLabel")
	var exp_lbl: Label = get_node_or_null(^"MainMargin/MainColumn/TopBar/MainMargin/TopRow/ExpLabel")
	var bar: ProgressBar = get_node_or_null(^"MainMargin/MainColumn/ExpBar")
	if realm != null:
		realm.text = _char.get_realm_title()
	if points != null:
		points.text = "可用技能点 %d" % _char.skill_points
	if exp_lbl != null:
		var need: int = _char.exp_to_next(_char.level)
		exp_lbl.text = ("经验 %d/%d" % [_char.exp, need]) if need > 0 else "已臻圆满"
	if bar != null:
		var need2: int = _char.exp_to_next(_char.level)
		bar.max_value = maxf(float(need2), 1.0)
		bar.value = float(_char.exp)


## 行内提示（HintLabel 文案反馈，3 秒后还原）
var _hint_default := ""
var _hint_timer: Timer


func _hint(text: String) -> void:
	var lbl: Label = get_node_or_null(^"MainMargin/MainColumn/HintLabel")
	if lbl == null:
		return
	if _hint_default.is_empty():
		_hint_default = lbl.text
	lbl.text = text
	if _hint_timer == null:
		_hint_timer = Timer.new()
		_hint_timer.one_shot = true
		_hint_timer.wait_time = 3.0
		_hint_timer.timeout.connect(func():
			if is_instance_valid(lbl):
				lbl.text = _hint_default)
		add_child(_hint_timer)
	_hint_timer.start()
