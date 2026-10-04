<ui>
  <!-- 储物袋顶栏：金色卷轴标题 + 灵石货币徽章 + 背包容量徽章 + 关闭按钮。
	   回调（@pressed）就近解析：本文件无脚本，沿父链回退到场景脚本
	   （bag_panel.gd 的 _on_close_pressed） -->
  <style>
	.topbar-bg {
	  background: #2e4a3a;
	  border_radius: 12;
	}
	.title-box {
	  background: #e8d9a8;
	  border_color: #8a6a1e;
	  border_width: 2;
	  border_radius: 8;
	}
	.title-text { color: #6b4a1e; font_size: 24; }
	.currency-box {
	  background: #243c2e;
	  border_color: #56705c;
	  border_width: 1;
	  border_radius: 14;
	}
	.currency-num { color: #f2ead2; font_size: 16; }
	.close-btn {
	  background: #3a7a5a;
	  color: #f2ead2;
	  border_radius: 20;
	  font_size: 18;
	}
  </style>
  <Panel name="BagTopBar" class="topbar-bg" custom_minimum_size="0,56">
	<HBoxContainer name="TopBarRow" anchor="full" margin="12 8 12 8">
	  <Panel class="title-box" custom_minimum_size="140,40">
		<Label class="title-text" text="储 物 袋" anchor="full" align="center" valign="center" />
	  </Panel>
	  <Control size_flags_horizontal="expand_fill" />
	  <!-- 灵石货币徽章 -->
	  <Panel class="currency-box" custom_minimum_size="150,36">
		<HBoxContainer anchor="full" margin="10 4 10 4" separation="8">
		  <Label text="💎" valign="center" font_size="15" />
		  <Label class="currency-num" text="12,800" valign="center" />
		</HBoxContainer>
	  </Panel>
	  <Control custom_minimum_size="8,0" />
	  <!-- 背包容量徽章 -->
	  <Panel class="currency-box" custom_minimum_size="120,36">
		<HBoxContainer anchor="full" margin="10 4 10 4" separation="8">
		  <Label text="👝" valign="center" font_size="15" />
		  <Label name="CapacityText" class="currency-num" text="86/120" valign="center" />
		</HBoxContainer>
	  </Panel>
	  <Control custom_minimum_size="8,0" />
	  <Button name="CloseBtn" class="close-btn" text="✕" custom_minimum_size="40,40"
	          @pressed="_on_close_pressed" />
	</HBoxContainer>
  </Panel>
</ui>
