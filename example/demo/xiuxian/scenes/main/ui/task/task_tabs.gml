<ui>
  <!-- 页签切换容器：复用框架 TabContainer/Tab，页签切换为原生行为
	   每个 Tab 页通过 <Gml> 引用 task_list.gml（各自独立实例）。
	   分类过滤：<script> 定义各页签分类标识，<Gml data-category="分类变量">
	   经具名数据映射注入子文件的 category 变量——task_list.gml 的 UIVList
	   filter_value_var="category" 取该值过滤 bean:task_list:tasks 全量数据，
	   各页签只显示自己分类的任务；
	   task_list.gd 每 5s 轮换分类插入任务时，对应页签的列表自动增长
	   （响应式数据绑定 + 分类过滤演示） -->
  <script>
	// 各页签的任务分类标识（与任务视图数据的 category 字段匹配）
	var cat_daily = "daily"
	var cat_main = "main"
	var cat_side = "side"
	var cat_guild = "guild"
	var cat_bounty = "bounty"
  </script>
  <TabContainer name="TaskTabs" anchor="full" tabs_visible="true" current_tab="0">
	<Tab title="日常">
	  <Gml src="task_list.gml" data-category="cat_daily" />
	</Tab>
	<Tab title="主线">
	  <Gml src="task_list.gml" data-category="cat_main" />
	</Tab>
	<Tab title="支线">
	  <Gml src="task_list.gml" data-category="cat_side" />
	</Tab>
	<Tab title="宗门">
	  <Gml src="task_list.gml" data-category="cat_guild" />
	</Tab>
	<Tab title="悬赏">
	  <Gml src="task_list.gml" data-category="cat_bounty" />
	</Tab>
  </TabContainer>
</ui>
