<ui>
  <style>
    .equip-slot {
      background: #1a1a3e;
      border_radius: 4;
      border_color: #3a3a6e;
      border_width: 1;
      padding: 4;
    }
    .equip-slot-icon {
      color: #88aaff;
    }
    .equip-slot-count {
      color: #ccccee;
    }
    .drawer-btn {
      background: #2a2a4e;
      color: #ccccee;
      border_radius: 4;
      padding: 8;
    }
    .grid-item {
      background: #1a1a3e;
      border_radius: 4;
      border_color: #3a3a6e;
      border_width: 1;
      padding: 8;
    }
    .grid-item-name {
      color: #ccccee;
    }
    .grid-item-desc {
      color: #888899;
    }
  </style>
  <Control anchor="full">
    <!-- 右上角抽屉按钮 -->
    <HBoxContainer anchor="top_wide">
      <Control size_flags_horizontal="expand_fill" />
      <Button name="DrawerBtn" text="Bag" class="drawer-btn" on_pressed="toggle:InventoryDrawer" mouse_default_cursor_shape="pointing_hand" />
    </HBoxContainer>

    <!-- 底部居中装备栏，tooltip="EquipTooltip" 自动绑定提示框 -->
    <CenterContainer anchor="bottom_wide">
      <UIHList name="EquipBar" count="6" highlight_mode="1" highlight_color="#ffffff40" fill_mode="0" tooltip="EquipTooltip" data="bean:scene_main:equip_data">
        <MarginContainer class="equip-slot" custom_minimum_size="64,64">
          <VBoxContainer>
            <Label text="{{icon}}" class="equip-slot-icon" align="center" font_size="24" />
            <Label text="{{count}}" class="equip-slot-count" align="center" font_size="12" />
          </VBoxContainer>
        </MarginContainer>
      </UIHList>
    </CenterContainer>
  </Control>

  <!-- 装备栏提示框 -->
  <Tooltip name="EquipTooltip" delay="0.3" max_width="250" />

  <!-- 右侧抽屉面板 -->
  <Drawer name="InventoryDrawer" direction="right" slide_width="360" drawer_title="Inventory" close_on_overlay="true" animation_duration="0.25">
    <ScrollContainer size_flags_vertical="expand_fill" horizontal="disabled" vertical="auto">
      <UIGrid name="InventoryGrid" count="12" columns="3" highlight_mode="1" highlight_color="#ffffff30" tooltip="EquipTooltip" data="bean:scene_main:inventory_data">
        <MarginContainer class="grid-item" custom_minimum_size="96,96">
          <VBoxContainer>
            <Label text="{{name}}" class="grid-item-name" align="center" font_size="12" />
            <Label text="{{desc}}" class="grid-item-desc" align="center" font_size="10" />
          </VBoxContainer>
        </MarginContainer>
      </UIGrid>
    </ScrollContainer>
  </Drawer>
</ui>
