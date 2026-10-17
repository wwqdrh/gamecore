<ui script="explore.gd">
<!-- 秘境探索 HUD 组合根（scenes/explore/ui/）—— 只保留秘境冒险相关的必要 UI：
	 顶部：层数横幅（第 X/3 层，watch XiuDungeonState）| 离开秘境按钮（右上）
	 底部：技能栏（6 格 + 冷却）+ 灵气经验条（自 mainhud 迁移，仅探索场景显示）
	 弹窗：秘境结算 DungeonSettleModal（通关后 ExploreManager 打开，关闭即回主场景）
	 主场景的功能按钮列/任务抽屉/小地图/装备栏等均不进入秘境。
	 HUD 覆盖在游戏世界之上：根与布局层 mouse_filter=IGNORE 鼠标穿透
	 （explore.gd _ready 统一设置），交互节点（离开按钮/技能格）保持可点 -->
	<!-- 结算弹窗遮罩色（灰 40%）由 explore.gd _ready 运行期覆盖组件导出值：
		 不用 <theme> 传 overlay——builder 主题默认值在属性之后应用，一旦引入
		 <theme> 会把全场景 Label 字号盖回默认 16（属性失效的隐性副作用） -->
  <style>
	.floor-bg {
	  background: #2e2436d8;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 22;
	}
	.floor-text { color: #f2d98c; font_size: 20; }
  </style>
  <Control name="ExploreHud" anchor="full">
	<!-- 顶部行：层数横幅（居中）+ 离开按钮（右上） -->
	<MarginContainer name="TopLayer" anchor="full" margin="14 10 14 0">
	  <HBoxContainer name="TopRow" separation="16">
		<Control size_flags_horizontal="expand_fill" mouse_filter="ignore" />
		<Panel name="FloorBanner" class="floor-bg" custom_minimum_size="360,44" size_flags_vertical="shrink_begin">
		  <Label name="FloorText" class="floor-text" text="落霞秘境" anchor="full"
		         align="center" valign="center" mouse_filter="ignore" />
		</Panel>
		<Control size_flags_horizontal="expand_fill" mouse_filter="ignore" />
		<Button name="LeaveBtn" text="离开秘境 ✕" custom_minimum_size="140,44"
		        size_flags_vertical="shrink_begin" mouse_default_cursor_shape="pointing_hand"
		        @pressed="_on_leave_pressed" />
	  </HBoxContainer>
	</MarginContainer>
	<!-- 底部：技能栏 + 灵气经验条（居中；占位 Control 必须鼠标穿透） -->
	<MarginContainer name="BottomLayer" anchor="full" margin="240 0 240 12">
	  <VBoxContainer name="BottomColumn" separation="8">
		<Control size_flags_vertical="expand_fill" mouse_filter="ignore" />
		<Gml src="explore_skillbar.gml" size_flags_horizontal="shrink_center" />
	  </VBoxContainer>
	</MarginContainer>
	<!-- 秘境结算弹窗：ExploreManager.settle 通关后经 GdUIManager.find_ui 打开，
		 数据 = XiuDungeonState.settle_view（面板 watch 刷新）；
		 弹窗关闭（s_modal_closed）→ ExploreManager 回主场景 -->
	<Modal name="DungeonSettleModal" ui_id="DungeonSettleModal" anchor="full"
	       content_margin="56" close_on_overlay="true">
	  <Gml src="dungeon/dungeon_settle.gml" />
	</Modal>
  </Control>
</ui>
