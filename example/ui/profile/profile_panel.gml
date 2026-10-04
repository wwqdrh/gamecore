<ui>
  <!-- 修仙角色信息面板：主骨架（布局 + <Gml> 组合各区块）
	   解析自设计图：顶栏（标题/境界徽章/关闭）+ 左（立绘/五行灵根）
	   + 中（境界突破）+ 右（功法列表）+ 底（战斗属性条）
	   运行验证：godot --headless --path . -s res://example/ui/profile/check_profile_ui.gd -->
  <style>
	.window-bg {
	  background: #f0e8d2;
	  border_color: #7a6a4a;
	  border_width: 3;
	  border_radius: 16;
	}
  </style>
  <Panel name="ProfilePanel" class="window-bg" anchor="full" margin="2%">
	<MarginContainer name="MainMargin" anchor="full" margin="16">
	  <VBoxContainer name="MainColumn">
		<Gml src="profile_topbar.gml" />
		<Control custom_minimum_size="0,10" />
		<HBoxContainer name="BodyRow" size_flags_vertical="expand_fill">
		  <!-- 三列比例 3:4:3（stretch_ratio 构建期静态生效，编辑器直开 tscn 即正确布局；
		       勿用 custom_minimum_size 百分比——其运行时解析仅在 GdGmlScene 链路生效，
		       直开 tscn / 非 GdGmlScene 场景下会解析为 0 宽） -->
		  <Gml src="profile_avatar.gml" size_flags_horizontal="expand_fill" stretch_ratio="3" />
		  <Control custom_minimum_size="10,0" />
		  <Gml src="profile_realm.gml" size_flags_horizontal="expand_fill" stretch_ratio="4" />
		  <Control custom_minimum_size="10,0" />
		  <Gml src="profile_skills.gml" size_flags_horizontal="expand_fill" stretch_ratio="3" />
		</HBoxContainer>
		<Control custom_minimum_size="0,10" />
		<Gml src="profile_stats.gml" />
	  </VBoxContainer>
	</MarginContainer>
  </Panel>
</ui>
