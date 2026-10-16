# 剧情演出状态 Bean（分类：剧情演出）—— 仙途 demo 游戏状态层
#
# 职责：记录已播完的剧情演出（movie_id -> true），随存档持久化
#       （user://coredata.data，路径 init;xiuxian_movie;played）。
#
# 链路角色：
#   - scenes/main/movie_trigger.gd 进入主场景时据此判定是否自动补播
#     开场剧情（如 prologue 农田醒来）：未播过 → 播，s_movie_finished
#     后 mark_played → 下次进入游戏不再重复播放
#   - 中途退出游戏未播完 → 未置位 → 下次进入自动重播（剧情不丢失）
class_name XiuMovieState
extends GdBean

## 已播完的演出（movie_id -> true；未记录视为未播）
var played: Dictionary = {}


## 注册/获取单例 Bean（GdBean.bean 幂等：重复调用返回同一实例）
static func ins() -> XiuMovieState:
	return GdBean.bean("xiuxian_movie", func(): return new())


# ------------------------------------------------------------------ 查询

## 查询演出是否已播完（未记录 = false）
func is_played(movie_id: String) -> bool:
	return bool(played.get(movie_id, false))


# ------------------------------------------------------------------ 变更

## 标记演出已播完（s_movie_finished 落点；先算新值再 update——
## GdBean.update 同值提前返回不落盘）
func mark_played(movie_id: String) -> void:
	if is_played(movie_id):
		return
	var p: Dictionary = played.duplicate()
	p[movie_id] = true
	update("played", p, {}, false)


## 清除播放记录（重新触发演出用；正常流程不调用）
func clear_played(movie_id: String) -> void:
	if not is_played(movie_id):
		return
	var p: Dictionary = played.duplicate()
	p.erase(movie_id)
	update("played", p, {}, false)


## 还原演示基线（播放记录清空；跨运行确定性测试用）
func reset_demo() -> void:
	update("played", {}, {}, true)
