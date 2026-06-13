# 角色界面 GML 控制器 - 继承 GdGmlScene，使用 PopupPanel 实现角色属性面板
# 中间按钮点击弹出角色面板，面板左为装备区（三列：左侧装备/立绘/右侧装备），面板右为背包网格（5x5分页）
extends GdGmlScene

const ITEMS_PER_PAGE = 25

var current_page = 0
var total_pages = 1

var bag_data = []

var UI = """
<ui theme="dark">
  <style>
    .role-btn {
      background: $bg_button_primary;
      color: $text_primary;
      border_radius: 6;
      border_color: $border_highlight;
      border_width: 1;
      padding: 12 24;
    }
    .panel-section {
      background: $bg_secondary;
      border_radius: 6;
      border_color: $border_default;
      border_width: 1;
      padding: 8;
    }
    .equip-slot {
      background: $bg_primary;
      border_radius: 4;
      border_color: $border_default;
      border_width: 1;
      padding: 4;
    }
    .equip-label {
      color: $text_muted;
    }
    .portrait-panel {
      background: $bg_panel;
      border_radius: 6;
      border_color: $border_accent;
      border_width: 2;
      padding: 8;
    }
    .portrait-text {
      color: $text_title;
    }
    .grid-item {
      background: $bg_primary;
      border_radius: 4;
      border_color: $border_default;
      border_width: 1;
      padding: 4;
    }
    .grid-item-name {
      color: $text_primary;
    }
    .page-btn {
      background: $bg_button;
      color: $text_primary;
      border_radius: 4;
      padding: 6 16;
    }
    .page-info {
      color: $text_muted;
    }
  </style>
  <Control anchor="full">
    <!-- 居中角色按钮 -->
    <CenterContainer anchor="full">
      <Button name="OpenRoleBtn" text="Character" class="role-btn" font_size="20" on_pressed="show:RolePopup" mouse_default_cursor_shape="pointing_hand" />
    </CenterContainer>
  </Control>

  <!-- 角色属性弹窗 -->
  <PopupPanel name="RolePopup" popup_title="Character" width="80%" height="80%" close_on_overlay="true">
    <HBoxContainer margin="8" h_separation="12">
      <!-- 面板左：装备区 -->
      <VBoxContainer size_flags_horizontal="expand_fill" h_separation="0" v_separation="4">
        <Label text="Equipment" font_size="16" class="equip-label" />
        <HBoxContainer h_separation="8">
          <!-- 左侧装备列 -->
          <VBoxContainer h_separation="0" v_separation="6">
            <VBoxContainer class="equip-slot" custom_minimum_size="10%,10%">
              <Label text="Helmet" class="equip-label" font_size="10" align="center" />
              <Control size_flags_vertical="expand_fill" />
            </VBoxContainer>
            <VBoxContainer class="equip-slot" custom_minimum_size="10%,10%">
              <Label text="Weapon" class="equip-label" font_size="10" align="center" />
              <Control size_flags_vertical="expand_fill" />
            </VBoxContainer>
            <VBoxContainer class="equip-slot" custom_minimum_size="10%,10%">
              <Label text="Ring" class="equip-label" font_size="10" align="center" />
              <Control size_flags_vertical="expand_fill" />
            </VBoxContainer>
          </VBoxContainer>

          <!-- 中间角色立绘面板 -->
          <VBoxContainer class="portrait-panel" custom_minimum_size="25%,0" size_flags_horizontal="expand_fill" size_flags_vertical="expand_fill">
            <Label text="Portrait" class="portrait-text" font_size="14" align="center" />
            <Control size_flags_vertical="expand_fill" />
          </VBoxContainer>

          <!-- 右侧装备列 -->
          <VBoxContainer h_separation="0" v_separation="6">
            <VBoxContainer class="equip-slot" custom_minimum_size="10%,10%">
              <Label text="Armor" class="equip-label" font_size="10" align="center" />
              <Control size_flags_vertical="expand_fill" />
            </VBoxContainer>
            <VBoxContainer class="equip-slot" custom_minimum_size="10%,10%">
              <Label text="Shield" class="equip-label" font_size="10" align="center" />
              <Control size_flags_vertical="expand_fill" />
            </VBoxContainer>
            <VBoxContainer class="equip-slot" custom_minimum_size="10%,10%">
              <Label text="Boots" class="equip-label" font_size="10" align="center" />
              <Control size_flags_vertical="expand_fill" />
            </VBoxContainer>
          </VBoxContainer>
        </HBoxContainer>
      </VBoxContainer>

      <!-- 面板右：背包网格 -->
      <VBoxContainer size_flags_horizontal="expand_fill" h_separation="0" v_separation="4">
        <Label text="Backpack" font_size="16" class="equip-label" />
        <UIGrid name="BagGrid" count="25" columns="5" highlight_mode="1" highlight_color="#ffffff30" data="current_bag_data">
          <MarginContainer class="grid-item" custom_minimum_size="7%,7%">
            <VBoxContainer>
              <Label text="{{name}}" class="grid-item-name" align="center" font_size="10" />
            </VBoxContainer>
          </MarginContainer>
        </UIGrid>
        <!-- 分页控制 -->
        <HBoxContainer margin="4 0 0 0" h_separation="8">
          <Button name="PrevPageBtn" text="< Prev" class="page-btn" on_pressed="_on_prev_page" mouse_default_cursor_shape="pointing_hand" />
          <Control size_flags_horizontal="expand_fill" />
          <Label name="PageInfo" text="1 / 1" class="page-info" font_size="14" align="center" />
          <Control size_flags_horizontal="expand_fill" />
          <Button name="NextPageBtn" text="Next >" class="page-btn" on_pressed="_on_next_page" mouse_default_cursor_shape="pointing_hand" />
        </HBoxContainer>
      </VBoxContainer>
    </HBoxContainer>
  </PopupPanel>
</ui>
"""

func _ready() -> void:
	_init_bag_data()
	load_from_string(UI)
	_update_page_display()

func _init_bag_data() -> void:
	var names = [
		"Potion", "Elixir", "Scroll", "Ring", "Amulet",
		"Gem", "Herb", "Ore", "Wood", "Cloth",
		"Iron", "Crystal", "Feather", "Bone", "Leather",
		"Silk", "Amber", "Pearl", "Opal", "Ruby",
		"Sapphire", "Emerald", "Topaz", "Diamond", "Onyx",
		"Map", "Key", "Rope", "Torch", "Compass",
		"Bottle", "Coin", "Shard", "Core", "Essence",
		"Fragment", "Seed", "Spore", "Ink", "Wax",
		"Thread", "Nail", "Wire", "Lens", "Prism",
		"Fossil", "Shell", "Coral", "Amber", "Jade",
	]
	for i in range(names.size()):
		bag_data.append({ "name": names[i] })
	total_pages = max(1, ceili(float(bag_data.size()) / ITEMS_PER_PAGE))

var current_bag_data: Array:
	get:
		var start = current_page * ITEMS_PER_PAGE
		var end = min(start + ITEMS_PER_PAGE, bag_data.size())
		if start >= bag_data.size():
			return []
		return bag_data.slice(start, end - 1)

func _on_prev_page() -> void:
	if current_page > 0:
		current_page -= 1
		_refresh_bag()

func _on_next_page() -> void:
	if current_page < total_pages - 1:
		current_page += 1
		_refresh_bag()

func _refresh_bag() -> void:
	var grid: GdUIGrid = find_node("BagGrid")
	if grid:
		grid.update(current_bag_data)
	_update_page_display()

func _update_page_display() -> void:
	var label = find_node("PageInfo")
	if label:
		label.text = "%d / %d" % [current_page + 1, total_pages]
