# 道具静态定义查询表 —— 定义数据层（非 GdBean）
#
# 数据源：state/item/item.json（明文源，策划可直接改文件调参）
#   → 编辑器插件自动加密生成 item.gjson 产物（本表只读产物）
#   结构：items{}（id -> 定义）；基础字段
#   （id/name/type/rarity/desc/price/stack），扩展字段：
#     use=breakthrough  等阶突破材料（to_rank=目标阶 id）
#     use=exp           修为丹（服用获得 exp 点修为）
#
# 设计约定：
#   - 纯静态定义（只读缓存），不注册 GdBean、不持久化——运行期持有量
#     （背包 bag）由 XiuItemState 持有；本表只回答「这个东西是什么」
#   - XiuItemState 的道具总表由此派生（单一数据源，改文件即改全游戏目录）
class_name XiuItemTable
extends RefCounted

## 加密产物路径（由 item.json 经 GdJsonCodec 生成，勿手改）
const DATA_PATH := "res://example/demo/xiuxian/state/item/item.gjson"
## 明文源路径（策划编辑入口）
const SOURCE_PATH := "res://example/demo/xiuxian/state/item/item.json"

static var _cache: Dictionary = {}


## 加载定义（首次读盘后缓存；force=true 强制重读）
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
		push_error("[XiuItemTable] 定义表加载失败: %s（源 %s，重跑 test/regen_gjson.gd 重建产物）" % [DATA_PATH, SOURCE_PATH])
		_cache = {}
	return _cache


## 读取加密 .gjson 并解密为明文 JSON 文本（产物管线统一入口）
static func _read_decrypted(gjson_path: String) -> String:
	if not FileAccess.file_exists(gjson_path):
		push_error("[XiuItemTable] 加密产物缺失: %s（重跑 test/regen_gjson.gd）" % gjson_path)
		return ""
	return GdJsonCodec.decrypt_to_text(FileAccess.get_file_as_bytes(gjson_path))


## 全量道具定义（id -> 定义字典；锁定/解锁为运行进度状态，不在定义表）
static func get_items() -> Dictionary:
	return load_data().get("items", {})


## 按 id 取道具定义（无则返回空 Dictionary）
static func get_item(item_id: String) -> Dictionary:
	return get_items().get(item_id, {})


## 全部道具 id 列表
static func get_item_ids() -> Array:
	return get_items().keys()


## 按类型查询（消耗/材料/装备/特殊），返回全量定义
static func get_items_by_type(type_name: String) -> Array:
	var out: Array = []
	for it in get_items().values():
		if str(it.get("type", "")) == type_name:
			out.append(it)
	return out


## 按用途查询（breakthrough=突破材料 / exp=修为丹），返回全量定义
static func get_items_by_use(use_name: String) -> Array:
	var out: Array = []
	for it in get_items().values():
		if str(it.get("use", "")) == use_name:
			out.append(it)
	return out


## 指定目标阶的突破材料定义列表（[{...定义, need: 数量}, ...]）
static func get_breakthrough_items_for(rank_id: String) -> Array:
	var out: Array = []
	for req in XiuLevelTable.get_breakthrough_items(rank_id):
		var def := get_item(str(req.get("id", "")))
		if def.is_empty():
			push_error("[XiuItemTable] 突破材料 %s 未在 item.gjson 定义" % str(req.get("id")))
			continue
		var d: Dictionary = def.duplicate(true)
		d["need"] = int(req.get("count", 1))
		out.append(d)
	return out


## 清空缓存（测试用）
static func reset_cache() -> void:
	_cache = {}
