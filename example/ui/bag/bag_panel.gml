<ui script="bag_panel.gd">
<!-- 修仙储物袋面板：主骨架（布局 + <Gml> 组合各区块）
	 解析自设计图：顶栏（卷轴标题/灵石/容量/关闭）
	 + HBox 三列：左分类列表(1) / 中 8x6 物品网格(4) / 右物品详情卡(2.3)
	 跨区块联动 = GdState 临时状态总线（单例 GDSTATE，非持久化，
	 见 bag_panel.gd）：分类/网格/详情互不知道对方，只通过
	 set_state("bag.category"/"bag.selected_item") 上报命令，
	 watch 同名键各自更新自己
	 运行验证：godot --headless --path . -s res://example/ui/bag/check_bag_ui.gd -->
  <style>
	.window-bg {
	  background: #f0e8d2;
	  border_color: #7a6a4a;
	  border_width: 3;
	  border_radius: 16;
	}
  </style>
  <Panel name="BagPanel" class="window-bg" anchor="full" margin="2%">
	<MarginContainer name="MainMargin" anchor="full" margin="14">
	  <VBoxContainer name="MainColumn">
		<Gml src="bag_topbar.gml" />
		<Control custom_minimum_size="0,10" />
		<HBoxContainer name="BodyRow" size_flags_horizontal="expand_fill"
					   size_flags_vertical="expand_fill">
		  <Gml src="bag_categories.gml" size_flags_horizontal="expand_fill" stretch_ratio="1" />
		  <Control custom_minimum_size="12,0" />
		  <Gml src="bag_grid.gml" size_flags_horizontal="expand_fill" stretch_ratio="4" />
		  <Control custom_minimum_size="12,0" />
		  <Gml src="bag_detail.gml" size_flags_horizontal="expand_fill" stretch_ratio="2.3" />
		</HBoxContainer>
	  </VBoxContainer>
	</MarginContainer>
  </Panel>
</ui>
