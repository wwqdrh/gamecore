# 玩家状态 Bean（分类：玩家状态）—— 仙途 demo 存档级玩家运行态
#
# 职责：持有随存档持久化的玩家运行态数据。当前字段：
#   · pos_alias —— 上次所在地图 alias。主场景 map_restore.gd 先按它恢复
#     「进哪张图」（注册表命中则开该图），player.gd 再做图内精确落位
#     （alias 与当前图一致才落位；副本 alias 不在主场景注册表 → 默认图）
#   · pos_x/pos_y —— 上次世界坐标（精确像素，非格子坐标；进图时原样落位，
#     保留格子内亚像素位置；-1 = 无有效记录）
#
# 与 player.gd 的协作：
#   保存：player.gd _process 节流 2s 调 save_pos（update 现值比对天然去重，
#         只在实际移动后落盘）
#   恢复：进图 _apply_map(snap=true) → load_pos → alias 匹配 + 落点格
#         is_walkable 校验 → 原样落位；任一不满足 → 默认出生点
#
# 持久化：走 GdBean 标准链路（user://coredata_{save_id}.data，scope "init"），
# 路径 xiuxian_player;pos_* 。位置属于玩家运行态，独立成 Bean，
# 不混入养成状态（XiuCharacterState）。
class_name XiuPlayerState
extends GdBean

## 上次所在地图 alias（空 = 无记录）
var pos_alias: String = ""
## 上次世界坐标 x（-1 = 无有效记录）
var pos_x: float = -1.0
## 上次世界坐标 y（-1 = 无有效记录）
var pos_y: float = -1.0


## 注册/获取单例 Bean（GdBean.bean 幂等：重复调用返回同一实例）
static func ins() -> XiuPlayerState:
	return GdBean.bean("xiuxian_player", func(): return new())


# ------------------------------------------------------------------ 位置

## 保存玩家位置（地图 alias + 精确世界坐标；同值 update 自动短路不落盘）
func save_pos(alias: String, world: Vector2) -> void:
	update("pos_alias", alias, {}, false)
	update("pos_x", world.x, {}, false)
	update("pos_y", world.y, {}, false)


## 读取上次位置：{"alias": String, "pos": Vector2}；无有效记录返回空字典
func load_pos() -> Dictionary:
	if pos_alias.is_empty() or pos_x < 0.0 or pos_y < 0.0:
		return {}
	return {"alias": pos_alias, "pos": Vector2(pos_x, pos_y)}


## 清空位置记录（测试/重置用；force 绕过同值短路）
func clear_pos() -> void:
	update("pos_alias", "", {}, true)
	update("pos_x", -1.0, {}, true)
	update("pos_y", -1.0, {}, true)
