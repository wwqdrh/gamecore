<ui theme="cartoon">
  <!-- 页签切换容器：复用框架 TabContainer/Tab，页签切换为原生行为
	   每个 Tab 页通过 <Gml> 直接引用 task_list.gml（各自独立实例） -->
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
