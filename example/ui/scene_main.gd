# 游戏主界面 GML 控制器 - 继承 GdGmlScene，处理装备栏数据渲染
# Tooltip 显示由 Rust 内置的 tooltip 属性自动处理，无需手动绑定信号
# 数据通过 GdBean 响应式绑定，GML 中 data="bean:scene_main:equip_data" 格式引用
# Bean 属性变更时自动更新绑定的 UI 节点
extends GdGmlScene

var scene_main = SUIMain.ins()

var ui = """
<ui theme="cartoon">
  <style>
    .equip-slot {
      background: $bg_primary;
      border_radius: 4;
      border_color: $border_default;
      border_width: 1;
      padding: 4;
    }
    .equip-slot-icon {
      color: $text_accent;
    }
    .equip-slot-count {
      color: $text_primary;
    }
    .drawer-btn {
      background: $bg_button;
      color: $text_primary;
      border_radius: 4;
      padding: 8;
    }
    .grid-item {
      background: $bg_primary;
      border_radius: 4;
      border_color: $border_default;
      border_width: 1;
      padding: 8;
    }
    .grid-item-name {
      color: $text_primary;
    }
    .grid-item-desc {
      color: $text_secondary;
    }
  </style>
  <Control anchor="full">
    <!-- 右上角抽屉按钮 -->
	<HBoxContainer anchor="top_wide">
	  <Control size_flags_horizontal="expand_fill" />
	  <Button name="DrawerBtn" text="Bag" class="drawer-btn" on_pressed="toggle:InventoryDrawer" mouse_default_cursor_shape="pointing_hand" />
	  <Button name="AddEquipBtn" text="add equip" class="menu-button" custom_minimum_size="30%,6%" on_pressed="_add_equip" mouse_default_cursor_shape="pointing_hand" />
    </HBoxContainer>

	<!-- 底部居中装备栏，tooltip="EquipTooltip" 自动绑定提示框 -->
	<CenterContainer anchor="bottom_wide">
	  <UIHList name="EquipBar" count="8" highlight_mode="1" highlight_color="#ffffff40" fill_mode="0" tooltip="EquipTooltip" data="bean:scene_main:equip_data">
		<MarginContainer class="equip-slot" custom_minimum_size="8%,8%">
          <VBoxContainer>
			<Label text="{{icon}}" class="equip-slot-icon" align="center" font_size="24" />
			<Label text="{{count}}" class="equip-slot-count" align="center" font_size="12" />
          </VBoxContainer>
        </MarginContainer>
      </UIHList>
    </CenterContainer>
  </Control>

  <!-- 装备栏提示框 -->
  <Tooltip name="EquipTooltip" delay="0.3" max_width="250" max_height="100">
	<Label text="{{name}}" font_size="14" />
	<Label text="{{desc}}" font_size="12" />
  </Tooltip>

  <!-- 右侧抽屉面板 -->
  <Drawer name="InventoryDrawer" direction="right" slide_width="35%" drawer_title="Inventory" close_on_overlay="true" animation_duration="0.25">
	<ScrollContainer size_flags_vertical="expand_fill" horizontal="disabled" vertical="auto">
	  <UIGrid name="InventoryGrid" count="12" columns="3" highlight_mode="1" highlight_color="#ffffff30" tooltip="EquipTooltip" data="bean:scene_main:inventory_data">
		<MarginContainer class="grid-item" custom_minimum_size="12%,12%">
          <VBoxContainer>
			<Label text="{{name}}" class="grid-item-name" align="center" font_size="12" />
			<Label text="{{desc}}" class="grid-item-desc" align="center" font_size="10" />
          </VBoxContainer>
        </MarginContainer>
      </UIGrid>
    </ScrollContainer>
  </Drawer>
</ui>
"""

func _ready() -> void:
	load_from_string(ui)
	
	scene_main.watch("equip_data", on_equip_data_change)

func _add_equip():
	scene_main.add_equip()

func on_equip_data_change(v):
	print(v)
