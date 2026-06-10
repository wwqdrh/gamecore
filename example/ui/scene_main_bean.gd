# 游戏主界面数据 Bean - 继承 GdBean，管理装备栏和背包数据
# GML 中通过 data="bean:scene_main:equip_data" 格式引用
# 属性变更时自动触发 watch 回调，更新绑定的 UI 节点
class_name SUIMain
extends GdBean

# 装备栏数据：使用简单 key，GML 中通过 {{key}} 模板语法绑定
# icon/count 绑定到 Label 的 text 属性，name/desc 存储为 meta 供 Tooltip 读取
var equip_data: Array = [
	{
		"icon": "[W]",
		"count": "",
		"name": "Iron Sword",
		"desc": "ATK +10\nA basic iron sword.",
	},
	{
		"icon": "[S]",
		"count": "",
		"name": "Wooden Shield",
		"desc": "DEF +5\nA simple wooden shield.",
	},
	{
		"icon": "[H]",
		"count": "",
		"name": "Leather Helmet",
		"desc": "DEF +3\nLightweight head protection.",
	},
	{
		"icon": "[A]",
		"count": "",
		"name": "Chain Armor",
		"desc": "DEF +8\nSturdy chain mail armor.",
	},
	{
		"icon": "[R]",
		"count": "x3",
		"name": "Health Potion",
		"desc": "Restore 50 HP\nA red glowing potion.",
	},
	{
		"icon": "[?]",
		"count": "",
		"name": "",
		"desc": "Empty slot",
	},
]

# 背包数据
var inventory_data: Array = [
	{"name": "Sword", "desc": "ATK+10"},
	{"name": "Shield", "desc": "DEF+5"},
	{"name": "Helmet", "desc": "DEF+3"},
	{"name": "Armor", "desc": "DEF+8"},
	{"name": "Potion", "desc": "HP+50"},
	{"name": "Ring", "desc": "SPD+2"},
	{"name": "Scroll", "desc": "MAG+5"},
	{"name": "Gem", "desc": "ALL+1"},
	{"name": "", "desc": ""},
	{"name": "", "desc": ""},
	{"name": "", "desc": ""},
	{"name": "", "desc": ""},
]

static func ins() -> GdBean:
	var res = GdBean.bean("scene_main", func():
		return SUIMain.new()
	)
	return res
