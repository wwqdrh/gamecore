# 任务静态定义查询表 —— 定义数据层（非 GdBean）
#
# 数据源：state/task/task.json（明文源，策划可直接改文件调参）
#   → 编辑器插件自动加密生成 task.gjson 产物（本表只读产物）
#   结构：tasks{}（id -> 定义）；基础字段
#   （id/name/category/desc/unlock/precondition/steps/giver），扩展字段：
#     objectives[]  完成条件描述（type=talk/kill/goto/collect + target_role/desc）
#     rewards{}     完成奖励（exp/spirit_stones/coins/items[{id,count}]）
#
# 设计约定：
#   - 纯静态定义（只读缓存），不注册 GdBean、不持久化——运行进度
#     （接取/推进/完成）由 XiuTaskState 持有；本表只回答「这个任务是什么」
#   - XiuTaskState 的任务总表由此派生（单一数据源，改文件即改全游戏目录）
class_name XiuTaskTable
extends RefCounted

## 加密产物路径（由 task.json 经 GdJsonCodec 生成，勿手改）
const DATA_PATH := "res://example/demo/xiuxian/state/task/task.gjson"
## 明文源路径（策划编辑入口）
const SOURCE_PATH := "res://example/demo/xiuxian/state/task/task.json"

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
		push_error("[XiuTaskTable] 定义表加载失败: %s（源 %s，重跑 test/regen_gjson.gd 重建产物）" % [DATA_PATH, SOURCE_PATH])
		_cache = {}
	return _cache


## 读取加密 .gjson 并解密为明文 JSON 文本（产物管线统一入口）
static func _read_decrypted(gjson_path: String) -> String:
	if not FileAccess.file_exists(gjson_path):
		push_error("[XiuTaskTable] 加密产物缺失: %s（重跑 test/regen_gjson.gd）" % gjson_path)
		return ""
	return GdJsonCodec.decrypt_to_text(FileAccess.get_file_as_bytes(gjson_path))


## 全量任务定义（id -> 定义字典）
static func get_tasks() -> Dictionary:
	var d := load_data()
	return d.get("tasks", {})


## 按 id 取单个任务定义（无则返回空 Dictionary）
static func get_task(task_id: String) -> Dictionary:
	return get_tasks().get(task_id, {})


## 按分类取任务定义列表（主线/支线/日常/宗门/悬赏）
static func get_tasks_by_category(category: String) -> Array:
	var out: Array = []
	for t in get_tasks().values():
		if str(t.get("category", "")) == category:
			out.append(t)
	return out


## 任务定义条数（测试/校验用）
static func count() -> int:
	return get_tasks().size()
