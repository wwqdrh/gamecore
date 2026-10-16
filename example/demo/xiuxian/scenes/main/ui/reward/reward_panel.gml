<ui script="reward_panel.gd">
  <!-- 任务领奖弹窗内容面板：Modal 遮罩（黑色透明 rgba(0,0,0,0.55)）+
	   居中窗口三段布局——上方大标题 / 中间奖励格子列表 / 下方确认按钮。
	   数据链路：XiuTaskState.complete_task 自动发奖后写 last_reward
	   （last_reward_task=任务名），本面板 watch 自动填充并经
	   GdUIManager.find_ui("TaskRewardModal").open() 弹出；
	   「确认」按钮 clear_last_reward + emit s_close_requested →
	   Modal 整体关闭（close_on_overlay=false：只认确认键，防误触丢展示）。
	   奖励格子悬停 tips 由 reward_item.gd 接线到本控制器 gml_tips_host() -->
  <style>
	.window-bg {
	  background: #17120ae6;
	  border_color: #c9a24a;
	  border_width: 2;
	  border_radius: 14;
	}
	.reward-title {
	  color: #f2ca59;
	  font_size: 30;
	}
	.reward-sub {
	  color: #b9ad8e;
	  font_size: 15;
	}
	.confirm-btn {
	  background: #8a6a2a;
	  border_color: #f2ca59;
	  border_width: 1;
	  border_radius: 8;
	}
	.confirm-btn > Label {
	  color: #fff3d0;
	  font_size: 18;
	}
  </style>
  <Panel name="RewardPanel" class="window-bg" anchor="full" margin="24%"
		 mouse_filter="stop">
	<VBoxContainer name="MainColumn" anchor="full" margin="20" separation="6">
	  <!-- 上方：大标题 + 任务名副标题 -->
	  <Label name="TitleLabel" class="reward-title" text="任务完成"
	         align="center" size_flags_horizontal="expand_fill" />
	  <Label name="SubtitleLabel" class="reward-sub" text=""
	         align="center" size_flags_horizontal="expand_fill" />
	  <Control custom_minimum_size="0,10" />
	  <!-- 中间：奖励格子列表（方框 + 图标 + 数量角标 + 悬停 tips） -->
	  <UIHList name="RewardList" size_flags_horizontal="shrink_center"
	           separation="10">
	    <Gml src="reward_item.gml" />
	  </UIHList>
	  <Control custom_minimum_size="0,14" size_flags_vertical="expand_fill" />
	  <!-- 下方：确认按钮（点击关闭弹窗） -->
	  <CenterContainer>
	    <Button name="ConfirmBtn" class="confirm-btn" text="确  认"
	            custom_minimum_size="140,44" @pressed="_on_confirm_pressed" />
	  </CenterContainer>
	</VBoxContainer>
  </Panel>
</ui>
