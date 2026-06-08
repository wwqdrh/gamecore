extends Node2D

var progress = SProgress.ins()

func _ready() -> void:
	progress.watch("times", on_times)
	
	GDCORE.set_save_id("2")
	progress.get_update_times()
	#progress.get_update_times()
	#progress.get_update_times()
	
func on_times(v):
	print(v)
