<ui theme="cartoon">
  <!-- 页签切换容器：复用框架 TabContainer/Tab，页签切换为原生行为
	   每个 Tab 页通过 <Gml> 引用 task_list.gml（各自独立实例），
	   task_list.gml 内部 data="bean:task_list:tasks" 绑定 GdBean 数据源——
	   全部页签共享同一份 Bean 数据，task_list.gd 每 5s 插入任务时
	   所有页签的列表同时刷新（响应式数据绑定演示） -->
  <TabContainer name="TaskTabs" anchor="full" tabs_visible="true" current_tab="0">
	<Tab title="日常">
	  <Gml src="task_list.gml" />
	</Tab>
	<Tab title="主线">
	  <Gml src="task_list.gml" />
	</Tab>
	<Tab title="宗门">
	  <Gml src="task_list.gml" />
	</Tab>
	<Tab title="悬赏">
	  <Gml src="task_list.gml" />
	</Tab>
  </TabContainer>
</ui>
