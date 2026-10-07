<ui script="bag_detail.gd">
  <!-- 右侧物品详情卡：大图占位 + 名称 + 品质徽章 + 描述 + 装饰印章
	   + 底部操作按钮（使用/出售/合成）。
	   数据静态展示（选中物品变化时由业务脚本填充，demo 与设计图一致）；
	   @pressed 回调就近绑定本文件脚本 -->
  <style>
	.detail-bg {
	  background: #f2ecd8;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 10;
	}
	.icon-box {
	  background: #dce8f0;
	  border_color: #c9a44a;
	  border_width: 3;
	  border_radius: 10;
	}
	.detail-name { color: #3c2f1c; font_size: 24; }
	.quality-badge {
	  background: #d4a017;
	  border_color: #a87c10;
	  border_width: 1;
	  border_radius: 6;
	}
	.quality-text { color: #ffffff; font_size: 14; }
	.detail-desc { color: #6b5a3e; font_size: 15; }
	.seal-text { color: #b03a2e; font_size: 26; }
	.use-btn {
	  background: #e8b93e;
	  color: #4a3410;
	  border_color: #8a6a1e;
	  border_width: 2;
	  border_radius: 10;
	  font_size: 18;
	}
	.sub-btn {
	  background: #3a5a46;
	  color: #f2ead2;
	  border_radius: 10;
	  font_size: 16;
	}
  </style>
  <Panel name="DetailPanel" class="detail-bg" anchor="full">
	<VBoxContainer name="DetailColumn" anchor="full" margin="14" separation="8">
	  <Control size_flags_vertical="expand_fill">
		<!-- 大图占位（正式版替换为物品贴图） -->
		<CenterContainer anchor="full" margin="0 40 0 10">
		  <Panel class="icon-box" custom_minimum_size="120,120">
			<Label name="DetailIcon" text="🏺" anchor="full" align="center"
			       valign="center" font_size="64" />
		  </Panel>
		</CenterContainer>
		<!-- 装饰印章：右下 -->
		<Label name="SealText" class="seal-text" text="闲"
		       anchor="bottom_right" margin="0 0 16 10" />
	  </Control>
	  <Label name="DetailName" class="detail-name" text="聚气丹" align="center" />
	  <CenterContainer>
		<Panel class="quality-badge" custom_minimum_size="64,26">
		  <Label name="QualityText" class="quality-text" text="上品"
		         anchor="full" align="center" valign="center" />
		</Panel>
	  </CenterContainer>
	  <Label name="DetailDesc" class="detail-desc"
	         text="服用后可快速聚集天地灵气，提升修为。是修仙者常用的助修丹药。"
	         align="center" autowrap_mode="3" size_flags_vertical="expand_fill" />
	  <HBoxContainer separation="10">
		<Button name="UseBtn" class="use-btn" text="使  用" custom_minimum_size="120,44"
		        size_flags_horizontal="expand_fill" @pressed="_on_use_pressed" />
		<Button name="SellBtn" class="sub-btn" text="出售" custom_minimum_size="72,44"
		        @pressed="_on_sell_pressed" />
		<Button name="ComposeBtn" class="sub-btn" text="合成" custom_minimum_size="72,44"
		        @pressed="_on_compose_pressed" />
	  </HBoxContainer>
	</VBoxContainer>
  </Panel>
</ui>
