# 道具静态定义查询表 —— 定义数据层（非 GdBean）
#
# 数据源：state/item/*.json（**按分类拆分的多个明文源**，策划可直接改文件调参，
#   新增分类文件无需改代码——加载器扫描目录全量合并）
#   breakthrough.json 等阶突破材料 / pill.json 丹药 / farming.json 灵田种植 /
#   gather.json 采集材料 / equipment.json 装备 / special.json 特殊
#   → 编辑器插件自动加密生成同名 .gjson 产物（本表只读产物）
#   结构：各文件 items{}（id -> 定义）合并为一张总表；字段：
#     基础：id/name/type/rarity/desc/price/stack/icon
#     source 获得方式说明（坊市购买/秘境掉落/采集炼制；手册与界面可直接展示）
#     use=breakthrough  等阶突破材料（to_rank=目标阶 id）
#     use=exp           修为丹（服用获得 exp 点修为）
#
# 设计约定：
#   - 纯静态定义（只读缓存），不注册 GdBean、不持久化——运行期持有量
#     （背包 bag）由 XiuItemState 持有；本表只回答「这个东西是什么」
#   - XiuItemState 的道具总表由此派生（单一数据源，改文件即改全游戏目录）
#   - 物理文件按系统分类，逻辑分类仍是条目的 type 字段（消耗/材料/装备/特殊），
#     get_items_by_type 跨文件聚合
class_name XiuItemTable
extends RefCounted

## 加密产物目录（由各分类 .json 经 GdJsonCodec 生成，勿手改）
const SOURCE_DIR := "res://example/demo/xiuxian/state/item"
## 明文源目录（策划编辑入口；按分类拆分，全部带 pipeline=gjson 标记）

static var _cache: Dictionary = {}


## 加载定义（首次读盘后缓存；force=true 强制重读）。
## 扫描 SOURCE_DIR 下全部 *.gjson 加密产物，解密解析后合并 items 为一张总表；
## 同 id 冲突时后读的覆盖并告警（策划应保证 id 全局唯一）。
static func load_data(force: bool = false) -> Dictionary:
	if not _cache.is_empty() and not force:
		return _cache
	var merged: Dictionary = {"version": 1, "desc": "道具静态定义总表（多文件合并）", "items": {}}
	var files := _list_gjson_files()
	if files.is_empty():
		push_error("[XiuItemTable] 未找到任何加密产物（源目录 %s，重跑 test/regen_gjson.gd 重建）" % SOURCE_DIR)
		_cache = merged
		return _cache
	for path in files:
		var parsed: Variant = JSON.parse_string(_read_decrypted(path))
		if not (parsed is Dictionary):
			push_error("[XiuItemTable] 定义表解析失败: %s（重跑 test/regen_gjson.gd）" % path)
			continue
		var doc: Dictionary = parsed
		for k in doc.keys():
			if k == "items":
				continue
			merged[k] = doc[k]
		var items: Dictionary = doc.get("items", {})
		for iid in items.keys():
			if merged["items"].has(iid):
				push_warning("[XiuItemTable] 道具 id 重复定义，后者覆盖: %s（%s）" % [str(iid), path])
			merged["items"][iid] = items[iid]
	_cache = merged
	return _cache


## 列出定义目录下全部 .gjson 产物（按文件名排序，保证合并顺序稳定）
static func _list_gjson_files() -> PackedStringArray:
	var out := PackedStringArray()
	var dir := DirAccess.open(SOURCE_DIR)
	if dir == null:
		return out
	dir.list_dir_begin()
	var name := dir.get_next()
	while name != "":
		if not dir.current_is_dir() and name.get_extension() == "gjson":
			out.append(SOURCE_DIR.path_join(name))
		name = dir.get_next()
	dir.list_dir_end()
	out.sort()
	return out


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


## 道具获得方式说明（source 字段；无定义返回空串）
static func get_item_source(item_id: String) -> String:
	return str(get_item(item_id).get("source", ""))


## 指定目标阶的突破材料定义列表（[{...定义, need: 数量}, ...]）
static func get_breakthrough_items_for(rank_id: String) -> Array:
	var out: Array = []
	for req in XiuLevelTable.get_breakthrough_items(rank_id):
		var def := get_item(str(req.get("id", "")))
		if def.is_empty():
			push_error("[XiuItemTable] 突破材料 %s 未在定义表内" % str(req.get("id")))
			continue
		var d: Dictionary = def.duplicate(true)
		d["need"] = int(req.get("count", 1))
		out.append(d)
	return out


## 清空缓存（测试用）
static func reset_cache() -> void:
	_cache = {}
