<ui script="mainhud_menus.gd">
  <!-- 右侧功能按钮列：修炼/功法/储物袋/坊市 4 个圆形按钮 + 红点角标。
	   按钮内容 = anchor full 的 VBox（图标 expand_fill 占上部 + 文字贴底），
	   不用点锚 + margin（bottom_wide 的 offset_top=0 会把文字推到按钮外）。
	   红点用 top_wide + align right（语义明确：横跨按钮宽、贴右上）。
	   @pressed 回调就近绑定本文件脚本（mainhud_menus.gd）：
	   点击后只上报 GdState（mainhud.menu），高亮互斥由状态下行驱动 -->
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
	<Panel name="MenuBag" class="menu-btn" custom_minimum_size="84,84"
	      mouse_default_cursor_shape="pointing_hand" @pressed="_on_menu_pressed">
	  <VBoxContainer anchor="full" margin="8 10 8 8" separation="2">
		<Label class="menu-btn-icon" text="👝" size_flags_vertical="expand_fill"
		       align="center" valign="center" />
		<Label class="menu-btn-text" text="储物袋" align="center" />
	  </VBoxContainer>
	  <Label class="red-dot" text="●" anchor="top_wide" margin="0 6 10 0" align="right" />
	</Panel>
	<Panel name="MenuMarket" class="menu-btn" custom_minimum_size="84,84"
	      mouse_default_cursor_shape="pointing_hand" @pressed="_on_menu_pressed">
	  <VBoxContainer anchor="full" margin="8 10 8 8" separation="2">
		<Label class="menu-btn-icon" text="🏮" size_flags_vertical="expand_fill"
		       align="center" valign="center" />
		<Label class="menu-btn-text" text="坊市" align="center" />
	  </VBoxContainer>
	  <Label class="red-dot" text="●" anchor="top_wide" margin="0 6 10 0" align="right" />
	</Panel>
  </VBoxContainer>
</ui>
