# index.gd —— 顶层引导场景脚本（绑定 example/demo/xiuxian/index.tscn 根节点）
#
# 职责：
#   1. 向 GdConsole 注册调试命令：
#      · play_all_movies()  —— 一次性连播 movie/ 下全部剧情剧本
#        （prologue → c01..c20 → breakthrough → selection_eve，其余按名追加；
#        test_smoke 冒烟剧本排除）。逐段串行：上一段 s_movie_finished 后
#        自动接下一段；不写 XiuMovieState 播放记录（纯预览，不影响存档）。
#      · stop_all_movies()  —— 中止连播并停掉当前段。
#   2. 面板挂载：index.tscn 内已有 ConsolePanel 节点（GdConsolePanel），
#      按 ` 键（toggle_key，可在编辑器 Inspector 改）开合控制台，
#      控制台内输入 play_all_movies() 触发连播。
#
# 红线对齐：
#   · 播放器实例挂在本根节点（跨场景持久），玩家/对话框用绝对 NodePath
#     注入 GdMoviePlayer.configure（同 movie_trigger.gd 范式）
#   · 命令回调只做启动/停止，不放场景装配期；_exit_tree 注销命令防悬垂
extends GdSceneRoot

## 剧本根目录
const MOVIE_DIR := "res://example/demo/xiuxian/movie"
## 控制台命令名
const CMD_PLAY_ALL := "play_all_movies"
const CMD_STOP_ALL := "stop_all_movies"
## 排除的剧本文件夹（冒烟测试用，不属于剧情）
const EXCLUDED_DIRS: PackedStringArray = ["test_smoke"]

## 待播剧本队列（绝对 res 路径）
var _queue: Array[String] = []
## 连播进行中标记
var _playing_all := false
## 当前段的播放器实例
var _current_movie: Node = null


func _enter_tree() -> void:
	# 红线：根脚本 extends GdSceneRoot 时**严禁定义 _ready/_process**——
	# GDScript 虚函数会整体遮蔽 Rust 侧同名虚函数（GdSceneRoot.ready 负责
	# 建 SceneLayer/ViewCamera/组件表/entry_scene 自动进场景，process 驱动
	# 转场状态机，被遮蔽即框架瘫痪）。命令注册放 _enter_tree（Rust 未实现
	# 该虚函数，不冲突）。
	_register_console_commands()


func _exit_tree() -> void:
	# 注销命令，防止控制台悬垂 Callable 指向已释放节点
	if not Engine.has_singleton("GdConsole"):
		return
	var console: Object = Engine.get_singleton("GdConsole")
	console.unregister_command(CMD_PLAY_ALL)
	console.unregister_command(CMD_STOP_ALL)


## 当前段播放器（调试/测试探针用；无连播时为 null）
func _get_current_movie() -> Node:
	return _current_movie


# ------------------------------------------------------------------ 控制台命令

func _register_console_commands() -> void:
	if not Engine.has_singleton("GdConsole"):
		push_warning("[Index] GdConsole 单例不存在，调试命令未注册")
		return
	var console: Object = Engine.get_singleton("GdConsole")
	console.register_command(CMD_PLAY_ALL, _cmd_play_all_movies,
			"连播全部剧情剧本（控制台输入 play_all_movies()）")
	console.register_command(CMD_STOP_ALL, _cmd_stop_all_movies,
			"中止剧本连播（控制台输入 stop_all_movies()）")


func _cmd_play_all_movies() -> String:
	if _playing_all:
		return "连播进行中（剩余 %d 段），可用 stop_all_movies() 中止" % _queue.size()
	var paths := _scan_movie_scripts()
	if paths.is_empty():
		return "未在 %s 下找到 .movie 剧本" % MOVIE_DIR
	if get_tree().get_first_node_in_group("player") == null:
		return "当前场景没有玩家节点，请进入游戏主场景后再连播"
	_queue = paths
	_playing_all = true
	_play_next()
	return "开始连播 %d 段剧本（过程中可 stop_all_movies() 中止）" % paths.size()


func _cmd_stop_all_movies() -> String:
	if not _playing_all and _current_movie == null:
		return "当前没有连播任务"
	_queue.clear()
	_playing_all = false
	if _current_movie != null and is_instance_valid(_current_movie):
		_current_movie.call("stop")
		_current_movie.queue_free()
		_current_movie = null
	return "连播已停止"


# ------------------------------------------------------------------ 连播实现

## 扫描剧本目录，按叙事顺序返回（prologue → c01..c20 → breakthrough →
## selection_eve → 其余按名追加；EXCLUDED_DIRS 排除）
func _scan_movie_scripts() -> Array[String]:
	var found: Array[String] = []
	var dir := DirAccess.open(MOVIE_DIR)
	if dir == null:
		return found
	for folder in dir.get_directories():
		if folder.begins_with(".") or folder in EXCLUDED_DIRS:
			continue
		var sub := DirAccess.open(MOVIE_DIR + "/" + folder)
		if sub == null:
			continue
		for f in sub.get_files():
			if f.ends_with(".movie"):
				found.append(MOVIE_DIR + "/" + folder + "/" + f)
	found.sort_custom(func(a: String, b: String) -> bool:
		return _movie_order_key(a) < _movie_order_key(b))
	return found


## 排序键：prologue=0 段，c01..c20 = 001..020，breakthrough=900，
## selection_eve=901，其余 9xx 按名
func _movie_order_key(path: String) -> String:
	var folder := path.get_base_dir().get_file()
	if folder == "prologue":
		return "000"
	if folder.begins_with("c") and folder.substr(1, 2).is_valid_int():
		return "%03d" % folder.substr(1, 2).to_int()
	if folder == "breakthrough":
		return "900"
	if folder == "selection_eve":
		return "901"
	return "9" + folder


## 播放下一段（上段 s_movie_finished → 延迟一帧后进入）
func _play_next() -> void:
	if not _playing_all:
		return
	if _queue.is_empty():
		_playing_all = false
		_console_print("[play_all_movies] 全部剧本连播完毕")
		return
	var player_node: Node = get_tree().get_first_node_in_group("player")
	if player_node == null:
		_playing_all = false
		_queue.clear()
		_console_print("[play_all_movies] 玩家节点丢失，连播中止")
		return
	var path: String = _queue[0]
	# 台词宿主优先复用主场景 DialogBox（缺失时 rust 侧自动懒创建默认黑框）
	var dialog_box: Node = player_node.get_parent().get_node_or_null("DialogLayer/DialogBox")
	var movie: Node = GdMoviePlayer.new()
	movie.set_name("PlayAllMoviePlayer")
	add_child(movie)
	_current_movie = movie
	var box_path: NodePath = movie.get_path_to(dialog_box) if dialog_box != null else NodePath()
	movie.connect("s_movie_finished", _on_movie_finished.bind(movie))
	movie.call("configure", path, box_path, movie.get_path_to(player_node))
	movie.call("play")
	_console_print("[play_all_movies] 播放 %s（剩余 %d 段）" % [path.get_file(), _queue.size() - 1])


func _on_movie_finished(movie: Node) -> void:
	if not _queue.is_empty():
		_queue.remove_at(0)
	if _current_movie == movie:
		_current_movie = null
	if is_instance_valid(movie):
		movie.queue_free()
	# 本段收尾（快照回迁/玩家解锁）先落地一帧再接下一段
	call_deferred("_play_next")


## GDScript 侧向控制台面板推送进度（GdConsole.console_print，经
## console_output 信号显示；同时留痕 stdout）
func _console_print(text: String) -> void:
	print(text)
	if Engine.has_singleton("GdConsole"):
		# 红线：控制台命令由 GdConsole.execute **同步回调**（execute 持有
		# &mut 借用全程），直调 GdConsole 任何方法都借用重入 panic
		# （"already bound"）→ 必须 call_deferred，待 execute 返回后再发
		Engine.get_singleton("GdConsole").call_deferred("console_print", text)
