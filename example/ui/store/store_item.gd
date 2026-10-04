# 商品卡条目控制器 —— 由 store_item.gml 的 <ui script="store_item.gd"> 自动挂载
# 到条目根节点（GoodsCard），随 UIGrid duplicate 复制到每个条目。
#
# 数据契约：@export 变量 = 条目接受的外部注入字段（列表 update 时同名 key 自动
# 写入脚本实例），编辑 store_item.gml 时对照本列表即知会传入哪些数据。
# 带 setter 的字段注入时联动 UI 状态（角标染色/倒计时显隐/售罄置灰）。
extends Panel

## 品质 → 角标颜色映射（quality 文字由数据传入）
const QUALITY_COLORS := {
	"珍品": Color("#a8443a"),
	"上品": Color("#4a6fa5"),
	"下品": Color("#5e8d4a"),
	"限时": Color("#c47a2a"),
	"极品": Color("#c4922a"),
}
const QUALITY_DEFAULT := Color("#8a7a5a")


@export var quality: String = "":
	set(value):
		quality = value
		if is_node_ready():
			_apply_quality()

@export var icon: String = ""
@export var title: String = ""
@export var price: String = ""

## 划线原价（空串隐藏）
@export var price_old: String = "":
	set(value):
		price_old = value
		if is_node_ready():
			_apply_price_old()

## 限时倒计时（空串隐藏）
@export var countdown: String = "":
	set(value):
		countdown = value
		if is_node_ready():
			_apply_countdown()

## 售罄态：条目置灰 + 印章显示 + 购买按钮禁用
@export var sold_out: bool = false:
	set(value):
		sold_out = value
		if is_node_ready():
			_apply_sold_out()


func _ready() -> void:
	# 列表构建期注入发生在挂树前，setter 被 is_node_ready() 跳过——
	# ready 时统一补偿应用全部状态字段
	_apply_quality()
	_apply_price_old()
	_apply_countdown()
	_apply_sold_out()


func _apply_price_old() -> void:
	var old_label := find_child("PriceOld", true, false) as Label
	if old_label:
		old_label.visible = price_old != ""


func _apply_countdown() -> void:
	var cd := find_child("CountdownLabel", true, false) as Label
	if cd:
		cd.visible = countdown != ""


## 品质角标：竖排文字 + 按品质染色，空串隐藏
func _apply_quality() -> void:
	var tag := find_child("QualityTag", true, false) as Panel
	if tag == null:
		return
	tag.visible = quality != ""
	tag.modulate = QUALITY_COLORS.get(quality, QUALITY_DEFAULT)
	var text_label := find_child("QualityText", true, false) as Label
	if text_label:
		var vertical := ""
		for ch in quality:
			vertical += ch + "\n"
		text_label.text = vertical.strip_edges()


## 售罄联动：置灰 / 印章 / 禁用按钮
func _apply_sold_out() -> void:
	modulate = Color(1, 1, 1, 0.55) if sold_out else Color.WHITE
	var stamp := find_child("SoldOutStamp", true, false) as Control
	if stamp:
		stamp.visible = sold_out
	var btn := find_child("BuyBtn", true, false) as Button
	if btn:
		btn.disabled = sold_out


## 购买按钮回调（@pressed 声明在 store_item.gml，参数自动补绑发出按钮）
func _on_buy_pressed(_btn: Control) -> void:
	if sold_out:
		return
	print("[StoreItem] 购买 %s（%s 灵石）" % [title, price])
