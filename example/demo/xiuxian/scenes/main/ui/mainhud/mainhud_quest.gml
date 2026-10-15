<ui>
  <!-- 左中任务卷轴横幅：卷轴轴头 + 羊皮纸任务文本（追踪目标任务进度）。
	   文本由 mainhud.gd watch 状态层 XiuTaskState views 驱动
	   （首个进行中任务：名称 + 步数进度；无进行中时显示可接取指引）。
	   宽度自适应：中间羊皮纸用 PanelContainer（min 尺寸随内容撑开），
	   文本用 FitLabel（fit_width 按文本像素宽自适应 custom_minimum_size.x）
	   ——任务名/指引文案再长也不会溢出面板 -->
  <style>
	.quest-scroll {
	  background: #e8d9a8;
	  border_color: #8a6a1e;
	  border_width: 2;
	  border_radius: 8;
	  padding: 8 12 8 12;
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
	<PanelContainer class="quest-scroll" custom_minimum_size="0,48">
	  <HBoxContainer separation="8">
		<Label class="quest-icon" text="📜" valign="center" />
		<FitLabel name="QuestText" class="quest-text" text="暂无进行中的任务" valign="center" fit_width="true" />
	  </HBoxContainer>
	</PanelContainer>
	<Panel class="quest-roller" custom_minimum_size="12,48" />
  </HBoxContainer>
</ui>
