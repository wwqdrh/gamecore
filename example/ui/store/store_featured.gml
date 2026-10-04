<ui>
  <!-- 右侧推荐商品卡：标题横幅 + 大图占位（正式版替换为商品贴图） + 极品角标
	   + 名称 + 描述引文。数据静态展示（推荐位由运营配置，不做列表驱动） -->
  <style>
	.featured-bg {
	  background: #f6f0de;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 12;
	}
	.featured-title {
	  background: #2e4a3a;
	  border_color: #c9a44a;
	  border_width: 2;
	  border_radius: 10;
	}
	.featured-title-text { color: #e8c96a; font_size: 18; }
	.featured-name { color: #3c2f1c; font_size: 22; }
	.featured-desc { color: #8a7a5a; font_size: 14; }
	.rarity-tag {
	  background: #c4922a;
	  border_color: #8a6a1e;
	  border_width: 2;
	  border_radius: 6;
	}
	.rarity-text { color: #ffffff; font_size: 14; }
  </style>
  <Panel name="FeaturedPanel" class="featured-bg">
	<VBoxContainer name="FeaturedColumn" anchor="full" margin="14">
	  <Panel class="featured-title" custom_minimum_size="0,38">
		<Label class="featured-title-text" text="推 荐 商 品" anchor="full" align="center" valign="center" />
	  </Panel>
	  <Control custom_minimum_size="0,10" />
	  <Control name="FeatureShowcase" size_flags_vertical="expand_fill">
		<!-- 商品大图占位 -->
		<CenterContainer anchor="full" margin="0 36 0 10">
		  <Label name="FeaturedIcon" text="🏺" font_size="88" />
		</CenterContainer>
		<!-- 极品角标：右上竖排 -->
		<Panel class="rarity-tag" anchor="top_right" margin="0 10 10 0"
		       custom_minimum_size="30,64">
		  <Label class="rarity-text" text="极
品" anchor="full" align="center" valign="center" />
		</Panel>
	  </Control>
	  <Label name="FeaturedName" class="featured-name" text="紫金葫芦" align="center" />
	  <Control custom_minimum_size="0,8" />
	  <Label name="FeaturedDesc" class="featured-desc"
	         text="“吸纳天地灵气，助你修炼事半功倍！”" align="center" />
	</VBoxContainer>
  </Panel>
</ui>
