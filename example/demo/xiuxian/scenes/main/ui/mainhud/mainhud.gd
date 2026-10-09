# 修仙主界面 HUD 控制器 —— 由 mainhud.gml 的 <ui script="mainhud.gd"> 声明，
# 构建期自动挂载到 gml 根元素（MainHud）。无包装层：mainhud.gml.tscn 直开运行
# 即完整可用（信号绑定挂树自动连接）。
#
# 拆分与引用（GML 间直接 <Gml> 引用，无需脚本挂载）：
#   mainhud.gml            骨架：三层布局组合各区块
#   mainhud_player.gml     左上：头像 + 境界卷轴 + 等级
#   mainhud_resources.gml  顶中：灵石/丹药资源徽章 + 加号（回调回退本脚本）
#   mainhud_quest.gml      左中：任务卷轴横幅
#   mainhud_minimap.gml    右上：小地图
#   mainhud_menus.gml+gd   右侧功能按钮列（高亮由状态驱动）
#   mainhud_skillbar.gml+gd 底部技能栏 + 灵气经验条
#   skill_item.gml+gd      技能格条目契约（图标/键位/冷却）
#
# 跨区块联动 —— GdState 临时状态总线（非持久化，单例 GDSTATE）：
#   · 命令上行：功能按钮/技能格点击 set_state 写入状态总线
#   · 状态下行：监听方 watch 同名键各自更新（如菜单高亮）
#   · 红线：watch 回调内禁止同步再调用 GDSTATE 任何方法（重入 panic）
#
# 状态层数据源（GdBean 持久化状态，example/demo/xiuxian/state/）：
#   · XiuCharacterState（xiuxian_character）→ 玩家徽章（境界/等级/经验）
#     + 资源栏灵石
#   · XiuItemState（xiuxian_item）→ 资源栏丹药数（背包消耗类总量）；
#     ＋ 按钮真实写入状态（加灵石/加丹药，全 UI 响应式联动）
#   · XiuTaskState（xiuxian_task）→ 任务卷轴横幅（首个进行中任务进度）
extends Control
## 状态键：当前功能主界面（"cultivate"/"gongfa"/"bag"/"market"）
const KEY_MENU := "mainhud.menu"
## 状态键：当前点选的技能格（键位字符串 "1"~"6"）
const KEY_SKILL := "mainhud.skill"

## 资源按钮动作：每次点击加的灵石数
const STONE_GAIN := 100

var _char_bean: GdBean
var _item_bean: GdBean
var _task_bean: GdBean


func _ready() -> void:
	# 自适应分辨率：按 game_config.json display 配置启用 content scale
	# （headless 测试自动跳过；GdSceneRoot 流程已应用，此处覆盖直开组合根的启动路径）
	GdDisplayFit.apply_display_fit(false)
	# HUD 覆盖游戏世界：根与三层布局容器鼠标穿透（不影响子交互节点）
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	for layer in ["TopLayer", "TopRow", "MidLayer", "MidRow", "BottomLayer", "BottomColumn"]:
		var c := find_child(layer, true, false) as Control
		if c:
			c.mouse_filter = Control.MOUSE_FILTER_IGNORE
	# 初始功能页（子区块 _ready 自底向上先于本方法执行，watch 已注册完毕）
	_state().set_state(KEY_MENU, "cultivate")
	# 状态层数据绑定（watch 注册即回调当前值，无需手动刷初始 UI）
	_bind_game_state()


## 状态总线访问入口（子区块各自 Engine.get_singleton 亦可）
func _state():
	return Engine.get_singleton("GDSTATE")


# ---------- 状态层数据绑定 ----------

func _bind_game_state() -> void:
	_char_bean = XiuCharacterState.ins()
	_item_bean = XiuItemState.ins()
	_task_bean = XiuTaskState.ins()
	_char_bean.watch("level", _on_char_changed)
	_char_bean.watch("exp", _on_char_changed)
	_char_bean.watch("attrs", _on_char_changed)
	_char_bean.watch("spirit_stones", _on_stones_changed)
	_item_bean.watch("bag_views", _on_bag_changed)
	_task_bean.watch("views", _on_task_views_changed)


## 玩家徽章：境界标题 / 等级 / 经验条（注册即回调当前值）
func _on_char_changed(_value: Variant = null, _metas: Variant = null) -> void:
	var realm: Label = find_child("RealmText", true, false)
	if realm:
		realm.text = _char_bean.get_realm_title()
	var lv: Label = find_child("LevelText", true, false)
	if lv:
		lv.text = "Lv.%d" % int(_char_bean.level)
	var pct: float = _char_bean.get_exp_progress()
	var fill: Control = find_child("LevelFill", true, false)
	if fill:
		fill.custom_minimum_size.x = 90.0 * pct
	var pct_label: Label = find_child("LevelPct", true, false)
	if pct_label:
		pct_label.text = "%d%%" % roundi(pct * 100.0)


## 资源栏灵石（千分位展示）
func _on_stones_changed(_value: Variant = null, _metas: Variant = null) -> void:
	var num: Label = find_child("StoneNum", true, false)
	if num:
		num.text = _fmt_thousands(int(_char_bean.spirit_stones))


## 资源栏丹药：背包消耗类总量
func _on_bag_changed(_value: Variant = null, _metas: Variant = null) -> void:
	var num: Label = find_child("PillNum", true, false)
	if num == null:
		return
	var total := 0
	for v in _item_bean.bag_views:
		if str(v.get("category", "")) == "pill":
			total += int(str(v.get("count", "x0")).trim_prefix("x"))
	num.text = "x%d" % total


## 任务横幅：只反映玩家已接取的任务——首个进行中（accepted，名称+步数进度），
## 无则首个已完成待领取（completed），再无则提示文案
## （未接取 available/locked 不上横幅；submitted 已领奖不占横幅）
func _on_task_views_changed(_value: Variant = null, _metas: Variant = null) -> void:
	var text: Label = find_child("QuestText", true, false)
	if text == null:
		return
	var shown := {}
	for v in _task_bean.views:
		if str(v.get("status", "")) == XiuTaskState.STATUS_ACCEPTED:
			shown = v
			break
	var claimable := false
	if shown.is_empty():
		for v in _task_bean.views:
			if str(v.get("status", "")) == XiuTaskState.STATUS_COMPLETED:
				shown = v
				claimable = true
				break
	if shown.is_empty():
		text.text = "暂无进行中的任务"
	elif claimable:
		text.text = "可领取：%s" % str(shown.get("title", ""))
	else:
		text.text = "%s %s" % [str(shown.get("title", "")), str(shown.get("progress", ""))]


func _fmt_thousands(v: int) -> String:
	var s := str(v)
	var out := ""
	while s.length() > 3:
		out = "," + s.substr(s.length() - 3) + out
		s = s.substr(0, s.length() - 3)
	return s + out


# ---------- 资源栏回调（mainhud_resources.gml 的 @pressed，就近解析回退到本脚本） ----------
## ＋ 按钮真实写状态层：灵石 +STONE_GAIN / 回春丹 +1（全 UI 响应式联动）
func _on_res_add(btn: Control) -> void:
	if btn.name == StringName("AddPillBtn"):
		_item_bean.add_item("pill_hp", 1)
	else:
		_char_bean.add_spirit_stones(STONE_GAIN)
