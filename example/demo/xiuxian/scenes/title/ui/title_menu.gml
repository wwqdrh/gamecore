<ui script="title_menu.gd">
<!-- 仙途 Demo 标题页：居中标题 + 开始游戏 / 退出游戏。
	 开始游戏 → 沿父链找 GdScene 场景根 → change_scene("xiuxian_main")，
	 经 GdSceneRoot 转场（遮罩淡入淡出）进入主场景。 -->
  <style>
	.title-bg {
	  background: #16202c;
	}
	.title-text { color: #e8d9a8; font_size: 72; }
	.subtitle-text { color: #8a9ab0; font_size: 18; }
	.menu-btn {
	  background: #2e4a3a;
	  color: #f2ead2;
	  border_color: #8a6a1e;
	  border_width: 2;
	  border_radius: 10;
	  font_size: 22;
	}
	.menu-btn-quit {
	  background: #3a3040;
	}
	.footnote-text { color: #5a6a7a; font_size: 13; }
  </style>
  <Panel name="TitleMenu" class="title-bg" anchor="full">
	<VBoxContainer name="MenuColumn" anchor="full" separation="16">
	  <Control size_flags_vertical="expand_fill" />
	  <Label class="title-text" text="仙  途" align="center" />
	  <Label class="subtitle-text" text="— 修 仙 问 道 · demo —" align="center" />
	  <Control custom_minimum_size="0,32" />
	  <Button name="BtnStart" class="menu-btn" text="开 始 游 戏"
			  custom_minimum_size="260,56" size_flags_horizontal="shrink_center"
			  @pressed="_on_start_pressed" />
	  <Button name="BtnQuit" class="menu-btn menu-btn-quit" text="退 出 游 戏"
			  custom_minimum_size="260,56" size_flags_horizontal="shrink_center"
			  @pressed="_on_quit_pressed" />
	  <Control size_flags_vertical="expand_fill" />
	  <Label class="footnote-text" text="ESC 打开设置 · GML 框架 demo" align="center" />
	</VBoxContainer>
  </Panel>
</ui>
