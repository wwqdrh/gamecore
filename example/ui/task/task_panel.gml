<ui theme="cartoon">
  <!-- 宗门任务面板骨架：整体布局 + 通过 <Gml> 标签直接引用子视图
	   （无需控制脚本手动 parse_file 挂载，GML 即所见即所得）
	   <Gml src="task_topbar.gml" />    顶部资源栏 + 关闭按钮
	   <Gml src="task_tabs.gml" />      TabContainer 页签（每页内嵌 task_list.gml）
	   <Gml src="task_activity.gml" />  活跃度进度 -->
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
		<Gml src="task_topbar.gml" />
		<Control custom_minimum_size="0,12" />
		<Panel class="title-banner" custom_minimum_size="0,48">
		  <Label text="宗门任务" class="title-text" font_size="24" align="center" valign="center" anchor="full" />
		</Panel>
		<Control custom_minimum_size="0,10" />
		<Gml src="task_tabs.gml" size_flags_vertical="expand_fill" />
		<Control custom_minimum_size="0,12" />
		<Gml src="task_activity.gml" />
	  </VBoxContainer>
	</MarginContainer>
  </Panel>
</ui>
