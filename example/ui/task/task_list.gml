<ui script="task_list.gd">
  <!-- 任务列表容器（可复用）：滚动列表
	   UIVList 的 slot 模板通过 <Gml> 直接引用 task_item.gml（构建期注入），
	   <script> 定义默认数据（task_list.gd 注册 GdBean 时取作初始值），
	   UIVList 用 data="bean:task_list:tasks" 声明式绑定 GdBean 全量任务——
	   GdGmlScene 场景加载后自动完成初始填充 + watch 注册，
	   Bean 属性变化（task_list.gd 每 5s 插入任务）时自动刷新全部同名列表。
	   分类过滤：filter_key="category"（数据字段）+ filter_value_var="category"
	   （过滤值取本文件 <script> 的 category 变量）——多页签复用本文件时，
	   引用方用 <Gml src="task_list.gml" data-category="分类变量" /> 注入各自分类，
	   本实例列表即成为"只显示该分类任务"的视图；category 为空（独立打开）显示全部 -->
  <script>
	// 任务分类标识：引用方 <Gml data-category="分类变量"> 注入覆盖，
	// 独立打开时为空 = 不分类（显示全部任务）
	var category = ""
	// 默认演示数据（task_list.gd 注册 GdBean 时取作初始值，此后数据由 Bean 驱动）
	var tasks = [
	  { category: "daily", icon: "📦", title: "日常·示例任务", desc: "task_list.gml 内置默认数据", progress: "1/2", reward1: "💎 10", reward2: "🪙 5", btn_text: "前往", btn_state: "go" },
	  { category: "daily", icon: "🌿", title: "日常·采集灵草", desc: "后山采集 5 株灵草", progress: "3/5", reward1: "💎 15", reward2: "🪙 8", btn_text: "前往", btn_state: "go" },
	  { category: "main", icon: "⚔️", title: "主线·击杀小怪", desc: "击败 10 只怪物", progress: "0/10", reward1: "💎 20", reward2: "🪙 8", btn_text: "前往", btn_state: "go" },
	  { category: "guild", icon: "🏯", title: "宗门·修建大殿", desc: "捐献木材 100 单位", progress: "60/100", reward1: "💎 30", reward2: "🪙 12", btn_text: "捐献", btn_state: "go" },
	  { category: "bounty", icon: "📜", title: "悬赏·缉拿逃犯", desc: "黑风寨方向追击", progress: "0/1", reward1: "💎 50", reward2: "🪙 20", btn_text: "追击", btn_state: "go" },
	]
  </script>
  <ScrollContainer name="TaskScroll" size_flags_vertical="expand_fill">
	<UIVList name="TaskList" size_flags_horizontal="expand_fill"
			 data="bean:task_list:tasks" filter_key="category" filter_value_var="category">
	  <Gml src="task_item.gml" />
	</UIVList>
  </ScrollContainer>
</ui>
