# 秘境技能栏控制器 —— 由 explore_skillbar.gml 的 <ui script="explore_skillbar.gd">
# 声明，挂载到技能栏根节点（SkillBar）。自 mainhud_skillbar 迁移（仅探索场景显示）。
#
# 职责（GdState 状态总线模式中的「视图」）：
#   命令上行：点选技能格 -> 读条目 __item_data 取键位 -> 写 KEY_SKILL
#   （再点已选中格 = 取消选中；按键 1-6 与点选同走状态键，见 player.gd）
#   状态下行：watch KEY_SKILL 同步选中高亮（选中态唯一真源 = 状态键）
#   状态下行：每帧从玩家弹幕发射器同步技能位冷却 → 条目灰罩进度条
#   （冷却唯一真源 = player.gd SKILL_PATTERNS / _skill_cd）
#   状态下行：灵气经验条 watch 状态层 XiuCharacterState（bean: xiuxian_character）
#   经验变化 → 填充宽度 + 文本（exp/exp_to_next）
extends VBoxContainer
## 键名沿用 mainhud.skill（玩家侧消费方 watch 同名键，跨场景兼容）
const KEY_SKILL := "mainhud.skill"

var _char_bean: GdBean
## 当前选中键位本地镜像（watch 回调维护；点选切换用它判取/舍）
var _sel := ""
## 冷却数据源（player 节点，组 "player"；跨场景缓存失效自愈）
var _cd_src: Node = null


func _ready() -> void:
	_char_bean = XiuCharacterState.ins()
	_char_bean.watch("exp", _on_exp_changed)
	# watch 注册即回调当前值：进场景时恢复既有选中态（重开/回切场景不丢高亮）
	_state().watch(KEY_SKILL, _on_skill_state)


func _state():
	return Engine.get_singleton("GDSTATE")


## 状态下行：经验变化 → 经验条填充 + 文本（watch 注册即回调当前值）
func _on_exp_changed(_value: Variant = null, _metas: Variant = null) -> void:
	var track: Control = find_child("XpTrack", true, false)
	var fill: Control = find_child("XpFill", true, false)
	var text: Label = find_child("XpText", true, false)
	if track == null or fill == null:
		return
	var pct: float = _char_bean.get_exp_progress()
	fill.custom_minimum_size.x = track.size.x * pct
	if text:
		text.text = "%d/%d" % [int(_char_bean.exp), int(_char_bean.exp_to_next(_char_bean.level))]


## 状态下行：选中键位变化 → 各条目高亮（键位 1-6 与点选两路统一收敛）
func _on_skill_state(value: Variant = null, _key: Variant = null) -> void:
	_sel = str(value) if value != null else ""
	var list: Node = find_child("SkillList", true, false)
	if list == null:
		return
	# 子节点 0 = slot 模板（不参与高亮）；duplicate 条目重名带 @ 后缀，
	# 按子节点序遍历（勿按名找）
	var children := list.get_children()
	for i in children.size():
		if i == 0:
			continue
		var item: Node = children[i]
		if not ("selected" in item):
			continue
		var item_key := _item_key(item)
		# 空键位（数据未注入/模板节点）永远不高亮——否则未选中时
		# 「"" == ""」会把所有格全点亮（契约注入失效时的兜底）
		item.set("selected", item_key != "" and item_key == _sel)


## 每帧冷却下行：读玩家技能位冷却（_skill_cd 记账）→ 各条目 set_cooldown。
## 无冷却技能（total=0）/ 数据源缺失 → 灰罩隐藏（条目侧兜底）
func _process(_delta: float) -> void:
	var list: Node = find_child("SkillList", true, false)
	if list == null:
		return
	var src := _cd_source()
	var children := list.get_children()
	for i in range(1, children.size()):
		var item: Node = children[i]
		if not item.has_method("set_cooldown"):
			continue
		if src == null:
			item.call("set_cooldown", 0.0, 0.0)
			continue
		var st: Vector2 = src.call("skill_cd_state", _item_key(item))
		item.call("set_cooldown", st.x, st.y)


## 冷却数据源解析：player 组首节点（须挂弹幕发射器）；换场景缓存悬垂自愈
func _cd_source() -> Node:
	if _cd_src != null and is_instance_valid(_cd_src):
		return _cd_src
	_cd_src = null
	var players := get_tree().get_nodes_in_group("player")
	if not players.is_empty() and players[0].get("danmaku") != null:
		_cd_src = players[0]
	return _cd_src


## 条目键位解析：契约 @export key 优先，回退 __item_data meta
## （构建期 tscn 若早于 @export 契约生成，key 属性为空、meta 仍是真源）
func _item_key(item: Node) -> String:
	var key := str(item.get("key")) if "key" in item else ""
	if key != "":
		return key
	var data: Dictionary = item.get_meta("__item_data", {})
	return str(data.get("key", ""))


## UIGrid/UIHList 原生点击信号回调（参数 = 条目根节点）：
## 再点已选中格 = 取消选中（toggle）；状态键 update 同值自动短路去重
func _on_skill_clicked(item: Control) -> void:
	var key := _item_key(item)
	if key == "":
		return
	_state().set_state(KEY_SKILL, "" if _sel == key else key)
