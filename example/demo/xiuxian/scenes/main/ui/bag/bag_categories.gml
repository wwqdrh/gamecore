<ui script="bag_categories.gd">
  <!-- 左侧分类列表：5 个分类按钮（全部/法器/丹药/材料/符篆）+ 底部「修仙」章。
	   @pressed 声明在 Panel 等非按钮控件上由框架 gui_input 点击回退驱动
	   （左键按下触发，回调补绑发出面板），就近绑定本文件脚本（bag_categories.gd）：
	   点击后互斥高亮 + 联动过滤右侧背包网格（set_meta __filter_value + update） -->
  <style>
	.cat-panel {
	  background: #e9e2c8;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 10;
	}
	.cat-btn {
	  background: #f2ecd4;
	  border_color: #c9b98c;
	  border_width: 1;
	  border_radius: 8;
	}
	.cat-text { color: #4a3a22; font_size: 17; }
	.seal-text {
	  color: #b03a2e;
	  font_size: 20;
	}
  </style>
  <Panel name="CategoryPanel" class="cat-panel" anchor="full">
	<VBoxContainer name="CatColumn" anchor="full" margin="10 12 10 12" separation="10">
	  <Panel name="CatAll" class="cat-btn" custom_minimum_size="0,50" mouse_default_cursor_shape="pointing_hand"
	       @pressed="_on_category">
		<HBoxContainer anchor="full" margin="12 0 8 0" separation="10">
		  <Label text="🔳" valign="center" font_size="18" />
		  <Label class="cat-text" text="全部" valign="center" />
		</HBoxContainer>
	  </Panel>
	  <Panel name="CatTool" class="cat-btn" custom_minimum_size="0,50" mouse_default_cursor_shape="pointing_hand"
	       @pressed="_on_category">
		<HBoxContainer anchor="full" margin="12 0 8 0" separation="10">
		  <Label text="🗡️" valign="center" font_size="18" />
		  <Label class="cat-text" text="法器" valign="center" />
		</HBoxContainer>
	  </Panel>
	  <Panel name="CatPill" class="cat-btn" custom_minimum_size="0,50" mouse_default_cursor_shape="pointing_hand"
	       @pressed="_on_category">
		<HBoxContainer anchor="full" margin="12 0 8 0" separation="10">
		  <Label text="⚗️" valign="center" font_size="18" />
		  <Label class="cat-text" text="丹药" valign="center" />
		</HBoxContainer>
	  </Panel>
	  <Panel name="CatMaterial" class="cat-btn" custom_minimum_size="0,50" mouse_default_cursor_shape="pointing_hand"
	       @pressed="_on_category">
		<HBoxContainer anchor="full" margin="12 0 8 0" separation="10">
		  <Label text="🌿" valign="center" font_size="18" />
		  <Label class="cat-text" text="材料" valign="center" />
		</HBoxContainer>
	  </Panel>
	  <Panel name="CatTalisman" class="cat-btn" custom_minimum_size="0,50" mouse_default_cursor_shape="pointing_hand"
	       @pressed="_on_category">
		<HBoxContainer anchor="full" margin="12 0 8 0" separation="10">
		  <Label text="📜" valign="center" font_size="18" />
		  <Label class="cat-text" text="符篆" valign="center" />
		</HBoxContainer>
	  </Panel>
	  <Control size_flags_vertical="expand_fill" />
	  <!-- 底部竖排装饰章 -->
	  <Label class="seal-text" text="修
仙" align="center" custom_minimum_size="0,64" />
	</VBoxContainer>
  </Panel>
</ui>
