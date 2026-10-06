<ui>
  <!-- 任务抽屉组件（自包含，可独立挂载/独立 F6 运行）：
       本文件只提供"任务抽屉"本身，不引用任何其他面板。
       ui_id="TaskDrawer" 把组件注册进统一 UI 管理层（GdUIManager）——
       任意面板无需 import 本文件夹任何文件，即可跨组件调用：
         · GML 内部动作：@pressed="show:TaskDrawer"（mainhud 功能按钮即此方式）
         · GD 代码：GdUIManager.find_ui("TaskDrawer").open()
       组合根 example/demo/xiuxian/main.gml 负责把本组件与 mainhud 装配到同一棵树 -->
  <Drawer name="TaskDrawer" ui_id="TaskDrawer" anchor="full" direction="left"
          slide_width="50%" drawer_title="任务" close_on_overlay="true">
    <Gml src="task_tabs.gml" size_flags_vertical="expand_fill" />
  </Drawer>
</ui>
