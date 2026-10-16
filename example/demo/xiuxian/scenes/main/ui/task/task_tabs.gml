<ui>
  <!-- 页签切换容器：复用框架 TabContainer/Tab，页签切换为原生行为
	   每个 Tab 页通过 <Gml> 引用 task_list.gml（各自独立实例）。
	   按玩家接取状态过滤（数据字段 status_group，由 XiuTaskState.refresh_views 派生）：
	   · 可接取 = available（📍 desc 前缀注明找 giver 接取）
	   · 进行中 = accepted（推进中）/ completed（已完成待领取）
	   · 已完成 = submitted（已提交领奖，留档展示）
	   locked（前置未满足）不进任何页签——前置任务没完成，后续任务不显示，
	   任务随前置链完成逐个点亮进目录。默认页签"可接取"= 游戏的任务目录：
	   玩家不用盲找 NPC，打开面板就知道下一步找谁对话。数据联动链路：
	   unlock/accept/advance/complete_task → refresh_views → watch("views") →
	   各页签列表自动刷新（可接取 → 进行中 → 已完成） -->
  <script>
	// 各页签的状态组标识（与任务视图数据的 status_group 字段匹配）
	var grp_avail = "avail"
	var grp_active = "active"
	var grp_done = "done"
  </script>
  <TabContainer name="TaskTabs" anchor="full" tabs_visible="true" current_tab="0">
	<Tab title="可接取">
	  <Gml src="task_list.gml" data-status_group="grp_avail" />
	</Tab>
	<Tab title="进行中">
	  <Gml src="task_list.gml" data-status_group="grp_active" />
	</Tab>
	<Tab title="已完成">
	  <Gml src="task_list.gml" data-status_group="grp_done" />
	</Tab>
  </TabContainer>
</ui>
