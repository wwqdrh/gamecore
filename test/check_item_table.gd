# 道具定义表（分类多文件）验收 —— XiuItemTable 目录合并加载 + source 获得方式
#
# 运行：perl -e 'alarm 90; exec @ARGV' godot --headless --path . \
#   -s res://test/check_item_table.gd
#
# 覆盖：
#   1. 多文件合并：目录下全部分类 .gjson 加载为一张总表（34 条全量、跨文件无遗漏）
#   2. source 获得方式字段：全部道具非空；突破材料与教程手册口径一致
#   3. 逻辑分类跨文件聚合：get_items_by_type / get_items_by_use
#   4. 突破链路：get_breakthrough_items_for 按 level 表数量挂 need
#   5. 缓存与重读：force=true 重读结果一致
extends SceneTree

var ok := true


func check(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[ItemTable] " + msg)
		ok = false


func _initialize() -> void:
	_run()


func _run() -> void:
	var table: GDScript = load("res://example/demo/xiuxian/state/item/item_table.gd")
	table.reset_cache()

	# ---- 1. 多文件合并 ----
	var items: Dictionary = table.get_items()
	check(items.size() == 34, "多文件合并应为 34 条（实际 %d）" % items.size())
	# 每个分类文件至少一条代表条目被合并进来
	for probe_id in ["break_qizhe", "pill_hp", "seeds_veg", "ore_coldiron", "sword_qingfeng", "token_sect"]:
		check(items.has(probe_id), "合并表应含 %s" % probe_id)
	check(str(table.get_item("herb_ninghun").get("name")) == "凝魂草",
		"突破材料凝魂草（breakthrough.json）应可直查")

	# ---- 2. source 获得方式字段 ----
	var missing_src: Array = []
	for iid in items.keys():
		if str(items[iid].get("source", "")).is_empty():
			missing_src.append(str(iid))
	check(missing_src.is_empty(), "全部道具应有非空 source（缺: %s）" % ", ".join(missing_src))
	# 突破材料获得方式与手册《能力体系》三类正途对齐（坊市/秘境/采集炼制）
	check(str(table.get_item_source("break_qizhe")).contains("坊市"), "聚气散获得方式应含坊市")
	check(str(table.get_item_source("herb_ninghun")).contains("采集"), "凝魂草获得方式应含采集")
	check(str(table.get_item_source("mat_longxue")).contains("掉落"), "龙血精获得方式应含掉落")
	# 便捷查询
	check(str(table.get_item_source("pill_hp")).contains("坊市"), "get_item_source 便捷查询应生效")

	# ---- 3. 逻辑分类跨文件聚合 ----
	var bt: Array = table.get_items_by_use("breakthrough")
	check(bt.size() == 12, "突破材料应跨文件聚合出 12 条（实际 %d）" % bt.size())
	var mat: Array = table.get_items_by_type("材料")
	check(mat.size() >= 10, "材料类型应跨文件聚合（实际 %d）" % mat.size())
	var equip: Array = table.get_items_by_type("装备")
	check(equip.size() == 5, "装备应为 5 条（实际 %d）" % equip.size())

	# ---- 4. 突破链路（need 数量挂载；表挂在「当前大阶」id 上=突破出去所需）----
	var for_qizhe: Array = table.get_breakthrough_items_for("dou_zhiqi")
	check(for_qizhe.size() == 1, "斗之气突破应只需 1 种材料")
	if not for_qizhe.is_empty():
		check(str(for_qizhe[0].get("id")) == "break_qizhe", "斗之气突破材料应为聚气散")
		check(int(for_qizhe[0].get("need")) == 3, "聚气散需求应为 ×3（实际 %s）" % str(for_qizhe[0].get("need")))
		check(not str(for_qizhe[0].get("source", "")).is_empty(), "突破材料定义应带获得方式")
	var for_dadoushi: Array = table.get_breakthrough_items_for("da_dou_shi")
	check(for_dadoushi.size() == 2, "大斗师突破应需 2 种材料（玄玉膏+凝魂草）")

	# ---- 5. 缓存与重读 ----
	var again: Dictionary = table.get_items()
	check(again.size() == 34, "缓存路径二次读取应一致")
	table.reset_cache()
	var reread: Dictionary = table.get_items()
	check(reread.size() == 34, "force 重读后应仍 34 条")

	print("[ItemTable] 全部通过" if ok else "[ItemTable] 存在失败项")
	quit(0 if ok else 2)
