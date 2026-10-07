# 游戏设置面板控制器 —— 由 settings_panel.gml 的 <ui script="settings_panel.gd"> 声明，
# 构建期自动挂载到 gml 根元素（SettingsPanel）。
#
# 职责划分：
#   · 音量/全屏/垂直同步/画质 = 接口组件（SettingSlider/SettingSwitch/SettingSelect），
#     Rust 侧预定义读写逻辑（bind → GdViewSetting），即时应用并持久化——本脚本零接线
#   · 难度单选/战斗提示多选 = 基础表单组件（FormRadio/FormCheck），
#     业务联动走 GdState 临时状态总线（命令上行 set_state，其他系统 watch 自取）
extends Panel

## 面板关闭请求：独立 F6 运行时仅自身 hide()；组合进 <Modal> 时
## Modal 监听本信号整体关闭（见 ui_modal.rs 内容关闭联动）
signal s_close_requested

## 状态键：游戏难度 / 战斗提示开关（UI 联动等临时状态，与 GdBean 分工）
const KEY_DIFFICULTY := "setting.difficulty"
const KEY_SHOW_DAMAGE := "setting.show_damage"
const KEY_SHOW_DROP := "setting.show_drop"


func _ready() -> void:
	# 基础表单组件没有持久化后端，_ready 时把当前 UI 状态广播一次，
	# 让下游监听方（战斗系统等）拿到初值（组件 GML 已声明默认选中态）
	var radio: Control = find_child("RadioDifficulty", true, false)
	var chk_damage: Control = find_child("ChkDamage", true, false)
	var chk_drop: Control = find_child("ChkDrop", true, false)
	if radio:
		_state().set_state(KEY_DIFFICULTY, radio.get("value"))
	if chk_damage:
		_state().set_state(KEY_SHOW_DAMAGE, chk_damage.get("checked"))
	if chk_drop:
		_state().set_state(KEY_SHOW_DROP, chk_drop.get("checked"))


## 状态总线访问入口
func _state():
	return Engine.get_singleton("GDSTATE")


# ---------- 顶栏回调（settings_panel.gml 的 @pressed，就近解析回退到本脚本） ----------
func _on_close_pressed() -> void:
	print("[SettingsPanel] 关闭设置")
	s_close_requested.emit()
	hide()


# ---------- 基础表单组件回调（@s_value_changed / @s_toggled 就近绑定） ----------
func _on_difficulty_changed(value: String) -> void:
	print("[SettingsPanel] 难度 -> ", value)
	_state().set_state(KEY_DIFFICULTY, value)


func _on_damage_toggled(checked: bool) -> void:
	print("[SettingsPanel] 伤害数字 -> ", checked)
	_state().set_state(KEY_SHOW_DAMAGE, checked)


func _on_drop_toggled(checked: bool) -> void:
	print("[SettingsPanel] 掉落提示 -> ", checked)
	_state().set_state(KEY_SHOW_DROP, checked)
