<ui>
  <!-- 修仙坊市面板：主骨架（布局 + <Gml> 组合各区块）
	   解析自设计图：顶栏（标题/双货币徽章/关闭）+ TabContainer（每页内
	   HBoxContainer 分隔：左侧 3x2 商品网格 / 右侧推荐商品卡）+ 底部限购进度
	   运行验证：godot --headless --path . -s res://example/ui/store/check_store_ui.gd -->
  <style>
	.window-bg {
	  background: #f0e8d2;
	  border_color: #7a6a4a;
	  border_width: 3;
	  border_radius: 16;
	}
  </style>
  <Panel name="StorePanel" class="window-bg" anchor="full" margin="2%">
	<MarginContainer name="MainMargin" anchor="full" margin="16">
	  <VBoxContainer name="MainColumn">
		<Gml src="store_topbar.gml" />
		<Control custom_minimum_size="0,10" />
		<Gml src="store_tabs.gml" size_flags_horizontal="expand_fill"
		     size_flags_vertical="expand_fill" />
		<Control custom_minimum_size="0,10" />
		<Gml src="store_footer.gml" />
	  </VBoxContainer>
	</MarginContainer>
  </Panel>
</ui>
