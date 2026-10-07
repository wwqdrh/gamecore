<ui>
  <!-- 顶中资源栏：灵石/丹药徽章 + 加号按钮。
	   @pressed 就近解析：本文件无脚本，沿父链回退到场景脚本
	   （mainhud.gd 的 _on_res_add） -->
  <style>
	.res-box {
	  background: #2e4a3acc;
	  border_color: #e8b93e;
	  border_width: 2;
	  border_radius: 22;
	}
	.res-num { color: #f2ead2; font_size: 17; }
	.plus-btn {
	  background: #5fae6a;
	  color: #f2ead2;
	  border_radius: 12;
	  font_size: 15;
	}
  </style>
  <HBoxContainer name="ResourceBar" separation="12">
	<!-- 灵石徽章 -->
	<Panel class="res-box" custom_minimum_size="168,44">
	  <HBoxContainer anchor="full" margin="14 6 8 6" separation="8">
		<Label text="💎" valign="center" font_size="16" />
		<Label class="res-num" text="12,800" valign="center" />
		<Control size_flags_horizontal="expand_fill" />
		<Button name="AddStoneBtn" class="plus-btn" text="＋" custom_minimum_size="26,26"
		        @pressed="_on_res_add" />
	  </HBoxContainer>
	</Panel>
	<!-- 丹药徽章 -->
	<Panel class="res-box" custom_minimum_size="140,44">
	  <HBoxContainer anchor="full" margin="14 6 8 6" separation="8">
		<Label text="🧪" valign="center" font_size="16" />
		<Label class="res-num" text="3/36" valign="center" />
		<Control size_flags_horizontal="expand_fill" />
		<Button name="AddPillBtn" class="plus-btn" text="＋" custom_minimum_size="26,26"
		        @pressed="_on_res_add" />
	  </HBoxContainer>
	</Panel>
  </HBoxContainer>
</ui>
