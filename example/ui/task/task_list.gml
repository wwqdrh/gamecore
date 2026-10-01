<ui theme="cartoon">
  <!-- 任务列表容器（可复用）：滚动列表
	   UIVList 的 slot 模板通过 <Gml> 直接引用 task_item.gml，
	   构建期即注入模板，控制器只需 update(data) 驱动刷新 -->
  <ScrollContainer name="TaskScroll" size_flags_vertical="expand_fill">
	<UIVList name="TaskList" size_flags_horizontal="expand_fill">
	  <Gml src="task_item.gml" />
	</UIVList>
  </ScrollContainer>
</ui>
