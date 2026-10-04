<ui script="mainhud.gd">
<!-- 修仙主界面 HUD：主骨架（三层透明布局层 + <Gml> 组合各区块）
	 解析自设计图：
	 · 顶层行：玩家徽章(头像+境界) | 资源栏(灵石/丹药+加号) | 小地图
	 · 中层行：任务卷轴(左) | 功能按钮列(修炼/功法/储物袋/坊市，右)
	 · 底层列：技能栏(6 格 + 冷却) + 灵气经验条
	 HUD 覆盖在游戏世界之上：根与布局层 mouse_filter=IGNORE 鼠标穿透
	 （mainhud.gd _ready 统一设置），交互区块（功能按钮/技能格）保持可点。
	 跨区块联动 = GdState 临时状态总线（单例 GDSTATE，非持久化）：
	 功能按钮/技能格点击 set_state 上报，监听方 watch 各自更新
	 运行验证：godot --headless --path . -s res://example/ui/mainhud/check_mainhud_ui.gd -->
  <Control name="MainHud" anchor="full">
	<!-- 顶层行：左头像徽章 / 中资源栏 / 右小地图（shrink_begin 防止被最高子项拉伸） -->
	<MarginContainer name="TopLayer" anchor="full" margin="14 10 14 0">
	  <HBoxContainer name="TopRow" separation="16">
		<Gml src="mainhud_player.gml" size_flags_vertical="shrink_begin" />
		<Control size_flags_horizontal="expand_fill" />
		<Gml src="mainhud_resources.gml" size_flags_vertical="shrink_begin" />
		<Control size_flags_horizontal="expand_fill" />
		<Gml src="mainhud_minimap.gml" size_flags_vertical="shrink_begin" />
	  </HBoxContainer>
	</MarginContainer>
	<!-- 中层行：任务卷轴（左） | 功能按钮列（右，整体下移避开小地图） -->
	<MarginContainer name="MidLayer" anchor="full" margin="14 175 14 0">
	  <HBoxContainer name="MidRow" separation="16">
		<Gml src="mainhud_quest.gml" size_flags_vertical="shrink_begin" />
		<Control size_flags_horizontal="expand_fill" />
		<Gml src="mainhud_menus.gml" size_flags_vertical="shrink_begin" />
	  </HBoxContainer>
	</MarginContainer>
	<!-- 底层列：技能栏 + 经验条水平居中贴底（左右收窄 240） -->
	<MarginContainer name="BottomLayer" anchor="full" margin="240 0 240 12">
	  <VBoxContainer name="BottomColumn" separation="8">
		<Control size_flags_vertical="expand_fill" />
		<Gml src="mainhud_skillbar.gml" size_flags_horizontal="shrink_center" />
	  </VBoxContainer>
	</MarginContainer>
  </Control>
</ui>
