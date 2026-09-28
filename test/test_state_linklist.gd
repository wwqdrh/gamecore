# suite: state - GdDataLinkList 数据链表测试
# 覆盖: from_json/to_json 往返、get_list、has、add_one
# 注意: GdDataLinkList 是 Resource（RefCounted），由引用计数释放，不要手动 free()
# 格式说明: from_json/to_json 接受/返回带 "[GdDataLinkList]" 前缀的 JSON 字符串，
#           内容为 { 键: 数组 } 形式的 Dictionary JSON
extends "res://test/test_case.gd"

const PREFIX := "[GdDataLinkList]"


func test_from_json_and_query() -> void:
	var ll := GdDataLinkList.new()
	var data := '{"inventory": ["sword", "shield", "potion"], "enemies": ["slime"]}'
	ll.from_json(PREFIX + data)

	assert_true(ll.has("inventory"), "from_json 后应有 inventory 键")
	assert_true(ll.has("enemies"), "from_json 后应有 enemies 键")
	assert_false(ll.has("missing"), "不存在的键 has 应为 false")

	var list: Array = ll.get_list("inventory")
	assert_eq(list.size(), 3, "inventory 列表长度应为 3")
	assert_eq(String(list[0]), "sword", "inventory[0] 应为 sword")


func test_add_one_appends() -> void:
	var ll := GdDataLinkList.new()
	ll.from_json(PREFIX + '{"inventory": ["sword"]}')
	ll.add_one("inventory", "apple")

	var list: Array = ll.get_list("inventory")
	assert_eq(list.size(), 2, "add_one 后长度应为 2")
	assert_eq(String(list[1]), "apple", "新增项应在列表末尾")


func test_json_roundtrip() -> void:
	var ll := GdDataLinkList.new()
	ll.from_json(PREFIX + '{"a": [1, 2], "b": ["x"]}')

	var out: String = ll.to_json()
	assert_true(out.begins_with(PREFIX), "to_json 输出应带前缀")
	var parsed: Variant = JSON.parse_string(out.trim_prefix(PREFIX))
	assert_not_null(parsed, "前缀剥离后应为合法 JSON")
	if parsed is Dictionary:
		assert_true(parsed.has("a") and parsed.has("b"), "往返后应保留全部键")
		assert_eq(parsed["a"].size(), 2, "往返后数组长度应一致")


func test_empty_key_access() -> void:
	var ll := GdDataLinkList.new()
	var empty: Array = ll.get_list("no_such_key")
	assert_eq(empty.size(), 0, "不存在的键应返回空数组")

	# 非 JSON 输入应被安全忽略（不崩溃）
	ll.from_json("garbage input")
	assert_false(ll.has("garbage"), "非法输入不应产生数据")
