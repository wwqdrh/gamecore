# 修仙储物袋面板控制器 —— 由 bag_panel.gml 的 <ui script="bag_panel.gd"> 声明，
# 构建期自动挂载到 gml 根元素（BagPanel）。无包装层：本 gml 即完整面板，
# 编辑器生成的 bag_panel.gml.tscn 直开运行即完整可用（信号绑定挂树自动连接）。
#
# 拆分与引用（GML 间直接 <Gml> 引用，无需脚本挂载）：
#   bag_panel.gml          骨架：组合 topbar / categories / grid / detail
#   bag_topbar.gml         卷轴标题 + 灵石/容量徽章 + 关闭按钮（回调回退本脚本）
#   bag_categories.gml+gd  分类列表（高亮由状态驱动）
#   bag_grid.gml+gd        8 列 UIGrid + <script> 48 格数据 + 分类过滤 + 点选上报
#   bag_item.gml+gd        格子条目契约（图标/数量/品质/锁定/选中态）
#   bag_detail.gml+gd      右侧详情卡（监听选中变化自更新）
#
# 跨区块联动 —— GdState 临时状态总线（非持久化，单例 GDSTATE）：
#   · 公用状态只存在状态总线（KEY_* 键），子区块之间互不知道对方
#   · 命令上行：子区块把用户操作写入状态总线（set_state）
#   · 状态下行：各子区块 watch 自己关心的键，值变化时各自更新自己
#   · 与 GdBean 分工：GdBean 存可持久化游戏状态（随存档落盘），
#     GDSTATE 存 UI 联动等临时状态（进程内有效，不落盘）
#   · 红线：watch 回调内禁止同步再调用 GDSTATE 任何方法（重入 panic），
#     需要级联状态时用 call_deferred 包一层
extends Panel

## 面板关闭请求：独立 F6 运行时仅自身 hide()；组合进 <Modal> 时
## Modal 监听本信号整体关闭（见 ui_modal.rs 内容关闭联动）
signal s_close_requested

## 状态键：当前分类（"" = 全部；各子文件同名常量保持一致）
const KEY_CATEGORY := "bag.category"
## 状态键：当前选中物品（完整条目数据字典）
const KEY_SELECTED_ITEM := "bag.selected_item"

## 初始分类（与 bag_grid.gml <script> 的 category 变量一致，供构建期过滤）
const INITIAL_CATEGORY := "pill"


func _ready() -> void:
	# 子区块 _ready 自底向上先于本方法执行（watch 已注册完毕），
	# 此处写入初始分类完成首轮联动；未监听前写入的值由 watch 初始回调带回
	_state().set_state(KEY_CATEGORY, INITIAL_CATEGORY)


## 状态总线访问入口（子区块各自 Engine.get_singleton 亦可）
func _state():
	return Engine.get_singleton("GDSTATE")


# ---------- 顶栏回调（bag_topbar.gml 的 @pressed，就近解析回退到本脚本） ----------
func _on_close_pressed() -> void:
	print("[BagPanel] 关闭储物袋")
	s_close_requested.emit()
	hide()
