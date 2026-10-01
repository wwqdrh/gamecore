<ui theme="cartoon">
  <!-- 宗门任务面板骨架：只负责整体布局与插槽（Slot）占位
	   TopBarSlot    <- task_topbar.gml    （资源栏 + 关闭按钮）
	   TabSlot       <- task_tabs.gml      （TabContainer 页签，每页挂 task_list.gml）
	   ActivitySlot  <- task_activity.gml  （活跃度进度）
	   任务条目模板   <- task_item.gml      （注入各页的 UIVList） -->
  <style>
	.window-bg {
	  background: #2e4234;
	  border_color: #16241a;
	  border_width: 4;
	  border_radius: 18;
	}
	.title-banner {
	  background: #24352a;
	  border_color: #b8a06a;
	  border_width: 2;
	  border_radius: 10;
	}
	.title-text { color: #e8dcb8; }
  </style>
  <Panel name="WindowPanel" class="window-bg" anchor="full" margin="2%">
	<MarginContainer anchor="full" margin="18">
	  <VBoxContainer>
		<Control name="TopBarSlot" custom_minimum_size="0,52" />
		<Control custom_minimum_size="0,12" />
		<Panel class="title-banner" custom_minimum_size="0,48">
		  <Label text="宗门任务" class="title-text" font_size="24" align="center" valign="center" anchor="full" />
		</Panel>
		<Control custom_minimum_size="0,10" />
		<Control name="TabSlot" size_flags_vertical="expand_fill" />
		<Control custom_minimum_size="0,12" />
		<Control name="ActivitySlot" custom_minimum_size="0,72" />
	  </VBoxContainer>
	</MarginContainer>
  </Panel>
</ui>
