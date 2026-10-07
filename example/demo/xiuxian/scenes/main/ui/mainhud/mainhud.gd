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
extends Control

## 状态键：当前功能主界面（"cultivate"/"gongfa"/"bag"/"market"）
const KEY_MENU := "mainhud.menu"
## 状态键：当前点选的技能格（键位字符串 "1"~"6"）
const KEY_SKILL := "mainhud.skill"


func _ready() -> void:
	# HUD 覆盖游戏世界：根与三层布局容器鼠标穿透（不影响子交互节点）
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	for layer in ["TopLayer", "TopRow", "MidLayer", "MidRow", "BottomLayer", "BottomColumn"]:
		var c := find_child(layer, true, false) as Control
		if c:
			c.mouse_filter = Control.MOUSE_FILTER_IGNORE
	# 初始功能页（子区块 _ready 自底向上先于本方法执行，watch 已注册完毕）
	_state().set_state(KEY_MENU, "cultivate")


## 状态总线访问入口（子区块各自 Engine.get_singleton 亦可）
func _state():
	return Engine.get_singleton("GDSTATE")


# ---------- 资源栏回调（mainhud_resources.gml 的 @pressed，就近解析回退到本脚本） ----------
func _on_res_add(btn: Control) -> void:
	print("[MainHud] 资源获取入口: ", btn.name)
