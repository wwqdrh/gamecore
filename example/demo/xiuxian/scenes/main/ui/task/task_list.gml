<ui script="task_list.gd">
  <!-- 任务列表容器（可复用）：滚动列表
	   UIVList 的 slot 模板通过 <Gml> 直接引用 task_item.gml（构建期注入），
	   data="bean:xiuxian_task:views" 声明式绑定状态层任务展示视图
	   （example/demo/xiuxian/state/task/task_state.gd 的 XiuTaskState）——
	   只显示玩家接取过的任务：filter_key="status_group" +
	   filter_value_var="status_group" 按状态组过滤，引用方用
	   <Gml src="task_list.gml" data-status_group="变量" /> 具名注入：
	   · "active" = 进行中（accepted/completed）
	   · "done"  = 已完成（submitted 留档）
	   过滤值为空（独立打开）显示全部；列表为空时显示 EmptyHint 引导文案
	   初始填充 + watch 响应式刷新由 task_list.gd 完成（组合根直开 tscn 场景
	   不走 GdGmlScene auto_bind_data 链路，绑定由控制器手动接线） -->
  <style>
	.empty-hint { color: #8a7a58; font_size: 15; }
  </style>
  <script>
	// 状态组过滤标识：引用方 <Gml data-status_group="变量"> 注入覆盖，
	// 独立打开时为空 = 不过滤（显示全部）
	var status_group = ""
  </script>
  <ScrollContainer name="TaskScroll" size_flags_vertical="expand_fill">
	<VBoxContainer name="ListBox" size_flags_horizontal="expand_fill" size_flags_vertical="expand_fill">
	  <Label name="EmptyHint" class="empty-hint" text="暂无任务，去 NPC 处接取吧"
			 align="center" size_flags_vertical="shrink_center" visible="false" />
	  <UIVList name="TaskList" size_flags_horizontal="expand_fill" size_flags_vertical="expand_fill"
			   data="bean:xiuxian_task:views" filter_key="status_group" filter_value_var="status_group">
		<Gml src="task_item.gml" />
	  </UIVList>
	</VBoxContainer>
  </ScrollContainer>
</ui>
