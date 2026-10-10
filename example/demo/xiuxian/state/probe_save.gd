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
	return {
		"level": int(level_f), "day": int(day_f),
		"playtime": int(secs_f), "stones": int(stones_f),
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
		print("[Probe] %s -> level=%d day=%d stones=%d playtime=%dh%dm" % [
			f, info["level"], info["day"], info["stones"], h, m])
