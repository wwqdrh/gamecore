<ui>
  <!-- 左中任务卷轴横幅：卷轴轴头 + 羊皮纸任务文本（追踪目标任务进度）。
	   无脚本：静态展示区块（进度接任务数据后可改 {{}} 模板） -->
  <style>
	.quest-scroll {
	  background: #e8d9a8;
	  border_color: #8a6a1e;
	  border_width: 2;
	  border_radius: 8;
	}
	.quest-roller {
	  background: #6b4a1e;
	  border_color: #4a3212;
	  border_width: 1;
	  border_radius: 5;
	}
	.quest-text { color: #4a3a22; font_size: 16; }
	.quest-icon { font_size: 18; }
  </style>
  <HBoxContainer name="QuestBanner" separation="4">
	<Panel class="quest-roller" custom_minimum_size="12,48" />
	<Panel class="quest-scroll" custom_minimum_size="226,48">
	  <HBoxContainer anchor="full" margin="12 8 12 8" separation="8">
		<Label class="quest-icon" text="📜" valign="center" />
		<Label class="quest-text" text="击败山中妖狼 3/5" valign="center" />
	  </HBoxContainer>
	</Panel>
	<Panel class="quest-roller" custom_minimum_size="12,48" />
  </HBoxContainer>
</ui>
