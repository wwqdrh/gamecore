<ui script="mainhud_equipbar.gd">
  <!-- 底部装备栏（Hotbar 框架组件）：4 格快捷装备，数字键 1~4 / 点击选中，
       选中格金色高亮框。第一个槽 = 枪支。
       联动 = GdState 状态总线：选中上报写 mainhud.equip（命令上行），
       player.gd watch 同名键开关射击能力（状态下行），UI 与角色零耦合 -->
  <style>
	.equip-title { color: #f2ead2; font_size: 13; }
  </style>
  <HBoxContainer name="EquipBar" separation="10">
	<Label name="EquipTitle" class="equip-title" text="装备" valign="center" />
	<Hotbar name="EquipSlots" slot_count="4" slot_size="46,46" separation="6"
	        key_bind="true" selected_index="0"
	        slot_bg="#171f1aeb" slot_border="#6b6147ff"
	        highlight_color="#f2ca59ff" text_color="#f2ead2ff"
	        @s_selected="_on_equip_selected" />
  </HBoxContainer>
</ui>
