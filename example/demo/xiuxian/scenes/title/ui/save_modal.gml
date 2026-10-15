<ui script="save_modal.gd">
<!-- 存档选择弹窗内容（被 title_menu.gml 的 <Modal ui_id="SaveModal"> 包裹）：
	 三个存档槽位：空档显示「暂无存档 + 新游戏」，有档显示 境界/天数/时长 +
	 「继续游戏 + 删除存档」（show_if="slotN_has_save" 响应式条件显隐，
	 GdUIReactive.set_var 驱动；删除走两步确认，见 save_modal.gd）；
	 返回按钮发 s_close_requested → Modal 整体关闭 -->
  <style>
	.save-panel {
	  background: #1c2733;
	  border_color: #8a6a1e;
	  border_width: 2;
	  border_radius: 12;
	}
	.save-title { color: #e8d9a8; font_size: 26; }
	.save-slot {
	  background: #24303e;
	  border_color: #3a4a5a;
	  border_width: 1;
	  border_radius: 8;
	}
	.save-slot-name { color: #c9b98c; font_size: 18; }
	.save-line { color: #aebbc9; font_size: 14; }
	.save-btn {
	  background: #2e4a3a;
	  color: #f2ead2;
	  border_color: #8a6a1e;
	  border_width: 1;
	  border_radius: 8;
	  font_size: 16;
	}
	.save-btn-back {
	  background: #3a3040;
	}
	.save-btn-del {
	  background: #4a2a2a;
	  color: #f0c8b8;
	  border_color: #8a3a2e;
	  font_size: 14;
	}
  </style>
  <Panel name="SaveRoot" class="save-panel" custom_minimum_size="560,0" mouse_filter="stop">
	<VBoxContainer anchor="full" margin="24" separation="12">
	  <Label name="Title" class="save-title" text="选 择 存 档" align="center" />
	  <Panel name="Slot1" class="save-slot" custom_minimum_size="0,84">
		<HBoxContainer anchor="full" margin="14" separation="12">
		  <Label name="Slot1Name" class="save-slot-name" text="存档一" />
		  <VBoxContainer size_flags_horizontal="expand_fill" separation="2">
			<Label name="Slot1Line1" class="save-line" text="暂无存档" />
			<Label name="Slot1Line2" class="save-line" text="" />
		  </VBoxContainer>
		  <Button name="Slot1Btn" class="save-btn" text="新游戏"
				  custom_minimum_size="120,44" mouse_default_cursor_shape="pointing_hand"
				  @pressed="_on_slot1_pressed" />
		  <Button name="Slot1DelBtn" class="save-btn-del" text="删除存档"
				  custom_minimum_size="88,44" mouse_default_cursor_shape="pointing_hand"
				  show_if="slot1_has_save" @pressed="_on_slot1_del_pressed" />
		</HBoxContainer>
	  </Panel>
	  <Panel name="Slot2" class="save-slot" custom_minimum_size="0,84">
		<HBoxContainer anchor="full" margin="14" separation="12">
		  <Label name="Slot2Name" class="save-slot-name" text="存档二" />
		  <VBoxContainer size_flags_horizontal="expand_fill" separation="2">
			<Label name="Slot2Line1" class="save-line" text="暂无存档" />
			<Label name="Slot2Line2" class="save-line" text="" />
		  </VBoxContainer>
		  <Button name="Slot2Btn" class="save-btn" text="新游戏"
				  custom_minimum_size="120,44" mouse_default_cursor_shape="pointing_hand"
				  @pressed="_on_slot2_pressed" />
		  <Button name="Slot2DelBtn" class="save-btn-del" text="删除存档"
				  custom_minimum_size="88,44" mouse_default_cursor_shape="pointing_hand"
				  show_if="slot2_has_save" @pressed="_on_slot2_del_pressed" />
		</HBoxContainer>
	  </Panel>
	  <Panel name="Slot3" class="save-slot" custom_minimum_size="0,84">
		<HBoxContainer anchor="full" margin="14" separation="12">
		  <Label name="Slot3Name" class="save-slot-name" text="存档三" />
		  <VBoxContainer size_flags_horizontal="expand_fill" separation="2">
			<Label name="Slot3Line1" class="save-line" text="暂无存档" />
			<Label name="Slot3Line2" class="save-line" text="" />
		  </VBoxContainer>
		  <Button name="Slot3Btn" class="save-btn" text="新游戏"
				  custom_minimum_size="120,44" mouse_default_cursor_shape="pointing_hand"
				  @pressed="_on_slot3_pressed" />
		  <Button name="Slot3DelBtn" class="save-btn-del" text="删除存档"
				  custom_minimum_size="88,44" mouse_default_cursor_shape="pointing_hand"
				  show_if="slot3_has_save" @pressed="_on_slot3_del_pressed" />
		</HBoxContainer>
	  </Panel>
	  <Button name="BackBtn" class="save-btn save-btn-back" text="返 回"
		  custom_minimum_size="120,40" size_flags_horizontal="shrink_center"
		  mouse_default_cursor_shape="pointing_hand" @pressed="_on_back_pressed" />
	</VBoxContainer>
  </Panel>
</ui>
