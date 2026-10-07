<ui>
  <!-- 顶栏：角色名标题 + 境界副标题徽章 + 右上角关闭按钮
	   CloseBtn @pressed 回退绑定到 GdGmlScene 场景脚本（本文件无 <ui script>） -->
  <style>
	.top-title { color: #3c2f1c; }
	.realm-badge {
	  background: #5e8d7a;
	  color: #f2ecd9;
	  border_radius: 10;
	}
	.close-btn {
	  background: #6b5a3e;
	  color: #f2ecd9;
	  border_radius: 16;
	}
  </style>
  <VBoxContainer name="TopBar">
	<HBoxContainer name="TitleRow">
	  <Control size_flags_horizontal="expand_fill" />
	  <Label name="TitleLabel" text="道友 · 云无涯" class="top-title" font_size="30" valign="center" />
	  <Control size_flags_horizontal="expand_fill" />
	  <Button name="CloseBtn" text="✕" class="close-btn" custom_minimum_size="36,36" @pressed="_on_close_pressed" />
	</HBoxContainer>
	<Control custom_minimum_size="0,6" />
	<CenterContainer name="BadgeRow">
	  <Panel class="realm-badge" custom_minimum_size="120,30">
		<Label name="RealmBadgeText" text="炼气三层" font_size="15" align="center" valign="center" anchor="full" />
	  </Panel>
	</CenterContainer>
  </VBoxContainer>
</ui>
