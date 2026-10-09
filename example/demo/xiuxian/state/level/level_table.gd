# 等阶经验体系静态查询表 —— 定义数据层（非 GdBean）
#
# 数据源：state/level/level.json（明文源，策划可直接改文件调参）
#   → 编辑器插件自动加密生成 level.gjson 产物（本表只读产物）
#   结构：ranks[]（11 阶）× levels[]（每阶 10 级经验）+ breakthrough（突破道具）
#   路径查询示例（GJson 分号语义）：
#     ranks;0;levels;4        → 斗之气第 5 级升 6 级所需经验
#     rank_index;dou_zhe      → 斗者的阶下标
#     ranks;1;breakthrough;items;0;id → 斗者突破所需首个道具 id
#
# 设计约定：
#   - 纯静态定义（只读缓存），不注册 GdBean、不持久化、不发变更——
#     谁都改不了表内容，运行状态（level/exp）由 XiuCharacterState 持有
#   - 扁平等级模型：全局 1..110 级，rank = (lv-1)/10，段位 = (lv-1)%10+1
class_name XiuLevelTable
extends RefCounted

## 加密产物路径（由 level.json 经 GdJsonCodec 生成，勿手改）
const DATA_PATH := "res://example/demo/xiuxian/state/level/level.gjson"
## 明文源路径（策划编辑入口；产物缺失时按报错指引重建）
const SOURCE_PATH := "res://example/demo/xiuxian/state/level/level.json"
## 每阶层数
const LEVELS_PER_RANK := 10
## 段位中文数字（阶内 1..10 段）
const CHINESE_NUM: Array = ["一", "二", "三", "四", "五", "六", "七", "八", "九", "十"]

static var _cache: Dictionary = {}


## 加载定义（首次读盘后缓存；force=true 强制重读，便于运行时改表调试）
static func load_data(force: bool = false) -> Dictionary:
	if not _cache.is_empty() and not force:
		return _cache
	# .gjson 是 GdJsonCodec 加密产物：读字节 → 解密 → 解析
	# （产物缺失 = 源 .json 新增后未生成，用编辑器打开一次项目或跑
	#   test/regen_gjson.gd 重建）
	var parsed: Variant = JSON.parse_string(_read_decrypted(DATA_PATH))
	if parsed is Dictionary:
		_cache = parsed
	else:
		push_error("[XiuLevelTable] 定义表加载失败: %s（源 %s，重跑 test/regen_gjson.gd 重建产物）" % [DATA_PATH, SOURCE_PATH])
		_cache = {}
	return _cache


## 读取加密 .gjson 并解密为明文 JSON 文本（产物管线统一入口）
static func _read_decrypted(gjson_path: String) -> String:
	if not FileAccess.file_exists(gjson_path):
		push_error("[XiuLevelTable] 加密产物缺失: %s（重跑 test/regen_gjson.gd）" % gjson_path)
		return ""
	return GdJsonCodec.decrypt_to_text(FileAccess.get_file_as_bytes(gjson_path))


## 总阶数（11）
static func get_rank_count() -> int:
	return int(load_data().get("ranks", []).size())


## 按阶 id 取阶定义（含 name/base/levels/breakthrough）
static func get_rank(rank_id: String) -> Dictionary:
	var idx: Variant = load_data().get("rank_index", {}).get(rank_id, -1)
	return get_rank_at(int(idx))


## 按阶下标取阶定义（越界返回空）
static func get_rank_at(order: int) -> Dictionary:
	var ranks: Array = load_data().get("ranks", [])
	if order < 0 or order >= ranks.size():
		return {}
	return ranks[order]


## 扁平等级 → 阶下标（1..110 → 0..10；越界钳制到表尾）
static func rank_order_of(lv: int) -> int:
	var max_order := int(load_data().get("max_rank_order", 0))
	return clampi((lv - 1) / LEVELS_PER_RANK, 0, max_order)


## 扁平等级 → 阶内段位（1..10；越界钳制）
static func level_in_rank_of(lv: int) -> int:
	return clampi((lv - 1) % LEVELS_PER_RANK + 1, 1, LEVELS_PER_RANK)


## 全局等级上限（11 阶 × 10 级 = 110）
static func get_max_level() -> int:
	return (get_rank_count()) * LEVELS_PER_RANK


## 指定全局等级升下一级所需经验（顶阶 10 级 = 满级，返回 0）
static func get_level_exp(lv: int) -> int:
	if lv < 1 or lv >= get_max_level():
		return 0
	var rank := get_rank_at(rank_order_of(lv))
	var levels: Array = rank.get("levels", [])
	var seg := level_in_rank_of(lv)
	if seg < 1 or seg > levels.size():
		return 0
	return int(levels[seg - 1])


## 指定等级是否处于「阶满待突破」（阶内 10 级，且非顶阶满级）
static func is_rank_full(lv: int) -> bool:
	return lv < get_max_level() and lv % LEVELS_PER_RANK == 0


## 指定阶的突破道具需求（[{id, count}, ...]；顶阶返回空）
static func get_breakthrough_items(rank_id: String) -> Array:
	var rank := get_rank(rank_id)
	return rank.get("breakthrough", {}).get("items", [])


## 指定全局等级所在阶的突破道具需求（便于角色状态直查）
static func get_breakthrough_items_at(lv: int) -> Array:
	return get_breakthrough_items(str(get_rank_at(rank_order_of(lv)).get("id", "")))


## 境界显示名（阶名 + 段位，如 "斗者七段"；顶阶满级显示 "斗帝圆满"）
static func get_rank_title_at(lv: int) -> String:
	var rank := get_rank_at(rank_order_of(lv))
	if rank.is_empty():
		return "无名"
	var seg := level_in_rank_of(lv)
	if lv >= get_max_level():
		return "%s圆满" % str(rank.get("name", ""))
	return "%s%s段" % [str(rank.get("name", "")), CHINESE_NUM[seg - 1]]


## 清空缓存（测试用）
static func reset_cache() -> void:
	_cache = {}
