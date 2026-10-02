<ui theme="cartoon">
  <!-- 任务列表容器（可复用）：滚动列表
	   UIVList 的 slot 模板通过 <Gml> 直接引用 task_item.gml（构建期注入），
	   <script> 定义默认数据，UIVList 用 data="tasks" 直接绑定——
	   构建期即填充条目，被 <Gml> 引用时可用 data-tasks="变量名"
	   （data-后跟本文件的 tasks 变量名）映射为引用方的变量 -->
  <script>
	// 默认演示数据（真实项目可来自服务器 / GdBean，控制器运行时 update 覆盖）
	var tasks = [
	  { icon: "📦", title: "示例任务", desc: "task_list.gml 内置默认数据", progress: "1/2", reward1: "💎 10", reward2: "🪙 5", btn_text: "前往", btn_state: "go" },
	  { icon: "⚔️", title: "击杀小怪", desc: "击败 10 只怪物", progress: "0/10", reward1: "💎 20", reward2: "🪙 8", btn_text: "前往", btn_state: "go" },
	]
  </script>
  <ScrollContainer name="TaskScroll" size_flags_vertical="expand_fill">
	<UIVList name="TaskList" size_flags_horizontal="expand_fill" data="tasks">
	  <Gml src="task_item.gml" />
	</UIVList>
  </ScrollContainer>
</ui>
