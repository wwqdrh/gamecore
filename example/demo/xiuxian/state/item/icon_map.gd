# 图片资源注册表 —— 图标/贴图的统一管理位置（静态表，不随存档）
#
# 分工：
#   - item.json 等定义表只写**逻辑图片 id**（如 "img_ore_coldiron"），
#     不直接写 emoji 或资源路径
#   - 本表登记图片 id → 具体资源，值两种形态：
#       "res://..." 开头 = 本地资源路径（贴图，UI 用 TextureRect 显示）
#       其他             = 字符图标（emoji，UI 用 Label 显示）
#   - 多个物品可共用同一图片 id（如 break 系列共用一套突破丹药图标）
#   - 未登记的图片 id 回退 FALLBACK（📦），UI 不再各写一套兜底
#
# 换正式美术素材时：只改本表的值（emoji → "res://assets/icons/xxx.png"），
# 定义表与 UI 代码零改动。
class_name XiuIconMap
extends RefCounted

## 未登记图片 id 的兜底图标
const FALLBACK := "📦"

## 图片 id → 图标资源（emoji 或 res:// 贴图路径）
const ICONS := {
	# 矿石
	"img_ore_coldiron": "🪨",
	"img_ore_amethyst": "💎",
	# 药材
	"img_herb_lingzhi": "🍄",
	"img_herb_xueshen": "🌿",
	"img_herb_ninghun": "🌱",
	# 丹药/消耗
	"img_pill_hp": "🧪",
	"img_pill_mp": "💙",
	"img_pill_break": "⚗️",
	"img_pill_exp": "✨",
	# 装备
	"img_tool_pickaxe": "⛏️",
	"img_tool_hoe": "🔨",
	"img_sword_qingfeng": "🗡️",
	"img_robe_yunwen": "🥋",
	# 特殊
	"img_token_sect": "🎖️",
	"img_map_secret": "🗺️",
	# 稀有材料
	"img_mat_longxue": "🩸",
	"img_break_dadoushi": "🧫",
	"img_break_douling": "🫙",
	"img_break_douwang": "🫘",
	"img_break_douhuang": "🪷",
	"img_break_douzong": "🔮",
	"img_break_douzun": "🌑",
	"img_break_dousheng": "🪵",
	"img_break_doudi": "👑",
}


## 解析图片 id → 图标资源（emoji 串或 res:// 贴图路径）
static func resolve(icon_id: String) -> String:
	if ICONS.has(icon_id):
		return str(ICONS[icon_id])
	return FALLBACK


## 解析结果是否为本地贴图路径（true = TextureRect 显示 / false = Label emoji）
static func is_texture(resolved: String) -> bool:
	return resolved.begins_with("res://")
