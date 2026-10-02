# 单个任务条目控制器
# 挂载方式：task_item.gml 根标签 <ui script="task_item.gd"> 声明，构建期自动挂到
# 条目根节点（ItemRoot Panel）上——无需在场景或控制器中手动 add script。
# 条目被列表 duplicate 时脚本随节点复制，条目内 @pressed 声明就近绑定到本脚本；
# 列表 update() 重建条目后由 bind_events 自动重连，绑定声明完全收敛在 GML 与本文件内。
extends Panel

# ===== 数据契约（@export 即契约）=====
# 下方 @export 变量即本条目接受的外部注入字段：列表 update(data) 时，
# data 字典中与变量同名的 key 自动写入脚本实例（类型即文档，声明了才注入）。
# 编辑 task_item.gml 时对照此处即可知道条目会收到哪些数据；
# 未在此声明的 key 仍走 GML {{模板}} 绑定与 __item_data meta 兜底。
@export var icon: String = ""
@export var title: String = ""
@export var desc: String = ""
@export var progress: String = ""
@export var reward1: String = ""
@export var reward2: String = ""
@export var btn_text: String = ""
@export var btn_state: String = ""


## 条目按钮回调（task_item.gml ItemBtn 的 @pressed 声明）
## btn: 发出信号的按钮；参数由信号绑定的智能补绑机制自动传入
## 数据直接读本脚本契约变量，无需再沿父链解析 __item_data meta
func _on_task_action(btn: Control) -> void:
	print("[TaskItem] 点击任务按钮: ", title, " state=", btn_state)
