# 对话状态 Bean（分类：对话）—— 仙途 demo 游戏状态层
#
# 职责：持有对话进度 flag（角色关系、任务联动、剧情节点等），随存档持久化
#       （user://coredata.data，路径 init;xiuxian_dialog;flags）。
#
# 链路角色：
#   - DialogBox 的 set_flag/has_flag/clear_flag 委托到这里（timeline 中
#     @set_flag:xxx 的落点，跨运行保留 → 对话进度记录的基础）
#   - GdDialogue 的 stage flag 门控（[stage@flagname] / [stage@!flagname]）
#     经 control.has_flag 走到这里 → 已看过的段落/已完成的任务选项自动跳过
#   - XiuTaskState 接取/完成任务时写入 task_<id>_accepted / task_<id>_done
class_name XiuDialogState
extends GdBean

## 对话进度 flag（flag 名 -> true；未记录视为 false）
var flags: Dictionary = {}


## 注册/获取单例 Bean（GdBean.bean 幂等：重复调用返回同一实例）
static func ins() -> XiuDialogState:
	return GdBean.bean("xiuxian_dialog", func(): return new())


# ------------------------------------------------------------------ 查询

## 查询 flag（未记录 = false）
func has_flag(flag: String) -> bool:
	return bool(flags.get(flag, false))


# ------------------------------------------------------------------ 变更

## 设置 flag（timeline 中 @set_flag:xxx 的落点；先算新值再 update——
## GdBean.update 同值提前返回不落盘）
func set_flag(flag: String) -> void:
	if has_flag(flag):
		return
	var f: Dictionary = flags.duplicate()
	f[flag] = true
	update("flags", f, {}, false)


## 清除 flag
func clear_flag(flag: String) -> void:
	if not has_flag(flag):
		return
	var f: Dictionary = flags.duplicate()
	f.erase(flag)
	update("flags", f, {}, false)


## 还原演示基线（进度 flag 清空；跨运行确定性测试用）
func reset_demo() -> void:
	update("flags", {}, {}, true)
