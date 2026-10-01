<ui theme="cartoon">
  <!-- 任务列表容器（可复用）：滚动列表
	   每个页签挂一个实例；UIVList 的 slot 模板由控制器注入（task_item.gml） -->
  <ScrollContainer name="TaskScroll" size_flags_vertical="expand_fill">
	<UIVList name="TaskList" size_flags_horizontal="expand_fill" />
  </ScrollContainer>
</ui>
