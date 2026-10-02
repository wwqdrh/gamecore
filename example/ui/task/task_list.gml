<ui theme="cartoon" script="task_list.gd">
  <!-- 任务列表容器（可复用）：滚动列表
	   UIVList 的 slot 模板通过 <Gml> 直接引用 task_item.gml（构建期注入），
	   <script> 定义默认数据（task_list.gd 注册 GdBean 时取作初始值），
	   UIVList 用 data="bean:task_list:tasks" 声明式绑定 GdBean 属性——
	   GdGmlScene 场景加载后自动完成初始填充 + watch 注册，
	   Bean 属性变化（task_list.gd 每 5s 插入任务）时自动刷新全部同名列表。
	   被引用时也可用 data-tasks="变量名" 覆盖 tasks 变量（静态数据模式） -->
  <script>
	// 默认演示数据（task_list.gd 注册 GdBean 时取作初始值，
	// 此后数据由 Bean 驱动：每 5s 插入一条动态任务）
	var tasks = [
	  { icon: "📦", title: "示例任务", desc: "task_list.gml 内置默认数据", progress: "1/2", reward1: "💎 10", reward2: "🪙 5", btn_text: "前往", btn_state: "go" },
	  { icon: "⚔️", title: "击杀小怪", desc: "击败 10 只怪物", progress: "0/10", reward1: "💎 20", reward2: "🪙 8", btn_text: "前往", btn_state: "go" },
	]
  </script>
  <ScrollContainer name="TaskScroll" size_flags_vertical="expand_fill">
	<UIVList name="TaskList" size_flags_horizontal="expand_fill" data="bean:task_list:tasks">
	  <Gml src="task_item.gml" />
	</UIVList>
  </ScrollContainer>
</ui>
