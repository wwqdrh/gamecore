class_name SProgress
extends GdBean

var times: int = 1 # 是否是第一次打开游戏

static func ins() -> SProgress:
	var res = GdBean.bean("state_progress", func():
		return SProgress.new()#.set_force(true)
	)
	#res.flush()
	return res

func get_update_times():
	update("times", times + 1, {}, false)
