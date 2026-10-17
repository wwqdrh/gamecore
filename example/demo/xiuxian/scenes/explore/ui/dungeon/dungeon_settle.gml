<ui script="dungeon_settle.gd">
  <!-- 秘境通关结算面板（被 explore.gml 的 <Modal ui_id="DungeonSettleModal"> 包裹）：
	   数据 = XiuDungeonState.settle_view（DungeonManager.settle 写入），
	   watch 注册即回调当前值；关闭按钮发 s_close_requested → Modal 整体关闭。
	   布局：外层 CenterContainer 全屏居中（面板收缩到内容尺寸，不再铺满）；
	   战利品横向排列（每项 = 图标方框 + 下方数量）；按钮固定宽度居中 -->
  <style>
	.settle-panel {
	  background: #2e2436f2;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 12;
	}
	.settle-title { color: #f2d98c; font_size: 36; }
	.settle-line { color: #f2ead2; font_size: 16; }
	.item-box {
	  background: #3a3146;
	  border_color: #c9b98c;
	  border_width: 1;
	  border_radius: 8;
	}
	.item-icon { color: #f2ead2; font_size: 26; }
	.item-count { color: #f2ead2; font_size: 14; }
	.item-note { color: #b9a8d0; font_size: 11; }
	.settle-close { font_size: 17; }
  </style>
  <CenterContainer name="Center" anchor="full" mouse_filter="pass">
	<PanelContainer name="SettleRoot" class="settle-panel" custom_minimum_size="520,0"
	                mouse_filter="stop">
	  <VBoxContainer name="Content" separation="12">
		<Label name="Title" class="settle-title" text="秘境通关" align="center"
		       font_size="36" />
		<Label name="LineFloors" class="settle-line" text="扫荡层数：-" align="center"
		       font_size="16" />
		<Label name="LineExp" class="settle-line" text="获得修为：+0" align="center"
		       font_size="16" />
		<Label name="LineStones" class="settle-line" text="获得灵石：+0" align="center"
		       font_size="16" />
		<Label class="settle-line" text="—— 战利品 ——" align="center" font_size="16" />
		<HBoxContainer name="ItemList" separation="18" size_flags_horizontal="shrink_center" />
		<Button name="CloseBtn" class="settle-close" text="收下奖励"
		        custom_minimum_size="200,48" size_flags_horizontal="shrink_center"
		        mouse_default_cursor_shape="pointing_hand" @pressed="_on_close_pressed" />
	  </VBoxContainer>
	</PanelContainer>
  </CenterContainer>
</ui>
