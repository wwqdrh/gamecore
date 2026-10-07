<ui script="mainhud_menus.gd">
  <!-- 右侧功能按钮列：修炼/功法/储物袋/任务/坊市 5 个圆形按钮 + 红点角标。
	   按钮内容 = anchor full 的 VBox（图标 expand_fill 占上部 + 文字贴底），
	   不用点锚 + margin（bottom_wide 的 offset_top=0 会把文字推到按钮外）。
	   红点用 top_wide + align right（语义明确：横跨按钮宽、贴右上）。
	   @pressed 回调两种方式：
	   · 方法绑定（_on_menu_pressed）就近绑定本文件脚本：点击后只上报
	     GdState（mainhud.menu），高亮互斥由状态下行驱动
	   · 内部动作（show:TaskDrawer）经统一 UI 管理层跨组件调用：任务抽屉
	     在 ui/task/ 目录、与本文件零引用，按下时按 ui_id 查注册表触发 open() -->
  <style>
	.menu-btn {
	  background: #3a5a46;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 42;
	}
	.menu-btn-icon { font_size: 32; }
	.menu-btn-text { color: #f2ead2; font_size: 14; }
	.red-dot {
	  color: #e05048;
	  font_size: 14;
	}
  </style>
  <VBoxContainer name="MenuColumn" separation="14">
	<Panel name="MenuCultivate" class="menu-btn" custom_minimum_size="84,84"
	      mouse_default_cursor_shape="pointing_hand" @pressed="_on_menu_pressed">
	  <VBoxContainer anchor="full" margin="8 10 8 8" separation="2">
		<Label class="menu-btn-icon" text="🧘" size_flags_vertical="expand_fill"
		       align="center" valign="center" />
		<Label class="menu-btn-text" text="修炼" align="center" />
	  </VBoxContainer>
	  <Label class="red-dot" text="●" anchor="top_wide" margin="0 6 10 0" align="right" />
	</Panel>
	<Panel name="MenuGongfa" class="menu-btn" custom_minimum_size="84,84"
	      mouse_default_cursor_shape="pointing_hand" @pressed="_on_menu_pressed">
	  <VBoxContainer anchor="full" margin="8 10 8 8" separation="2">
		<Label class="menu-btn-icon" text="📖" size_flags_vertical="expand_fill"
		       align="center" valign="center" />
		<Label class="menu-btn-text" text="功法" align="center" />
	  </VBoxContainer>
	  <Label class="red-dot" text="●" anchor="top_wide" margin="0 6 10 0" align="right" />
	</Panel>
	<!-- 储物袋（ui/bag/）：内部动作经统一 UI 管理层跨组件触发，
		 按下时按 ui_id=BagModal 查注册表 → Modal.open()，零文件引用 -->
	<Panel name="MenuBag" class="menu-btn" custom_minimum_size="84,84"
	      mouse_default_cursor_shape="pointing_hand" @pressed="show:BagModal">
	  <VBoxContainer anchor="full" margin="8 10 8 8" separation="2">
		<Label class="menu-btn-icon" text="👝" size_flags_vertical="expand_fill"
		       align="center" valign="center" />
		<Label class="menu-btn-text" text="储物袋" align="center" />
	  </VBoxContainer>
	  <Label class="red-dot" text="●" anchor="top_wide" margin="0 6 10 0" align="right" />
	</Panel>
	<!-- 任务抽屉（ui/task/ 目录）：内部动作经统一 UI 管理层跨组件触发，
		 按下时按 ui_id=TaskDrawer 查注册表 → Drawer.open()，零文件引用 -->
	<Panel name="MenuTask" class="menu-btn" custom_minimum_size="84,84"
	      mouse_default_cursor_shape="pointing_hand" @pressed="show:TaskDrawer">
	  <VBoxContainer anchor="full" margin="8 10 8 8" separation="2">
		<Label class="menu-btn-icon" text="📜" size_flags_vertical="expand_fill"
		       align="center" valign="center" />
		<Label class="menu-btn-text" text="任务" align="center" />
	  </VBoxContainer>
	</Panel>
	<!-- 坊市（ui/store/）：同储物袋，跨组件触发 StoreModal.open() -->
	<Panel name="MenuMarket" class="menu-btn" custom_minimum_size="84,84"
	      mouse_default_cursor_shape="pointing_hand" @pressed="show:StoreModal">
	  <VBoxContainer anchor="full" margin="8 10 8 8" separation="2">
		<Label class="menu-btn-icon" text="🏮" size_flags_vertical="expand_fill"
		       align="center" valign="center" />
		<Label class="menu-btn-text" text="坊市" align="center" />
	  </VBoxContainer>
	  <Label class="red-dot" text="●" anchor="top_wide" margin="0 6 10 0" align="right" />
	</Panel>
  </VBoxContainer>
</ui>
