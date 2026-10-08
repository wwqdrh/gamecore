<ui>
  <!-- 修仙 Demo 组合根（Composition Root）—— 整个 demo 唯一同时"认识"各组件的地方：
	   跨目录装配在此完成，组件彼此零引用。
	   组件间调用走统一 UI 管理层（GdUIManager）：
		 · mainhud 功能按钮 → @pressed="show:TaskDrawer"（运行期按 ui_id 查注册表）
		 · GD 代码 → GdUIManager.find_ui("TaskDrawer").open()
	   新增组件三步：组件文件里声明 ui_id → 本文件 <Gml> 装配 → 其他组件按 id 调用 -->
  <!-- mouse_filter=ignore：组合根全屏覆盖在世界上，必须鼠标穿透——
	   否则根节点（默认 STOP）会被报为 hovered 并吞掉全部世界点击，
	   导致玩家"无 UI 悬停才开火"的射击分支永不成立（点击寻路同样被吞）。
	   交互子节点（按钮/装备栏/弹窗）各自声明鼠标行为，不受父级 IGNORE 影响 -->
  <Control name="DemoRoot" anchor="full" mouse_filter="ignore">
	<!-- 游戏主界面 HUD（底层） -->
	<Gml src="mainhud/mainhud.gml" />
	<!-- 任务抽屉（顶层，后挂载渲染在上；初始隐藏） -->
	<Gml src="task/task_drawer.gml" />
	<!-- 模态弹窗（<Modal> 组件：全屏遮罩 + 内容缩放动画，初始隐藏）：
		 内容 = 现成的面板 gml（<Gml> 引用即可），面板关闭按钮 emit
		 s_close_requested → Modal 自动整体关闭；
		 content_margin 弹窗四周留白，留白区点击落到遮罩 → 关闭 -->
	<Modal name="ProfileModal" ui_id="ProfileModal" anchor="full"
		   content_margin="56" close_on_overlay="true">
	  <Gml src="profile/profile_panel.gml" />
	</Modal>
	<Modal name="BagModal" ui_id="BagModal" anchor="full"
		   content_margin="56" close_on_overlay="true">
	  <Gml src="bag/bag_panel.gml" />
	</Modal>
	<Modal name="StoreModal" ui_id="StoreModal" anchor="full"
		   content_margin="56" close_on_overlay="true">
	  <Gml src="store/store_panel.gml" />
	</Modal>
	<!-- 设置弹窗：key_bind="escape" → 按 ESC 开/关（toggle），Modal 内建按键绑定 -->
	<Modal name="SettingModal" ui_id="SettingModal" anchor="full"
		   content_margin="56" close_on_overlay="true" key_bind="escape">
	  <Gml src="setting/settings_panel.gml" />
	</Modal>
  </Control>
</ui>
