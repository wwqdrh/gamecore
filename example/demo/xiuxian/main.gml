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
  </Control>
</ui>
