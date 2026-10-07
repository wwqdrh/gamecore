<ui script="store_item.gd">
  <!-- 商品卡条目模板：作为 store_goods.gml 中 UIGrid 的 slot 模板（构建期注入）。
	   数据契约见 store_item.gd 的 @export 列表：
	   quality(品质角标，空串隐藏) / icon / title / price / price_old(划线原价，空串隐藏)
	   / countdown(限时倒计时，空串隐藏) / sold_out(售罄态：置灰+印章+按钮禁用)
	   {{key}} 模板绑定负责静态文本渲染，契约注入负责动态状态联动 -->
  <style>
	.goods-card {
	  background: #faf4e2;
	  border_color: #b8a06a;
	  border_width: 2;
	  border_radius: 12;
	}
	.quality-tag {
	  background: #ffffff;
	  border_radius: 8;
	}
	.quality-text { color: #ffffff; font_size: 14; }
	.goods-title { color: #4a3a22; font_size: 17; }
	.goods-price { color: #3f6b58; font_size: 15; }
	.price-old { color: #a89880; font_size: 13; }
	.countdown { color: #c47a2a; font_size: 13; }
	.stamp-box {
	  background: #ffffff;
	  border_color: #b03a2e;
	  border_width: 3;
	  border_radius: 8;
	}
	.stamp-text { color: #b03a2e; font_size: 24; }
	.buy-btn {
	  background: #e8b93e;
	  color: #4a3410;
	  border_color: #8a6a1e;
	  border_width: 2;
	  border_radius: 12;
	  font_size: 16;
	}
	.buy-btn-disabled {
	  background: #b8b0a0;
	  color: #6b6456;
	  border_color: #8a8272;
	  border_width: 2;
	  border_radius: 12;
	  font_size: 16;
	}
  </style>
  <!-- 根节点双向 expand_fill：作为 UIGrid 条目时格子拉伸填满列宽/行高
       （GridContainer 只拉伸带 expand 标志的子节点），独立打开预览时无副作用 -->
  <Panel name="GoodsCard" class="goods-card" custom_minimum_size="205,190"
         size_flags_horizontal="expand_fill" size_flags_vertical="expand_fill">
	<!-- 品质角标：左上竖排小徽章（modulate 由条目脚本按品质染色） -->
	<Panel name="QualityTag" class="quality-tag" anchor="top_left" margin="8 8 0 0"
	       custom_minimum_size="26,54">
	  <Label name="QualityText" class="quality-text" anchor="full" align="center" valign="center" />
	</Panel>
	<!-- 限时倒计时：右上（默认隐藏，countdown 契约非空时显示）。
	     模板绑定只支持整值插值（text="{{key}}"），⏱ 前缀写在数据值里 -->
	<Label name="CountdownLabel" class="countdown" text="{{countdown}}"
	       anchor="top_right" margin="0 8 8 0" visible="false" />
	<VBoxContainer name="CardColumn" anchor="full" margin="14 14 14 12">
	  <CenterContainer name="IconRow" size_flags_vertical="expand_fill">
		<Label name="GoodsIcon" text="{{icon}}" font_size="54" />
	  </CenterContainer>
	  <Label name="GoodsTitle" class="goods-title" text="{{title}}" align="center" />
	  <Control custom_minimum_size="0,4" />
	  <CenterContainer name="PriceRow">
		<HBoxContainer>
		  <Label text="💎" valign="center" font_size="14" />
		  <Control custom_minimum_size="4,0" />
		  <Label class="goods-price" text="{{price}}" valign="center" />
		  <Label class="goods-price" text=" 灵石" valign="center" />
		  <Label name="PriceOld" class="price-old" text="{{price_old}}" valign="center" visible="false" />
		</HBoxContainer>
	  </CenterContainer>
	  <Control custom_minimum_size="0,6" />
	  <CenterContainer name="BuyRow">
		<Button name="BuyBtn" class="buy-btn" text="购  买"
		        custom_minimum_size="110,36" @pressed="_on_buy_pressed" />
	  </CenterContainer>
	</VBoxContainer>
	<!-- 售罄印章：中央红框（sold_out 契约联动显示，配合条目整体置灰） -->
	<Panel name="SoldOutStamp" class="stamp-box" anchor="center"
	       custom_minimum_size="110,52" visible="false">
	  <Label name="StampText" class="stamp-text" text="售  罄" anchor="full" align="center" valign="center" />
	</Panel>
  </Panel>
</ui>
