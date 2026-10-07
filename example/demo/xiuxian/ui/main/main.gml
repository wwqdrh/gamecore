<ui>
  <!-- 修仙 Demo 组合根（Composition Root）—— 整个 demo 唯一同时"认识"各组件的地方：
	   跨目录装配在此完成，组件彼此零引用。
	   组件间调用走统一 UI 管理层（GdUIManager）：
		 · mainhud 功能按钮 → @pressed="show:TaskDrawer"（运行期按 ui_id 查注册表）
		 · GD 代码 → GdUIManager.find_ui("TaskDrawer").open()
	   新增组件三步：组件文件里声明 ui_id → 本文件 <Gml> 装配 → 其他组件按 id 调用 -->
  <Control name="DemoRoot" anchor="full">
	<!-- 游戏主界面 HUD（底层） -->
	<Gml src="ui/mainhud/mainhud.gml" />
	<!-- 任务抽屉（顶层，后挂载渲染在上；初始隐藏） -->
	<Gml src="ui/task/task_drawer.gml" />
	<!-- 模态弹窗（<Modal> 组件：全屏遮罩 + 内容缩放动画，初始隐藏）：
		 内容 = 现成的面板 gml（<Gml> 引用即可），面板关闭按钮 emit
		 s_close_requested → Modal 自动整体关闭；
		 content_margin 弹窗四周留白，留白区点击落到遮罩 → 关闭 -->
	<Modal name="ProfileModal" ui_id="ProfileModal" anchor="full"
		   content_margin="56" close_on_overlay="true">
	  <Gml src="ui/profile/profile_panel.gml" />
	</Modal>
	<Modal name="BagModal" ui_id="BagModal" anchor="full"
		   content_margin="56" close_on_overlay="true">
	  <Gml src="ui/bag/bag_panel.gml" />
	</Modal>
	<Modal name="StoreModal" ui_id="StoreModal" anchor="full"
		   content_margin="56" close_on_overlay="true">
	  <Gml src="ui/store/store_panel.gml" />
	</Modal>
  </Control>
</ui>
