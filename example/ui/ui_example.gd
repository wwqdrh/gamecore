# UI 标记语言示例
# 演示 GdUiBuilder 的用法：解析类 HTML 标记字符串生成 Godot UI 节点树
# 包含样式定义、布局容器、控件、信号绑定、列表扩展节点等功能

extends Control

var ui_builder: GdUiBuilder

func _ready():
	ui_builder = GdUiBuilder.new()

	# 示例1：基础布局 - 垂直排列的标签和按钮
	build_basic_layout()

	# 示例2：带样式的 UI - 使用 <style> 定义 CSS 类样式
	build_styled_ui()

	# 示例3：信号绑定 - 按钮点击事件
	build_signal_binding_ui()

	# 示例4：复杂布局 - 嵌套容器
	build_complex_layout()

	# 示例5：列表扩展节点 - UIHList/UIVList/UIGrid
	build_list_example()

# 基础布局示例
func build_basic_layout():
	var markup = """
	<ui>
		<VBoxContainer anchor="full" margin="20">
			<Label text="基础布局示例" font_size="24" align="center" />
			<Label text="这是一个垂直布局容器" />
			<HSeparator />
			<HBoxContainer>
				<Button text="按钮1" />
				<Button text="按钮2" />
				<Button text="按钮3" />
			</HBoxContainer>
		</VBoxContainer>
	</ui>
	"""
	var ui = ui_builder.parse_string(markup)
	ui.set_position(Vector2(20, 20))
	add_child(ui)

# 带样式的 UI 示例
func build_styled_ui():
	var markup = """
	<ui>
		<style>
			.panel-dark {
				background: #333333;
				border_radius: 8;
				padding: 15;
			}
			.button-primary {
				background: #2e7d32;
				color: white;
				border_radius: 4;
			}
			.title-text {
				color: white;
			}
		</style>
		<VBoxContainer anchor="full" margin="20 200 20 20">
			<Panel class="panel-dark" size="300,150">
				<MarginContainer margin="all:15">
					<VBoxContainer>
						<Label text="带样式的面板" class="title-text" font_size="20" />
						<Label text="使用 style 标签定义样式" class="title-text" />
						<Button text="主按钮" class="button-primary" />
					</VBoxContainer>
				</MarginContainer>
			</Panel>
		</VBoxContainer>
	</ui>
	"""
	var ui = ui_builder.parse_string(markup)
	ui.set_position(Vector2(20, 200))
	add_child(ui)

# 信号绑定示例
func build_signal_binding_ui():
	var markup = """
	<ui>
		<VBoxContainer anchor="full" margin="20 400 20 20">
			<Label text="信号绑定示例" font_size="20" />
			<HBoxContainer margin="0 10 0 0">
				<Button text="点击我" on_pressed="_on_example_button_pressed" />
				<Button text="鼠标悬停" on_mouse_entered="_on_mouse_entered" on_mouse_exited="_on_mouse_exited" />
			</HBoxContainer>
			<Label name="status_label" text="状态: 等待操作..." />
		</VBoxContainer>
	</ui>
	"""
	var ui = ui_builder.parse_string(markup)
	ui.set_position(Vector2(20, 400))
	add_child(ui)
	# 连接信号到当前脚本
	ui_builder.connect_signals(ui, self)

# 复杂布局示例
func build_complex_layout():
	var markup = """
	<ui>
		<VBoxContainer anchor="full" margin="20 550 20 20">
			<Label text="复杂布局示例" font_size="20" />
			<GridContainer columns="2" h_separation="10" v_separation="10">
				<Panel size="140,80">
					<MarginContainer margin="10">
						<VBoxContainer>
							<Label text="卡片1" font_size="16" />
							<Label text="描述文字" font_size="12" />
						</VBoxContainer>
					</MarginContainer>
				</Panel>
				<Panel size="140,80">
					<MarginContainer margin="10">
						<VBoxContainer>
							<Label text="卡片2" font_size="16" />
							<Label text="描述文字" font_size="12" />
						</VBoxContainer>
					</MarginContainer>
				</Panel>
				<Panel size="140,80">
					<MarginContainer margin="10">
						<VBoxContainer>
							<Label text="卡片3" font_size="16" />
							<Label text="描述文字" font_size="12" />
						</VBoxContainer>
					</MarginContainer>
				</Panel>
				<Panel size="140,80">
					<MarginContainer margin="10">
						<VBoxContainer>
							<Label text="卡片4" font_size="16" />
							<Label text="描述文字" font_size="12" />
						</VBoxContainer>
					</MarginContainer>
				</Panel>
			</GridContainer>
			<ProgressBar value="65" min_value="0" max_value="100" size="300,20" />
		</VBoxContainer>
	</ui>
	"""
	var ui = ui_builder.parse_string(markup)
	ui.set_position(Vector2(20, 550))
	add_child(ui)

# 信号回调
func _on_example_button_pressed():
	var label = get_node_or_null("StatusLabel")
	if label:
		label.set_text("状态: 按钮被点击了！")
	print("示例按钮被点击")

func _on_mouse_entered():
	var label = get_node_or_null("StatusLabel")
	if label:
		label.set_text("状态: 鼠标进入")
	print("鼠标进入")

func _on_mouse_exited():
	var label = get_node_or_null("StatusLabel")
	if label:
		label.set_text("状态: 鼠标离开")
	print("鼠标离开")

# 列表扩展节点示例（使用 GML 字符串）
func build_list_example():
	# === 水平列表示例 ===
	var hlist_markup = """
	<ui>
		<UIHList count="5" highlight_mode="1" highlight_color="#ffff00">
			<Button text="Item" custom_minimum_size="80,40" />
		</UIHList>
	</ui>
	"""
	var hlist_ui = ui_builder.parse_string(hlist_markup)
	hlist_ui.set_position(Vector2(20, 700))
	add_child(hlist_ui)

	# 获取列表节点并更新数据
	var hlist = hlist_ui.get_child(0)
	var hdata = []
	for i in range(5):
		hdata.append({"text": "项目 %d" % (i + 1)})
	hlist.update(hdata, true)
	hlist.s_click_item.connect(_on_list_item_click)

	# === 垂直列表示例 ===
	var vlist_markup = """
	<ui>
		<UIVList count="3" highlight_mode="2" highlight_color="#00ffff">
			<Panel custom_minimum_size="200,50">
				<MarginContainer margin="10 5 10 5">
					<Label name="VLabel" text="VItem" />
				</MarginContainer>
			</Panel>
		</UIVList>
	</ui>
	"""
	var vlist_ui = ui_builder.parse_string(vlist_markup)
	vlist_ui.set_position(Vector2(500, 20))
	add_child(vlist_ui)

	var vlist = vlist_ui.get_child(0)
	var vdata = []
	for i in range(3):
		vdata.append({"MarginContainer/VLabel:text": "垂直项 %d" % (i + 1)})
	vlist.update(vdata, true)

	# === 网格列表示例 ===
	var grid_markup = """
	<ui>
		<UIGrid count="6" columns="3" highlight_mode="1" highlight_color="#ff8800">
			<Panel custom_minimum_size="80,80">
				<MarginContainer margin="8">
					<Label name="GLabel" text="GItem" />
				</MarginContainer>
			</Panel>
		</UIGrid>
	</ui>
	"""
	var grid_ui = ui_builder.parse_string(grid_markup)
	grid_ui.set_position(Vector2(500, 300))
	add_child(grid_ui)

	var grid = grid_ui.get_child(0)
	var gdata = []
	for i in range(6):
		gdata.append({"MarginContainer/GLabel:text": "格子 %d" % (i + 1)})
	grid.update(gdata, true)

	# 测试 patch_item 单项更新
	grid.patch_item(0, {"MarginContainer/GLabel:text": "已更新!"})

# 列表点击回调
func _on_list_item_click(node):
	print("列表项被点击: ", node)
