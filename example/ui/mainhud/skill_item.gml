<ui script="skill_item.gd">
  <!-- 技能格条目模板：作为 mainhud_skillbar.gml 中 UIHList 的 slot 模板（构建期注入）。
	   数据契约见 skill_item.gd 的 @export 列表：
	   icon 技能图标 / key 键位角标（"1"~"6"） / cd 冷却文本（"8s"，空串隐藏遮罩）
	   {{key}} 模板绑定负责静态文本渲染，契约注入负责冷却态联动 -->
  <style>
	.slot-bg {
	  background: #243c2e;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 10;
	}
	.slot-icon { font_size: 26; }
	.cd-mask {
	  background: #0a1410c0;
	  border_radius: 10;
	}
	.cd-text { color: #f2ead2; font_size: 15; }
	.key-badge {
	  color: #f2ead2;
	  font_size: 11;
	  background: #4a3212;
	  border_radius: 3;
	}
  </style>
  <!-- 根节点双向 expand_fill：格子拉伸填满 UIHList 条目区 -->
  <Panel name="SkillSlot" class="slot-bg" custom_minimum_size="56,56"
         size_flags_horizontal="expand_fill" size_flags_vertical="expand_fill">
	<!-- 技能图标 -->
	<CenterContainer name="IconBox" anchor="full" margin="5">
	  <Label name="SlotIcon" class="slot-icon" text="{{icon}}" />
	</CenterContainer>
	<!-- 冷却遮罩 + 倒计时（cd 契约非空时显示，置于图标之上） -->
	<Panel name="CdMask" class="cd-mask" anchor="full" visible="false" />
	<Label name="CdText" class="cd-text" text="{{cd}}" anchor="full"
	       align="center" valign="center" visible="false" />
	<!-- 键位角标：底部居中（bottom_wide 需负 top margin 收进格子内） -->
	<Label name="KeyBadge" class="key-badge" text="{{key}}" anchor="bottom_wide"
	       margin="14 -24 14 2" align="center" />
  </Panel>
</ui>
