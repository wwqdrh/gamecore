extends Control
# 引导与演出层（mainhud）：beat0 教学 / 突破演出 / 通关字幅 / 传送门控提示
#
# 职责只有接线，布局与动效全在 rust 组件（ui_toast.rs / ui_banner.rs）：
#   · 教学：首次进主场景（无 tutorial_shown flag）连发 4 条操作提示，
#     开播即置 flag（中途退出不重放；s_toast_done 后无需补挂）
#   · 突破演出：_process 0.25s 轮询 XiuCharacterState.level 快照（不走
#     watch——回调内禁同步调 Bean，轮询是最安全的惰性方案），升大级播 Banner
#   · 通关字幅：dlg.has_flag("stage1_clear") 首次为真播收束字幅
#   · 门控提示：player.s_portal_locked → Toast（树查找 + 惰性重试，同
#     mainhud_minimap 的晚到自愈范式）
# 取子节点一律 GdUIManager.get_gml_node（免写长路径）。

const TICK := 0.25

## 教学提示（beat0，队列逐条播放）
const TUTORIAL_HINTS := [
	"左键点击地面即可移动",
	"点击农田可耕地浇水，作物成熟后点击收获",
	"走近 NPC 点击可对话，主线任务找萧老爷",
	"右上角小地图可点击打开世界舆图",
]

@onready var _toast: Control = GdUIManager.get_gml_node(self, "HintToast")
@onready var _banner: Control = GdUIManager.get_gml_node(self, "PerfBanner")

var _player: Node = null
var _prev_title := ""
var _clear_shown := false
var _acc := 0.0


func _ready() -> void:
	# 红线：mainhud _ready 处在 change_scene 装配期内，重活全部延到 _process
	# 惰性补齐；教学提示延到帧末（等 player/flag 均就绪）
	_start_tutorial.call_deferred()


func _process(delta: float) -> void:
	_acc += delta
	if _acc < TICK:
		return
	_acc = 0.0
	_tick_stage_watching()
	_ensure_player()


## 教学 4 连提示：开播即置 flag（持久化，重进不重放）
func _start_tutorial() -> void:
	if _toast == null:
		return
	var dlg := XiuDialogState.ins()
	if dlg.has_flag("tutorial_shown") or dlg.has_flag("met_xiao_master"):
		return
	dlg.set_flag("tutorial_shown")
	for hint in TUTORIAL_HINTS:
		_toast.call("show_toast", hint)


## 突破演出 + 通关字幅（轮询快照，避免 watch 回调内碰 Bean 的重入红线）
## 突破按「等阶头衔变化」判定（跨大阶才播；斗之气内升段不播，防止
## 1→10 段连灌 9 条演出队列）
func _tick_stage_watching() -> void:
	var ch := XiuCharacterState.ins()
	var title := str(ch.get_realm_title())
	if _prev_title != "" and title != _prev_title:
		_banner.call("play_banner", "突破成功", "气感圆融，迈入%s" % title)
	_prev_title = title
	var dlg := XiuDialogState.ins()
	if not _clear_shown and dlg.has_flag("stage1_clear"):
		_clear_shown = true
		_banner.call("play_banner", "第一阶段 · 完", "突破斗者，前路渐明——青草岭已向你敞开")


## 玩家晚到自愈：找到后接 s_portal_locked（只接一次）
func _ensure_player() -> void:
	if _player != null and is_instance_valid(_player):
		return
	var found: Node = get_tree().root.find_child("Player", true, false)
	if found == null:
		return
	_player = found
	_player.connect("s_portal_locked", _on_portal_locked)
	# 初始等阶头衔快照（避免进场景第一帧误判突破）
	_prev_title = str(XiuCharacterState.ins().get_realm_title())


## 传送点门控命中：目标图未解锁
func _on_portal_locked(_target_alias: String) -> void:
	if _toast == null:
		return
	_toast.call("show_toast", "此地尚未开放——先突破至斗者境界，再来探索")
