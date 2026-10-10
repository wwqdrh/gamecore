<ui script="dungeon_settle.gd">
  <!-- 秘境通关结算面板（被 main.gml 的 <Modal ui_id="DungeonSettleModal"> 包裹）：
	   数据 = XiuDungeonState.settle_view（DungeonManager.settle 写入），
	   watch 注册即回调当前值；关闭按钮发 s_close_requested → Modal 整体关闭 -->
  <style>
	.settle-panel {
	  background: #2e2436;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 12;
	}
	.settle-title { color: #f2d98c; font_size: 26; }
	.settle-line { color: #f2ead2; font_size: 16; }
	.settle-item { color: #cfe8c9; font_size: 15; }
	.settle-close { font_size: 16; }
  </style>
  <Panel name="SettleRoot" class="settle-panel" custom_minimum_size="460,0"
	     mouse_filter="stop">
	<VBoxContainer anchor="full" margin="24" separation="10">
	  <Label name="Title" class="settle-title" text="秘境通关结算" align="center" />
	  <Label name="LineFloors" class="settle-line" text="扫荡层数：-" align="center" />
	  <Label name="LineExp" class="settle-line" text="获得修为：+0" align="center" />
	  <Label name="LineStones" class="settle-line" text="获得灵石：+0" align="center" />
	  <Label class="settle-line" text="—— 战利品 ——" align="center" />
	  <VBoxContainer name="ItemList" separation="4" />
	  <Button name="CloseBtn" class="settle-close" text="收下奖励"
	          mouse_default_cursor_shape="pointing_hand" @pressed="_on_close_pressed" />
	</VBoxContainer>
  </Panel>
</ui>
