<ui script="bag_grid.gd">
  <!-- 背包物品网格（可复用）：UIGrid 8 列 x 6 行 + 分类过滤 + 点选联动详情。
	   数据绑定链路：数据完全由状态层 XiuItemState 驱动
	   （example/demo/xiuxian/state/item/item_state.gd，bean id: xiuxian_item）——
	   data="bean:xiuxian_item:bag_views" 声明保留为 __data_var meta（构建期不解析），
	   初始填充 + watch 响应式刷新由 bag_grid.gd 手动接线
	   （组合根直开 tscn 场景不走 GdGmlScene auto_bind_data 链路）；
	   视图数据 = 全量道具总表 + 背包持有量派生（只含已持有道具），
	   道具增减（add_item/remove_item）后网格自动刷新。
	   filter_key="category" + filter_value_var="category" 按本文件 category 变量过滤
	   ——初始值 "pill"（与 bag_panel.gd 初始状态一致）；运行时过滤由状态下行驱动：
	   网格监听 panel.category_changed 后 set_meta("__filter_value") + update 全量数据重刷；
	   空串 = 不过滤（"全部"）。
	   点选链路（命令上行）：UIGrid @s_click_item 回调本文件脚本，
	   条目选中互斥（selected 契约）+ panel.select_item(完整条目数据) 上报 -->
  <style>
	.grid-bg {
	  background: #e9e2c8;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 10;
	}
  </style>
  <script>
	// 当前分类（"pill" = 丹药页初始选中；运行时以 panel.category 状态为准）
	var category = "pill"
  </script>
  <Panel name="GoodsPanel" class="grid-bg" anchor="full">
	<ScrollContainer name="GoodsScroll" anchor="full" margin="8">
	  <UIGrid name="GoodsGrid" columns="8"
			 data="bean:xiuxian_item:bag_views"
			 filter_key="category" filter_value_var="category"
			 h_separation="6" v_separation="6"
			 @s_click_item="_on_item_clicked"
			 size_flags_horizontal="expand_fill" size_flags_vertical="expand_fill">
		<!-- slot 模板：背包格子条目（构建期注入，duplicate 复制） -->
		<Gml src="bag_item.gml" />
	  </UIGrid>
	</ScrollContainer>
  </Panel>
</ui>
