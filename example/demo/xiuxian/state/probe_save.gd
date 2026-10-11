# 探针：解密查询默认档与备份档的关键进度字段（只读，不写盘）
# 运行：godot --headless --path . -s res://example/demo/xiuxian/state/probe_save.gd
extends SceneTree

func _initialize() -> void:
	_probe()
	quit(0)


func _read(path: String) -> Dictionary:
	if not FileAccess.file_exists(path):
		print("[Probe] %s 不存在" % path)
		return {}
	var d: GdCoreData = GdCoreData.build(path, "{}", false, "init")
	var level_f: float = d.value("xiuxian_character;level", 0.0, "init")
	var day_f: float = d.value("xiuxian_calendar;day", 0.0, "init")
	var secs_f: float = d.value("total", 0.0, "playtime")
	var stones_f: float = d.value("xiuxian_character;spirit_stones", 0.0, "init")
	# 玩家位置（排查出生在上次位置是否落盘；独立 Bean xiuxian_player）
	var pos_alias: String = d.value("xiuxian_player;pos_alias", "", "init")
	var pos_x: float = d.value("xiuxian_player;pos_x", -1.0, "init")
	var pos_y: float = d.value("xiuxian_player;pos_y", -1.0, "init")
	# 背包（key1=x1 数量；排查挖矿/战利品入包是否落盘）
	var bag: Dictionary = d.value("xiuxian_item;bag", {}, "init")
	var bag_txt := ""
	for k in bag.keys():
		bag_txt += "%s=%d " % [k, int(bag[k])]
	# 地图采集冷却记录（排查挖掉的矿重启后复活；xiuxian_map Bean）
	var harvested: Dictionary = d.value("xiuxian_map;harvested", {}, "init")
	var hv := 0
	for k in harvested.keys():
		if float(harvested[k]) > 0.0:
			hv += 1
	return {
		"level": int(level_f), "day": int(day_f),
		"playtime": int(secs_f), "stones": int(stones_f),
		"bag": bag_txt.strip_edges(),
		"pos": "%s@(%.1f,%.1f)" % [pos_alias, pos_x, pos_y],
		"harvested": hv,
	}


func _probe() -> void:
	var dir := "user://"
	for f in ["coredata.data", "coredata.data-1ymd1x", "coredata.data-aD8P0F",
			"coredata.data-c7V5J1", "coredata_2.data", "coredata_3.data"]:
		var info := _read(dir + f)
		if info.is_empty():
			continue
		var h := int(info["playtime"] / 3600.0)
		var m := int(fmod(info["playtime"], 3600.0) / 60.0)
		print("[Probe] %s -> level=%d day=%d stones=%d playtime=%dh%dm pos=%s harvested_cooling=%d" % [
			f, info["level"], info["day"], info["stones"], h, m, info.get("pos", "?"), info.get("harvested", 0)])
		print("[Probe]   bag: %s" % info.get("bag", ""))
