<ui script="task_list.gd">
  <!-- 任务列表容器（可复用）：滚动列表
	   UIVList 的 slot 模板通过 <Gml> 直接引用 task_item.gml（构建期注入），
	   UIVList 用 data="bean:xiuxian_task:views" 声明式绑定状态层任务展示视图
	   （example/demo/xiuxian/state/task/task_state.gd 的 XiuTaskState）——
	   初始填充 + watch 响应式刷新由 task_list.gd 完成（组合根直开 tscn 场景
	   不走 GdGmlScene auto_bind_data 链路，绑定由控制器手动接线）；
	   视图数据 = 全量任务总表（含未解锁，未解锁显示封印态）+ 运行进度派生，
	   进度推进（advance_task/apply_task_rewards）后列表自动刷新。
	   分类过滤：filter_key="category"（数据字段）+ filter_value_var="category"
	   （过滤值取本文件 <script> 的 category 变量）——多页签复用本文件时，
	   引用方用 <Gml src="task_list.gml" data-category="分类变量" /> 注入各自分类，
	   本实例列表即成为"只显示该分类任务"的视图；category 为空（独立打开）显示全部 -->
  <script>
	// 任务分类标识：引用方 <Gml data-category="分类变量"> 注入覆盖，
	// 独立打开时为空 = 不分类（显示全部任务）
	var category = ""
  </script>
  <ScrollContainer name="TaskScroll" size_flags_vertical="expand_fill">
	<UIVList name="TaskList" size_flags_horizontal="expand_fill"
			 data="bean:xiuxian_task:views" filter_key="category" filter_value_var="category">
	  <Gml src="task_item.gml" />
	</UIVList>
  </ScrollContainer>
</ui>
