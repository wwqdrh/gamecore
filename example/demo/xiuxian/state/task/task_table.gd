# 任务静态定义查询表 —— 定义数据层（非 GdBean）
#
# 数据源：state/task/*.json（**按篇章拆分的多个明文源**，策划可直接改文件调参，
#   新增篇章文件无需改代码——加载器扫描目录全量合并）
#   stage1.json 家族篇（入门前，萧宅+青石镇，结尾=突破斗者+宗门选拔前十）
#   stage2.json 宗门篇（入门后，青云宗，报到/秘境/日常/巡逻/悬赏）
#   → 编辑器插件自动加密生成同名 .gjson 产物（本表只读产物）
#   结构：各文件 tasks{}（id -> 定义）合并为一张总表；基础字段
#   （id/name/category/desc/precondition/steps/giver），扩展字段：
#     objectives[]  完成条件描述（type=talk/kill/goto/collect/rank_up + target_role/desc）
#       collect 步可带 auto:true —— 背包获得该物品时自动推进一步（XiuTaskState 监听）
#     rewards{}     完成奖励（exp/spirit_stones/coins/items[{id,count}]）
#
# 设计约定：
#   - 纯静态定义（只读缓存），不注册 GdBean、不持久化——运行进度
#     （接取/推进/完成）由 XiuTaskState 持有；本表只回答「这个任务是什么」
#   - XiuTaskState 的任务总表由此派生（单一数据源，改文件即改全游戏目录）
#   - 物理文件按剧情篇章拆分，逻辑分类仍是条目的 category 字段
#     （主线/支线/日常/宗门/悬赏），get_tasks_by_category 跨文件聚合
class_name XiuTaskTable
extends RefCounted

## 加密产物目录（由各篇章 .json 经 GdJsonCodec 生成，勿手改）
const SOURCE_DIR := "res://example/demo/xiuxian/state/task"

static var _cache: Dictionary = {}


## 加载定义（首次读盘后缓存；force=true 强制重读）。
## 扫描 SOURCE_DIR 下全部 *.gjson 加密产物，解密解析后合并 tasks 为一张总表；
## 同 id 冲突时后读的覆盖并告警（策划应保证 id 全局唯一）。
static func load_data(force: bool = false) -> Dictionary:
	if not _cache.is_empty() and not force:
		return _cache
	var merged: Dictionary = {"version": 1, "desc": "任务静态定义总表（多文件合并）", "tasks": {}}
	var files := _list_gjson_files()
	if files.is_empty():
		push_error("[XiuTaskTable] 未找到任何加密产物（源目录 %s，重跑 test/regen_gjson.gd 重建）" % SOURCE_DIR)
		_cache = merged
		return _cache
	for path in files:
		var parsed: Variant = JSON.parse_string(_read_decrypted(path))
		if not (parsed is Dictionary):
			push_error("[XiuTaskTable] 定义表解析失败: %s（重跑 test/regen_gjson.gd）" % path)
			continue
		var doc: Dictionary = parsed
		for k in doc.keys():
			if k == "tasks":
				continue
			merged[k] = doc[k]
		var tasks: Dictionary = doc.get("tasks", {})
		for tid in tasks.keys():
			if merged["tasks"].has(tid):
				push_warning("[XiuTaskTable] 任务 id 重复定义，后者覆盖: %s（%s）" % [str(tid), path])
			merged["tasks"][tid] = tasks[tid]
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
		push_error("[XiuTaskTable] 加密产物缺失: %s（重跑 test/regen_gjson.gd）" % gjson_path)
		return ""
	return GdJsonCodec.decrypt_to_text(FileAccess.get_file_as_bytes(gjson_path))


## 全量任务定义（id -> 定义字典）
static func get_tasks() -> Dictionary:
	return load_data().get("tasks", {})


## 按 id 取单个任务定义（无则返回空 Dictionary）
static func get_task(task_id: String) -> Dictionary:
	return get_tasks().get(task_id, {})


## 按分类取任务定义列表（主线/支线/日常/宗门/悬赏），跨篇章聚合
static func get_tasks_by_category(category: String) -> Array:
	var out: Array = []
	for t in get_tasks().values():
		if str(t.get("category", "")) == category:
			out.append(t)
	return out


## 任务定义条数（测试/校验用）
static func count() -> int:
	return get_tasks().size()


## 清空缓存（测试用）
static func reset_cache() -> void:
	_cache = {}
