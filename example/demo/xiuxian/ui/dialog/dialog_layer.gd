# 对话层 —— 主场景共享对话系统装配（CanvasLayer）
#
# 组合框架能力（rust/src/dialog/ 对话三件套）：
#   - GdDialogue：对话控制节点（共享一个，加入 "__gd_dialogue" 分组）
#     本层不预载 timeline——每个 NPC 触发对话时经 GdDialogTrigger.timeline_path
#     加载自己目录下的专属 timeline（触发器 start_dialog 内自动切换）
#   - DialogBox（dialog_box.gd）：对话框 UI，实现 handle_line 回调，
#     同时充当对话状态存储（@set_flag / condition_fn=has_flag 的落点）
#
# NPC 触发链路：GdDialogTrigger（INTERACT 模式）→ 玩家进入半径按 E →
# register_role_node（Speaker 绑定）→ pause_roles 暂停双方移动互相面向 →
# DialogBox.handle_line 逐行显示 → s_finished 后恢复。
extends CanvasLayer

const DIALOGUE_GROUP := "__gd_dialogue"
const DialogBoxScript := preload("res://example/demo/xiuxian/ui/dialog/dialog_box.gd")

var dialogue: GdDialogue
var dialog_box: CanvasLayer


func _ready() -> void:
	layer = 2  # 高于 UILayer(1)，对话框覆盖在主 UI 之上
	dialogue = GdDialogue.new()
	dialogue.name = "Dialogue"
	add_child(dialogue)
	dialogue.add_to_group(DIALOGUE_GROUP)

	dialog_box = DialogBoxScript.new()
	dialog_box.name = "DialogBox"
	dialog_box.dialogue_path = NodePath("../Dialogue")
	# 嵌套 CanvasLayer 的层级是全局排序的（不随父 CanvasLayer）：
	# DialogBox 默认 layer=1 与 UILayer(1) 同层，会被 mainhud 底部道具栏遮住。
	# 拉到 3：高于 UILayer(1) 与本层(2)，对话框/选项永远置顶。
	dialog_box.layer = 3
	add_child(dialog_box)
