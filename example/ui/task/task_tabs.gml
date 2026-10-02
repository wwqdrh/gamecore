<ui theme="cartoon">
  <!-- 页签切换容器：复用框架 TabContainer/Tab，页签切换为原生行为
	   每个 Tab 页通过 <Gml> 引用 task_list.gml（各自独立实例），
	   并用 data-tasks="变量名" 把本文件的 <script> 变量映射到
	   task_list.gml 的 tasks 变量（data-后跟子文件变量名） -->
  <script>
	// 各页签任务数据（数据与 UI 同文件声明，构建期直接绑定到列表）
	// btn_state: go=前往 claim=可领取(金色) done=已完成(置灰)
	var daily_tasks = [
	  { icon: "🌿", title: "采集灵草", desc: "在宗门后山采集灵草", progress: "3/5", reward1: "💎 100", reward2: "🪙 50", btn_text: "前往", btn_state: "go" },
	  { icon: "🐺", title: "击败妖狼", desc: "击败宗门山林中的妖狼", progress: "8/8", reward1: "💎 150", reward2: "🪙 75", btn_text: "领取", btn_state: "claim" },
	  { icon: "🏺", title: "炼制聚气丹", desc: "炼制聚气丹", progress: "1/1", reward1: "💎 120", reward2: "🪙 60", btn_text: "已完成", btn_state: "done" },
	  { icon: "📜", title: "传功授业", desc: "为宗门弟子传功1次", progress: "0/1", reward1: "💎 80", reward2: "🪙 40", btn_text: "前往", btn_state: "go" },
	  { icon: "🧰", title: "捐献物资", desc: "向宗门仓库捐献物资5次", progress: "2/5", reward1: "💎 100", reward2: "🪙 50", btn_text: "前往", btn_state: "go" },
	]
	var main_tasks = [
	  { icon: "🗡", title: "初入宗门", desc: "与执事长老对话", progress: "1/1", reward1: "💎 200", reward2: "🪙 100", btn_text: "领取", btn_state: "claim" },
	  { icon: "🏔", title: "后山历练", desc: "在后山存活一炷香时间", progress: "0/1", reward1: "💎 300", reward2: "🪙 150", btn_text: "前往", btn_state: "go" },
	]
	var sect_tasks = [
	  { icon: "🥋", title: "宗门大比", desc: "报名参加季度大比", progress: "0/1", reward1: "💎 500", reward2: "🪙 200", btn_text: "前往", btn_state: "go" },
	  { icon: "🏮", title: "坊市采购", desc: "为坊市补充3件货物", progress: "1/3", reward1: "💎 120", reward2: "🪙 60", btn_text: "前往", btn_state: "go" },
	]
	var bounty_tasks = [
	  { icon: "🏴", title: "清剿山贼", desc: "击退黑风寨山贼10名", progress: "4/10", reward1: "💎 400", reward2: "🪙 180", btn_text: "前往", btn_state: "go" },
	  { icon: "🐎", title: "护送商队", desc: "护送商队至青州城", progress: "0/1", reward1: "💎 350", reward2: "🪙 160", btn_text: "前往", btn_state: "go" },
	]
  </script>
  <TabContainer name="TaskTabs" anchor="full" tabs_visible="true" current_tab="0">
	<Tab title="日常">
	  <Gml src="task_list.gml" data-tasks="daily_tasks" />
	</Tab>
	<Tab title="主线">
	  <Gml src="task_list.gml" data-tasks="main_tasks" />
	</Tab>
	<Tab title="宗门">
	  <Gml src="task_list.gml" data-tasks="sect_tasks" />
	</Tab>
	<Tab title="悬赏">
	  <Gml src="task_list.gml" data-tasks="bounty_tasks" />
	</Tab>
  </TabContainer>
</ui>
