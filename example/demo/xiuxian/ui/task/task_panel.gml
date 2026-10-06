<ui script="task_panel.gd">
  <!-- 宗门任务面板骨架：整体布局 + 通过 <Gml> 标签直接引用子视图
	   （无需控制脚本手动 parse_file 挂载，GML 即所见即所得）
	   <Gml src="task_topbar.gml" />    顶部资源栏 + 关闭按钮
	   <Gml src="task_tabs.gml" />      TabContainer 页签（每页内嵌 task_list.gml）
	   <Gml src="task_activity.gml" />  活跃度进度
	   任务列表（task_tabs）包进左滑 Drawer：点击主区按钮从左侧展开，
	   slide_width="50%" 按视口宽度换算（width_ratio），随窗口尺寸自适应 -->
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
	.open-drawer-btn {
	  background: #2f5a3c;
	  color: #e8dcb8;
	  border_color: #b8a06a;
	  border_width: 2;
	  border_radius: 12;
	}
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
		<!-- 打开任务抽屉：内部动作 show: 自动连到 Drawer 的 open()（无 show_popup 方法时回退） -->
		<Button name="OpenTasksBtn" text="📜 展开任务列表" class="open-drawer-btn"
				custom_minimum_size="0,56" font_size="20"
				@pressed="show:TaskDrawer" mouse_default_cursor_shape="pointing_hand" />
		<Control custom_minimum_size="0,12" />
		<Gml src="task_activity.gml" />
	  </VBoxContainer>
	</MarginContainer>
	<!-- 任务列表抽屉：必须挂 Panel 直下（容器父级会接管锚点，破坏全屏遮罩），
		 内容区引用 task_tabs.gml，构建期嫁接进 Drawer 的 ContentContainer -->
	<Drawer name="TaskDrawer" direction="left" slide_width="50%" drawer_title="任务"
			close_on_overlay="true">
	  <Gml src="task_tabs.gml" size_flags_vertical="expand_fill" />
	</Drawer>
  </Panel>
</ui>
