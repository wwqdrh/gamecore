<ui theme="cartoon">
  <!-- 页签切换容器：复用框架 TabContainer/Tab，页签切换为原生行为
	   每个 Tab 页内挂载一个 task_list.gml 实例（见 task_panel.gd） -->
  <TabContainer name="TaskTabs" anchor="full" tabs_visible="true" current_tab="0">
	<Tab title="日常">
	  <Control name="ListSlot0" size_flags_horizontal="expand_fill" size_flags_vertical="expand_fill" />
	</Tab>
	<Tab title="主线">
	  <Control name="ListSlot1" size_flags_horizontal="expand_fill" size_flags_vertical="expand_fill" />
	</Tab>
	<Tab title="宗门">
	  <Control name="ListSlot2" size_flags_horizontal="expand_fill" size_flags_vertical="expand_fill" />
	</Tab>
	<Tab title="悬赏">
	  <Control name="ListSlot3" size_flags_horizontal="expand_fill" size_flags_vertical="expand_fill" />
	</Tab>
  </TabContainer>
</ui>
