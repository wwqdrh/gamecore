<ui>
  <!-- 坊市顶栏：标题装饰框 + 灵石/灵玉双货币徽章（+号充值入口） + 关闭按钮。
	   回调（@pressed）就近解析：本文件无脚本，沿父链回退到场景脚本
	   （store_panel.gd 的 _on_close_pressed / _on_xxx_plus） -->
  <style>
	.topbar-bg {
	  background: #2e4a3a;
	  border_radius: 12;
	}
	.title-box {
	  background: #1f3628;
	  border_color: #c9a44a;
	  border_width: 2;
	  border_radius: 10;
	}
	.title-text { color: #e8c96a; font_size: 26; }
	.currency-box {
	  background: #243c2e;
	  border_color: #56705c;
	  border_width: 1;
	  border_radius: 14;
	}
	.currency-num { color: #f2ead2; font_size: 16; }
	.plus-btn {
	  background: #e8b93e;
	  color: #4a3410;
	  border_radius: 10;
	  font_size: 14;
	}
	.close-btn {
	  background: #3a5a46;
	  color: #f2ead2;
	  border_radius: 12;
	  font_size: 18;
	}
  </style>
  <Panel name="StoreTopBar" class="topbar-bg" custom_minimum_size="0,56">
	<HBoxContainer name="TopBarRow" anchor="full" margin="12 8 12 8">
	  <Panel class="title-box" custom_minimum_size="124,40">
		<Label class="title-text" text="坊  市" anchor="full" align="center" valign="center" />
	  </Panel>
	  <Control size_flags_horizontal="expand_fill" />
	  <!-- 灵石货币徽章 -->
	  <Panel class="currency-box" custom_minimum_size="168,36">
		<HBoxContainer anchor="full" margin="10 4 6 4">
		  <Label text="💎" valign="center" font_size="15" />
		  <Control custom_minimum_size="6,0" />
		  <Label class="currency-num" text="12,800" valign="center" />
		  <Control size_flags_horizontal="expand_fill" />
		  <Button class="plus-btn" text="＋" custom_minimum_size="26,26"
		          @pressed="_on_lingshi_plus" />
		</HBoxContainer>
	  </Panel>
	  <Control custom_minimum_size="8,0" />
	  <!-- 灵玉货币徽章 -->
	  <Panel class="currency-box" custom_minimum_size="140,36">
		<HBoxContainer anchor="full" margin="10 4 6 4">
		  <Label text="🟢" valign="center" font_size="15" />
		  <Control custom_minimum_size="6,0" />
		  <Label class="currency-num" text="260" valign="center" />
		  <Control size_flags_horizontal="expand_fill" />
		  <Button class="plus-btn" text="＋" custom_minimum_size="26,26"
		          @pressed="_on_lingyu_plus" />
		</HBoxContainer>
	  </Panel>
	  <Control custom_minimum_size="8,0" />
	  <Button name="CloseBtn" class="close-btn" text="✕" custom_minimum_size="40,40"
	          @pressed="_on_close_pressed" />
	</HBoxContainer>
  </Panel>
</ui>
