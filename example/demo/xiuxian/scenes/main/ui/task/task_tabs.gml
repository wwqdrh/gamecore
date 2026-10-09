<ui>
  <!-- 页签切换容器：复用框架 TabContainer/Tab，页签切换为原生行为
	   每个 Tab 页通过 <Gml> 引用 task_list.gml（各自独立实例）。
	   按玩家接取状态过滤（数据字段 status_group，由 XiuTaskState.refresh_views 派生）：
	   · 进行中 = accepted（推进中）/ completed（已完成待领取）
	   · 已完成 = submitted（已提交领奖，留档展示）
	   未接取（available/locked）不进列表——任务列表随接取/完成动态增减，
	   数据联动链路：accept/advance/complete_task → refresh_views →
	   watch("views") → 各页签列表自动刷新 -->
  <script>
	// 各页签的状态组标识（与任务视图数据的 status_group 字段匹配）
	var grp_active = "active"
	var grp_done = "done"
  </script>
  <TabContainer name="TaskTabs" anchor="full" tabs_visible="true" current_tab="0">
	<Tab title="进行中">
	  <Gml src="task_list.gml" data-status_group="grp_active" />
	</Tab>
	<Tab title="已完成">
	  <Gml src="task_list.gml" data-status_group="grp_done" />
	</Tab>
  </TabContainer>
</ui>
