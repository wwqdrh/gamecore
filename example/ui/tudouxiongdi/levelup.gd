# 土豆兄弟升级弹窗界面 - 2D像素风生存割草游戏升级选择界面
# 纯深灰哑光背景，扁平化极简黑色卡片UI，低饱和度暗色调
# 严格匹配 levelup.html 设计稿
extends GdGmlScene

var UI = """
<ui>
    <style>
        .hp-bar { background: #c91919; track: #551111; border_radius: 0; }
        .exp-bar { background: #24c442; track: #114411; border_radius: 0; }
        .coin-icon { background: #48d05e; border_radius: 20; }
        .talent-card { background: #000000; border_radius: 6; padding: 20 12; }
        .card-icon { background: #eeeeee; border_radius: 4; }
        .card-icon-heart { background: #48d05e; border_radius: 4; }
        .select-btn { background: #ffffff; border_radius: 0; border_width: 0; color: #000000; padding: 6 24; }
        .refresh-btn { background: #000000; border_radius: 6; border_width: 0; padding: 14 40; color: #ffffff; }
        .attr-panel { background: #000000; border_radius: 6; padding: 20 16; }
        .attr-icon { background: #666666; border_radius: 3; }
    </style>

    <!-- 根容器：全屏深灰背景 -->
	<Control anchor="full" margin="20">

        <!-- ====== 左上角状态栏 ====== -->
		<VBoxContainer name="StatusLeft" custom_minimum_size="260,0" v_separation="8">
            <!-- HP 血条 -->
			<Control custom_minimum_size="260,40">
				<ProgressBar anchor="full" class="hp-bar" value="80" max_value="80" show_percentage="false" />
				<Label anchor="full" text="80/80" align="right" valign="center" font_size="22" margin="0 0 12 0" font_color="#ffffff" />
            </Control>
            <!-- EXP 经验条 -->
			<Control custom_minimum_size="260,32">
				<ProgressBar anchor="full" class="exp-bar" value="117" max_value="515" show_percentage="false" />
				<Label anchor="full" text="117/515" align="right" valign="center" font_size="18" margin="0 0 12 0" font_color="#ffffff" />
            </Control>
            <!-- 金币 -->
			<HBoxContainer name="CoinLine" h_separation="10" custom_minimum_size="0,40">
				<ColorRect custom_minimum_size="40,40" class="coin-icon" />
				<Label text="388" font_size="32" valign="center" font_color="#ffffff" />
            </HBoxContainer>
        </VBoxContainer>

        <!-- ====== 右侧属性面板 (绝对定位) ====== -->
		<PanelContainer name="AttrPanel" class="attr-panel"
			anchor_left="1.0" anchor_right="1.0" anchor_top="0.0" anchor_bottom="1.0"
			offset_left="-240" offset_right="0" offset_top="0" offset_bottom="0"
			custom_minimum_size="240,0">
			<VBoxContainer name="AttrList" v_separation="2">
				<Label text="剩余升级点数: 1" align="right" font_size="22" font_color="#ffffff" />
				<Label text="目前等级        16" align="right" font_size="22" font_color="#ffffff" />

                <HSeparator />

                <HBoxContainer>
					<HBoxContainer size_flags_horizontal="expand" h_separation="8">
						<ColorRect custom_minimum_size="22,22" class="attr-icon" />
						<Label text="生命上限" font_size="18" font_color="#ffffff" />
                    </HBoxContainer>
					<Label text="80" font_size="18" font_color="#ffffff" />
                </HBoxContainer>
                <HBoxContainer>
					<HBoxContainer size_flags_horizontal="expand" h_separation="8">
						<ColorRect custom_minimum_size="22,22" class="attr-icon" />
						<Label text="伤害加成" font_size="18" font_color="#ffffff" />
                    </HBoxContainer>
					<Label text="13" font_size="18" font_color="#ffffff" />
                </HBoxContainer>
                <HBoxContainer>
					<HBoxContainer size_flags_horizontal="expand" h_separation="8">
						<ColorRect custom_minimum_size="22,22" class="attr-icon" />
						<Label text="移动速度" font_size="18" font_color="#ffffff" />
                    </HBoxContainer>
					<Label text="220" font_size="18" font_color="#ffffff" />
                </HBoxContainer>
                <HBoxContainer>
					<HBoxContainer size_flags_horizontal="expand" h_separation="8">
						<ColorRect custom_minimum_size="22,22" class="attr-icon" />
						<Label text="攻速加成" font_size="18" font_color="#ffffff" />
                    </HBoxContainer>
					<Label text="5" font_size="18" font_color="#ffffff" />
                </HBoxContainer>
                <HBoxContainer>
					<HBoxContainer size_flags_horizontal="expand" h_separation="8">
						<ColorRect custom_minimum_size="22,22" class="attr-icon" />
						<Label text="暴击率" font_size="18" font_color="#ffffff" />
                    </HBoxContainer>
					<Label text="4" font_size="18" font_color="#ffffff" />
                </HBoxContainer>
                <HBoxContainer>
					<HBoxContainer size_flags_horizontal="expand" h_separation="8">
						<ColorRect custom_minimum_size="22,22" class="attr-icon" />
						<Label text="闪避率" font_size="18" font_color="#ffffff" />
                    </HBoxContainer>
					<Label text="2" font_size="18" font_color="#ffffff" />
                </HBoxContainer>
                <HBoxContainer>
					<HBoxContainer size_flags_horizontal="expand" h_separation="8">
						<ColorRect custom_minimum_size="22,22" class="attr-icon" />
						<Label text="生命吸取" font_size="18" font_color="#ffffff" />
                    </HBoxContainer>
					<Label text="6" font_size="18" font_color="#ffffff" />
                </HBoxContainer>
                <HBoxContainer>
					<HBoxContainer size_flags_horizontal="expand" h_separation="8">
						<ColorRect custom_minimum_size="22,22" class="attr-icon" />
						<Label text="工程学" font_size="18" font_color="#ffffff" />
                    </HBoxContainer>
					<Label text="0" font_size="18" font_color="#ffffff" />
                </HBoxContainer>
                <HBoxContainer>
					<HBoxContainer size_flags_horizontal="expand" h_separation="8">
						<ColorRect custom_minimum_size="22,22" class="attr-icon" />
						<Label text="经济学" font_size="18" font_color="#ffffff" />
                    </HBoxContainer>
					<Label text="0" font_size="18" font_color="#ffffff" />
                </HBoxContainer>
                <HBoxContainer>
					<HBoxContainer size_flags_horizontal="expand" h_separation="8">
						<ColorRect custom_minimum_size="22,22" class="attr-icon" />
						<Label text="幸运值" font_size="18" font_color="#ffffff" />
                    </HBoxContainer>
					<Label text="0" font_size="18" font_color="#ffffff" />
                </HBoxContainer>
                <HBoxContainer>
					<HBoxContainer size_flags_horizontal="expand" h_separation="8">
						<ColorRect custom_minimum_size="22,22" class="attr-icon" />
						<Label text="经验加成" font_size="18" font_color="#ffffff" />
                    </HBoxContainer>
					<Label text="0" font_size="18" font_color="#ffffff" />
                </HBoxContainer>
            </VBoxContainer>
        </PanelContainer>

        <!-- ====== 中间主内容区域 (标题 + 天赋卡片 + 刷新按钮) ====== -->
		<VBoxContainer name="CenterContent"
			anchor_left="0.0" anchor_right="1.0" anchor_top="0.0" anchor_bottom="1.0"
			offset_left="260" offset_right="-240" offset_top="0" offset_bottom="0">

            <!-- 顶部中间标题 -->
            <CenterContainer>
				<VBoxContainer v_separation="8">
					<Label text="第16关" align="center" font_size="42" font_color="#ffffff" />
					<Label text="升级!!!" align="center" font_size="60" font_color="#ffffff" />
                </VBoxContainer>
            </CenterContainer>

            <!-- 中间天赋三卡片 -->
			<CenterContainer size_flags_vertical="expand">
				<HBoxContainer h_separation="30">
                    <!-- 天赋1 智慧 -->
					<PanelContainer class="talent-card" custom_minimum_size="240,0">
                        <VBoxContainer>
                            <CenterContainer>
								<ColorRect custom_minimum_size="60,60" class="card-icon" />
                            </CenterContainer>
							<Label text="智慧" align="center" font_size="24" font_color="#ffffff" />
							<Label text="经验获得 +5%" align="center" font_size="18" font_color="#48d05e" />
                            <CenterContainer>
								<Button text="选择" class="select-btn" font_size="18" />
                            </CenterContainer>
                        </VBoxContainer>
                    </PanelContainer>
                    <!-- 天赋2 最大生命 -->
					<PanelContainer class="talent-card" custom_minimum_size="240,0">
                        <VBoxContainer>
                            <CenterContainer>
								<ColorRect custom_minimum_size="60,60" class="card-icon-heart" />
                            </CenterContainer>
							<Label text="最大生命" align="center" font_size="24" font_color="#ffffff" />
							<Label text="最大生命 +6" align="center" font_size="18" font_color="#48d05e" />
                            <CenterContainer>
								<Button text="选择" class="select-btn" font_size="18" />
                            </CenterContainer>
                        </VBoxContainer>
                    </PanelContainer>
                    <!-- 天赋3 智慧 -->
					<PanelContainer class="talent-card" custom_minimum_size="240,0">
                        <VBoxContainer>
                            <CenterContainer>
								<ColorRect custom_minimum_size="60,60" class="card-icon" />
                            </CenterContainer>
							<Label text="智慧" align="center" font_size="24" font_color="#ffffff" />
							<Label text="经验获得 +5%" align="center" font_size="18" font_color="#48d05e" />
                            <CenterContainer>
								<Button text="选择" class="select-btn" font_size="18" />
                            </CenterContainer>
                        </VBoxContainer>
                    </PanelContainer>
                </HBoxContainer>
            </CenterContainer>

            <!-- 刷新按钮 -->
            <CenterContainer>
				<Button name="RefreshBtn" class="refresh-btn" font_size="26" text="刷新" />
            </CenterContainer>

        </VBoxContainer>

    </Control>
</ui>
"""

func _ready() -> void:
	load_from_string(UI)
