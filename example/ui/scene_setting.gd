# 设置界面 GML 控制器 - 继承 GdGmlScene，使用 NavMenu 组件实现多级级联菜单
# 画面中间有一个按钮，点击后左侧弹出一级菜单（3项），点击菜单项展开子级菜单
extends GdGmlScene

var UI = """
<ui>
  <style>
    .setting-btn {
      background: #2a2a4e;
      color: #ccccee;
      border_radius: 6;
      padding: 12 24;
    }
  </style>
  <Control anchor="full">
    <!-- 居中设置按钮 -->
	<CenterContainer anchor="full">
	  <Button name="OpenMenuBtn" text="Settings" class="setting-btn" font_size="20" on_pressed="toggle:NavMenu" mouse_default_cursor_shape="pointing_hand" />
    </CenterContainer>
  </Control>

  <!-- 导航菜单：左侧弹出，NavItem 递归嵌套支持多级菜单 -->
  <NavMenu name="NavMenu" direction="left" menu_width="160" sub_menu_width="200" close_on_overlay="true" animation_duration="0.2">
	<NavItem text="Audio">
	  <NavItem text="Volume">
		<NavItem text="Master" />
		<NavItem text="Music" />
      </NavItem>
	  <NavItem text="Output" />
    </NavItem>
	<NavItem text="Display">
	  <NavItem text="Resolution" />
	  <NavItem text="Fullscreen" />
	  <NavItem text="VSync" />
    </NavItem>
	<NavItem text="Controls">
	  <NavItem text="Key Bindings" />
	  <NavItem text="Mouse Sensitivity" />
	  <NavItem text="Gamepad" />
	  <NavItem text="Accessibility" />
    </NavItem>
  </NavMenu>
</ui>
"""

func _ready() -> void:
	load_from_string(UI)
