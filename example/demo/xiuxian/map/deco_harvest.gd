# 矿石采集回调宿主（demo 侧）—— 挂在地图场景内（xiaozhai.tscn / xiaozhen.tscn
# 的 DecoHarvest 节点），GdDecoLayer 配置 deco_callbacks 指向本节点方法。
#
# 分工（框架零业务感知）：
#   - 框架 GdDecoLayer：铺设矿石（跳过已采集格）、受击计数、定时随机补种、
#     血量归零 → 按配置的函数名回调
#   - demo（本脚本）：采集入库 + 采集/再生状态随档持久化——
#     · 进场：XiuMapState.harvested（值 = 采集时刻游戏秒）→ 到期判定 →
#       未到期格子 set_harvested_cells 注入 + set_regrow_elapsed 预推进
#       再生时钟（重启不重置节拍）；到期的弹出（矿本次铺设自然长回）
#     · 配置：XiuGameConfigState.deco_regrow_secs（默认 120s）→ layer 再生间隔
#     · 采集：读矿石 deco_cell meta → XiuMapState.add_harvested（存当前游戏秒）
#     · 再生：s_regrowned → XiuMapState.remove_harvested
#
# 回调签名（框架约定）：func <deco_callbacks 里的名字>(item_id: String, node: Node)
extends Node

var _layer: Node = null


func _ready() -> void:
	# DecoLayer 与本节点同为图根直挂（../DecoLayer）
	_layer = get_node_or_null("../DecoLayer")
	if _layer == null:
		return
	# 存档级再生间隔下发（默认 120s = 2 分钟长出下一批）；
	# watch 配置变化实时重下发（红线：回调内不同步调 Bean → call_deferred）
	_apply_regrow_interval()
	XiuGameConfigState.ins().watch(
		"deco_regrow_secs",
		func(_v = null, _m = null): _apply_regrow_interval.call_deferred())
	_restore_harvested()
	_layer.s_regrowned.connect(_on_regrowned)


## 进场恢复：把存档里的冷却记录注入 deco 层（spawn_deco 是 call_deferred，
## 本注入发生在同一帧更早时刻——顺序安全：注入先于铺设）。
## 记录值 = 采集时刻游戏秒：到期的弹出（矿本次长回），未到期的注入并按
## 最早采集格预推进再生时钟——重启/换图不再把冷却节拍清零。
## 旧格式值 1.0（标记位）视为早已到期的时刻 → 弹出（矿回一次，无害迁移）。
func _restore_harvested() -> void:
	var interval := maxf(float(XiuGameConfigState.ins().deco_regrow_secs), 0.0)
	var now: float = float(XiuCalendarState.ins().game_secs)
	var cooling: Array = []  # [采集秒, key] 升序排列后注入 = FIFO 先弹最早采集
	for k in XiuMapState.ins().get_harvested().keys():
		var t := float(XiuMapState.ins().get_harvested()[k])
		if t <= 0.0:
			continue  # 已弹出
		if interval > 0.0 and now - t >= interval:
			XiuMapState.ins().remove_harvested(int(str(k)))
			continue  # 冷却已过（含旧格式迁移）→ 矿本次铺设自然长回
		cooling.append([t, int(str(k))])
	cooling.sort_custom(func(a, b): return a[0] < b[0])
	var keys := PackedInt64Array()
	for pair in cooling:
		keys.append(pair[1])
	_layer.set_harvested_cells(keys)
	if interval > 0.0 and not cooling.is_empty():
		# 最早采集格已冷却 age 秒 → 首个补种 tick 在 interval - age 后触发
		_layer.set_regrow_elapsed(clampf(now - float(cooling[0][0]), 0.0, interval))


## 存档级配置 → deco 层再生间隔
func _apply_regrow_interval() -> void:
	if _layer != null:
		_layer.deco_regrow_interval = maxf(float(XiuGameConfigState.ins().deco_regrow_secs), 0.0)


## 采集完成：物品入包 + 记录已采集格子（随档持久化，框架同步进再生冷却队列；
## 值 = 当前游戏秒，重开后据此换算冷却进度）
func on_ore_harvested(item_id: String, node: Node) -> void:
	print("[DecoHarvest] 采集获得: %s" % item_id)
	XiuItemState.ins().add_item(item_id, 1)
	if node != null and node.has_meta("deco_cell"):
		var cell: Vector2i = node.get_meta("deco_cell")
		var key: int = (maxi(cell.x, 0) << 32) | maxi(cell.y, 0)
		XiuMapState.ins().add_harvested(key, float(XiuCalendarState.ins().game_secs))


## 框架补种成功：弹出该格冷却记录（缺口集合守恒 = 初始数 - 存活数）
func _on_regrowned(_item_id: String, cell_key: int) -> void:
	XiuMapState.ins().remove_harvested(cell_key)
