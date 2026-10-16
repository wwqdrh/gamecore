# 开场剧情触发器 —— 挂 scenes/main/index.tscn（MapRestore 之后）
#
# 职责：进入主场景时检查 XiuMovieState，未播过的剧情演出自动补播。
# 现接线 prologue（农田醒来开场，剧本自含 scene 指令负责开图）：
#   · 未播过 → 动态构造 GdMoviePlayer 播放，s_movie_finished 后
#     mark_played 落盘 → 下次进入游戏不再播放
#   · 中途退出未播完 → 未置位 → 下次进入自动重播
#
# 红线对齐：
#   · change_scene 装配期内禁调组件表/GdSceneRoot 方法 → call_deferred
#     接线（play 内部再经组件表查 MapManager/ViewCamera）
#   · 播放期玩家 set_paused(true) 由 GdMoviePlayer 负责，触发器不碰玩家
extends Node

## 演出 id（XiuMovieState.played 的键）
const MOVIE_ID := "prologue"
## 剧本路径
const MOVIE_PATH := "res://example/demo/xiuxian/movie/prologue/prologue.movie"


func _ready() -> void:
	if XiuMovieState.ins().is_played(MOVIE_ID):
		return
	# 装配期后接线（Player/DialogBox/组件表均已就位）
	call_deferred("_start_movie")


func _start_movie() -> void:
	# 双保险：同帧多入口/快速重进场景时不再重复触发
	if XiuMovieState.ins().is_played(MOVIE_ID):
		return
	var dialog_box: Node = get_node_or_null("../DialogLayer/DialogBox")
	var player: Node = get_node_or_null("../Player")
	if dialog_box == null or player == null:
		push_error("[MovieTrigger] 台词宿主/玩家缺失，跳过开场剧情")
		return
	var movie: Node = GdMoviePlayer.new()
	movie.set_name("MoviePlayer")
	get_parent().add_child(movie)
	movie.call("configure", MOVIE_PATH,
		movie.get_path_to(dialog_box), movie.get_path_to(player))
	movie.connect("s_movie_finished", _on_movie_finished)
	movie.call("play")
	print("[MovieTrigger] 播放开场剧情: %s" % MOVIE_ID)


func _on_movie_finished() -> void:
	XiuMovieState.ins().mark_played(MOVIE_ID)
	print("[MovieTrigger] 开场剧情播完，已记录: %s" % MOVIE_ID)
