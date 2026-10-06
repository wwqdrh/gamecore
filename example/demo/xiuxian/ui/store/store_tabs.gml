<ui>
  <!-- 坊市页签：TabContainer 原生切换；每个 Tab 页内 HBoxContainer 分隔
	   左侧商品网格（store_goods.gml，stretch_ratio 2）与右侧推荐商品卡
	   （store_featured.gml，stretch_ratio 1）——每个页签是独立的
	   "网格 + 推荐位"组合，各自注入分类（data-category 具名映射） -->
  <script>
	// 各页签的商品分类标识（与商品数据的 category 字段匹配）
	var cat_herb = "herb"
	var cat_tool = "tool"
	var cat_skill = "skill"
	var cat_limited = "limited"
  </script>
  <TabContainer name="StoreTabs" size_flags_horizontal="expand_fill"
	        size_flags_vertical="expand_fill" current_tab="0">
	<Tab title="🌿 灵药">
	  <HBoxContainer name="TabBody" size_flags_horizontal="expand_fill"
	                 size_flags_vertical="expand_fill">
		<Gml src="store_goods.gml" data-category="cat_herb"
		     size_flags_horizontal="expand_fill" stretch_ratio="2" />
		<Control custom_minimum_size="12,0" />
		<Gml src="store_featured.gml" size_flags_horizontal="expand_fill" stretch_ratio="1" />
	  </HBoxContainer>
	</Tab>
	<Tab title="🗡️ 法器">
	  <HBoxContainer name="TabBody" size_flags_horizontal="expand_fill"
	                 size_flags_vertical="expand_fill">
		<Gml src="store_goods.gml" data-category="cat_tool"
		     size_flags_horizontal="expand_fill" stretch_ratio="2" />
		<Control custom_minimum_size="12,0" />
		<Gml src="store_featured.gml" size_flags_horizontal="expand_fill" stretch_ratio="1" />
	  </HBoxContainer>
	</Tab>
	<Tab title="📖 功法">
	  <HBoxContainer name="TabBody" size_flags_horizontal="expand_fill"
	                 size_flags_vertical="expand_fill">
		<Gml src="store_goods.gml" data-category="cat_skill"
		     size_flags_horizontal="expand_fill" stretch_ratio="2" />
		<Control custom_minimum_size="12,0" />
		<Gml src="store_featured.gml" size_flags_horizontal="expand_fill" stretch_ratio="1" />
	  </HBoxContainer>
	</Tab>
	<Tab title="⏳ 限时">
	  <HBoxContainer name="TabBody" size_flags_horizontal="expand_fill"
	                 size_flags_vertical="expand_fill">
		<Gml src="store_goods.gml" data-category="cat_limited"
		     size_flags_horizontal="expand_fill" stretch_ratio="2" />
		<Control custom_minimum_size="12,0" />
		<Gml src="store_featured.gml" size_flags_horizontal="expand_fill" stretch_ratio="1" />
	  </HBoxContainer>
	</Tab>
  </TabContainer>
</ui>
