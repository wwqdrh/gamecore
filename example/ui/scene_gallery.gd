# 图鉴界面 GML 控制器 - 继承 GdGmlScene，处理图鉴弹窗中的 TabContainer 数据渲染
# PopupPanel 的显示/隐藏由 GML 内部信号绑定自动处理
# TabContainer 使用原生 Godot TabContainer，Tab 子标签的 title 属性作为 tab 标题
# 每个 Tab 页包含顶部描述文字和下方 UIGrid 网格列表
extends GdGmlScene

var UI = """
<ui>
  <style>
    .gallery-btn {
      background: #1a1a3e;
      color: white;
      border_radius: 6;
      border_color: #3a3a6e;
      border_width: 1;
      padding: 12;
    }
    .tab-desc {
      color: #aaaacc;
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
  <VBoxContainer anchor="full">
    <!-- 居中按钮 -->
    <CenterContainer>
      <Button name="GalleryBtn" text="Open Gallery" class="gallery-btn" custom_minimum_size="240,48" on_pressed="show:GalleryPopup" mouse_default_cursor_shape="pointing_hand" />
    </CenterContainer>
  </VBoxContainer>

  <!-- 图鉴弹窗 -->
  <PopupPanel name="GalleryPopup" popup_title="Gallery" popup_width="560" close_on_overlay="true">
    <TabContainer name="GalleryTabs" custom_minimum_size="500,400">
      <Tab title="Weapons">
        <Label text="Weapon collection - choose your weapon" class="tab-desc" />
        <UIGrid name="WeaponGrid" count="6" columns="3" highlight_mode="1" highlight_color="#ffffff30" data="weapon_data">
          <MarginContainer class="grid-item" custom_minimum_size="96,96">
            <VBoxContainer>
              <Label text="{{name}}" class="grid-item-name" align="center" font_size="12" />
              <Label text="{{desc}}" class="grid-item-desc" align="center" font_size="10" />
            </VBoxContainer>
          </MarginContainer>
        </UIGrid>
      </Tab>
      <Tab title="Armor">
        <Label text="Armor collection - protect yourself" class="tab-desc" />
        <UIGrid name="ArmorGrid" count="6" columns="3" highlight_mode="1" highlight_color="#ffffff30" data="armor_data">
          <MarginContainer class="grid-item" custom_minimum_size="96,96">
            <VBoxContainer>
              <Label text="{{name}}" class="grid-item-name" align="center" font_size="12" />
              <Label text="{{desc}}" class="grid-item-desc" align="center" font_size="10" />
            </VBoxContainer>
          </MarginContainer>
        </UIGrid>
      </Tab>
      <Tab title="Items">
        <Label text="Item collection - useful items" class="tab-desc" />
        <UIGrid name="ItemGrid" count="6" columns="3" highlight_mode="1" highlight_color="#ffffff30" data="item_data">
          <MarginContainer class="grid-item" custom_minimum_size="96,96">
            <VBoxContainer>
              <Label text="{{name}}" class="grid-item-name" align="center" font_size="12" />
              <Label text="{{desc}}" class="grid-item-desc" align="center" font_size="10" />
            </VBoxContainer>
          </MarginContainer>
        </UIGrid>
      </Tab>
    </TabContainer>
  </PopupPanel>
</ui>
"""

var weapon_data = [
	{ "name": "Sword", "desc": "ATK +10" },
	{ "name": "Bow", "desc": "ATK +8" },
	{ "name": "Staff", "desc": "MATK +12" },
	{ "name": "Dagger", "desc": "ATK +6" },
	{ "name": "Axe", "desc": "ATK +15" },
	{ "name": "Spear", "desc": "ATK +11" },
]

var armor_data = [
	{ "name": "Helmet", "desc": "DEF +5" },
	{ "name": "Chestplate", "desc": "DEF +12" },
	{ "name": "Shield", "desc": "DEF +8" },
	{ "name": "Boots", "desc": "SPD +3" },
	{ "name": "Gloves", "desc": "ATK +2" },
	{ "name": "Cloak", "desc": "EVA +5" },
]

var item_data = [
	{ "name": "Potion", "desc": "HP +50" },
	{ "name": "Elixir", "desc": "MP +30" },
	{ "name": "Scroll", "desc": "MATK +5" },
	{ "name": "Ring", "desc": "ALL +2" },
	{ "name": "Amulet", "desc": "LUK +10" },
	{ "name": "Gem", "desc": "Sell: 100G" },
]

func _ready() -> void:
	load_from_string(UI)
