<ui>
  <!-- 底部活跃度：数值 + 进度条 + 宝箱里程碑（30/60/90）
	   ProgressBar 的 class 中 background=填充色、track=轨道色 -->
  <style>
	.act-label { color: #e8dcb8; }
	.act-value {
	  background: #1c2a20;
	  color: #ffd968;
	  border_color: #b8a06a;
	  border_width: 2;
	  border_radius: 14;
	}
	.act-bar {
	  background: #d4a942;
	  track: #2a3b2f;
	  border_radius: 9;
	}
	.chest-num { color: #cdbf9a; }
  </style>
  <HBoxContainer name="ActivityRoot" anchor="full">
	<Label text="今日活跃度" class="act-label" font_size="17" valign="center" size_flags_vertical="shrink_center" />
	<Control custom_minimum_size="10,0" />
	<Panel class="act-value" custom_minimum_size="52,36" size_flags_vertical="shrink_center">
	  <Label name="ActValue" text="65" font_size="17" align="center" valign="center" anchor="full" />
	</Panel>
	<Control custom_minimum_size="14,0" />
	<VBoxContainer size_flags_horizontal="expand_fill" size_flags_vertical="shrink_center">
	  <ProgressBar name="ActivityBar" value="65" max_value="100" class="act-bar" custom_minimum_size="0,18" />
	  <Control custom_minimum_size="0,6" />
	  <HBoxContainer>
		<Control size_flags_horizontal="expand_fill" />
		<VBoxContainer name="Chest1">
		  <Label text="🎁" font_size="24" align="center" />
		  <Label text="30" class="chest-num" font_size="13" align="center" />
		</VBoxContainer>
		<Control size_flags_horizontal="expand_fill" />
		<VBoxContainer name="Chest2">
		  <Label text="🔒" font_size="24" align="center" />
		  <Label text="60" class="chest-num" font_size="13" align="center" />
		</VBoxContainer>
		<Control size_flags_horizontal="expand_fill" />
		<VBoxContainer name="Chest3">
		  <Label text="🔒" font_size="24" align="center" />
		  <Label text="90" class="chest-num" font_size="13" align="center" />
		</VBoxContainer>
		<Control size_flags_horizontal="expand_fill" />
	  </HBoxContainer>
	</VBoxContainer>
  </HBoxContainer>
</ui>
