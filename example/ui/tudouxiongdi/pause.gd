# 土豆兄弟暂停/结算面板 - 深色游戏UI
# 严格匹配 pause.html 设计稿
extends GdGmlScene

var UI = """
<ui>
    <style>
        .panel-bg { background: #222222; }
        .attr-box { background: #111111; padding: 20 15; }
        .weapon-red { background: #702020; border_color: #a83232; border_width: 2; border_radius: 6; }
        .weapon-purple { background: #402060; border_color: #7838b8; border_width: 2; border_radius: 6; }
        .item-black { background: #1a1a1a; border_color: #444444; border_width: 2; border_radius: 4; }
        .item-purple { background: #2c1840; border_color: #7038b0; border_width: 2; border_radius: 4; }
        .item-blue { background: #152840; border_color: #3078c0; border_width: 2; border_radius: 4; }
        .item-gray { background: #333333; border_color: #777777; border_width: 2; border_radius: 4; }
        .btn-dark { background: #111111; color: #ffffff; border_radius: 0; border_width: 0; padding: 0; }
        .btn-light { background: #dddddd; color: #111111; border_radius: 0; border_width: 0; padding: 0; }
    </style>

    <!-- 根容器：全屏深色背景 -->
    <Control anchor="full" class="panel-bg">
        <VBoxContainer anchor="full" margin="20 40 20 20" v_separation="30">

            <!-- ====== 顶部标题 ====== -->
            <CenterContainer>
                <HBoxContainer h_separation="40">
                    <Label text="胜利" font_size="36" font_color="#ffffff" />
                    <Label text="危险5" font_size="36" font_color="#ffffff" />
                </HBoxContainer>
            </CenterContainer>

            <!-- ====== 主内容区域 ====== -->
            <HBoxContainer size_flags_vertical="expand_fill" h_separation="20">

                <!-- 左侧属性面板 -->
                <PanelContainer class="attr-box" custom_minimum_size="260,0" size_flags_vertical="expand_fill">
                    <VBoxContainer v_separation="12">
                        <Label text="属性" font_size="32" align="center" font_color="#ffffff" />

                        <HBoxContainer>
                            <Label text="💚 最大生命值" font_size="18" font_color="#ffffff" size_flags_horizontal="expand" />
                            <Label text="77" font_size="18" font_color="#4cd964" />
                        </HBoxContainer>
                        <HBoxContainer>
                            <Label text="💚 生命再生" font_size="18" font_color="#ffffff" size_flags_horizontal="expand" />
                            <Label text="15" font_size="18" font_color="#4cd964" />
                        </HBoxContainer>
                        <HBoxContainer>
                            <Label text="❤️ %生命窃取" font_size="18" font_color="#ffffff" size_flags_horizontal="expand" />
                            <Label text="6" font_size="18" font_color="#4cd964" />
                        </HBoxContainer>
                        <HBoxContainer>
                            <Label text="👊 %伤害" font_size="18" font_color="#ffffff" size_flags_horizontal="expand" />
                            <Label text="85" font_size="18" font_color="#4cd964" />
                        </HBoxContainer>
                        <HBoxContainer>
                            <Label text="⚔️ 近战伤害" font_size="18" font_color="#ffffff" size_flags_horizontal="expand" />
                            <Label text="23" font_size="18" font_color="#4cd964" />
                        </HBoxContainer>
                        <HBoxContainer>
                            <Label text="🏹 远程伤害" font_size="18" font_color="#ffffff" size_flags_horizontal="expand" />
                            <Label text="-49" font_size="18" font_color="#ff3b30" />
                        </HBoxContainer>
                        <HBoxContainer>
                            <Label text="🔥 属性伤害" font_size="18" font_color="#ffffff" size_flags_horizontal="expand" />
                            <Label text="-5" font_size="18" font_color="#ff3b30" />
                        </HBoxContainer>
                        <HBoxContainer>
                            <Label text="⏱️ %攻击速度" font_size="18" font_color="#ffffff" size_flags_horizontal="expand" />
                            <Label text="-11" font_size="18" font_color="#ff3b30" />
                        </HBoxContainer>
                        <HBoxContainer>
                            <Label text="⭐ %暴击率" font_size="18" font_color="#ffffff" size_flags_horizontal="expand" />
                            <Label text="-8" font_size="18" font_color="#ff3b30" />
                        </HBoxContainer>
                        <HBoxContainer>
                            <Label text="🔧 工程学" font_size="18" font_color="#ffffff" size_flags_horizontal="expand" />
                            <Label text="-2" font_size="18" font_color="#ff3b30" />
                        </HBoxContainer>
                        <HBoxContainer>
                            <Label text="📡 范围" font_size="18" font_color="#ffffff" size_flags_horizontal="expand" />
                            <Label text="-62" font_size="18" font_color="#ff3b30" />
                        </HBoxContainer>
                        <HBoxContainer>
                            <Label text="🛡️ 护甲" font_size="18" font_color="#ffffff" size_flags_horizontal="expand" />
                            <Label text="32" font_size="18" font_color="#4cd964" />
                        </HBoxContainer>
                        <HBoxContainer>
                            <Label text="💨 %闪避" font_size="18" font_color="#ffffff" size_flags_horizontal="expand" />
                            <Label text="55" font_size="18" font_color="#4cd964" />
                        </HBoxContainer>
                        <HBoxContainer>
                            <Label text="👟 %速度" font_size="18" font_color="#ffffff" size_flags_horizontal="expand" />
                            <Label text="52" font_size="18" font_color="#4cd964" />
                        </HBoxContainer>
                        <HBoxContainer>
                            <Label text="🍀 幸运" font_size="18" font_color="#ffffff" size_flags_horizontal="expand" />
                            <Label text="26" font_size="18" font_color="#4cd964" />
                        </HBoxContainer>
                        <HBoxContainer>
                            <Label text="💰 收获" font_size="18" font_color="#ffffff" size_flags_horizontal="expand" />
                            <Label text="92" font_size="18" font_color="#4cd964" />
                        </HBoxContainer>
                    </VBoxContainer>
                </PanelContainer>

                <!-- 右侧武器+道具区域 -->
                <VBoxContainer size_flags_horizontal="expand" v_separation="30">

                    <!-- 武器区域 -->
                    <VBoxContainer v_separation="15">
                        <Label text="武器" font_size="32" font_color="#ffffff" />
                        <HBoxContainer h_separation="12">
                            <Panel class="weapon-red" custom_minimum_size="80,80">
                                <Label text="👊" anchor="full" align="center" valign="center" font_size="32" />
                            </Panel>
                            <Panel class="weapon-red" custom_minimum_size="80,80">
                                <Label text="👊" anchor="full" align="center" valign="center" font_size="32" />
                            </Panel>
                            <Panel class="weapon-purple" custom_minimum_size="80,80">
                                <Label text="👊" anchor="full" align="center" valign="center" font_size="32" />
                            </Panel>
                            <Panel class="weapon-red" custom_minimum_size="80,80">
                                <Label text="👊" anchor="full" align="center" valign="center" font_size="32" />
                            </Panel>
                            <Panel class="weapon-purple" custom_minimum_size="80,80">
                                <Label text="👊" anchor="full" align="center" valign="center" font_size="32" />
                            </Panel>
                            <Panel class="weapon-red" custom_minimum_size="80,80">
                                <Label text="👊" anchor="full" align="center" valign="center" font_size="32" />
                            </Panel>
                        </HBoxContainer>
                    </VBoxContainer>

                    <!-- 道具区域 -->
                    <VBoxContainer v_separation="15">
                        <Label text="道具" font_size="32" font_color="#ffffff" />
                        <VBoxContainer v_separation="10">
                            <!-- 第一行道具 -->
                            <HBoxContainer h_separation="12">
                                <Panel class="item-black" custom_minimum_size="72,72">
                                    <Label text="👕" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-purple" custom_minimum_size="72,72">
                                    <Label text="▶️" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-purple" custom_minimum_size="72,72">
                                    <Label text="👾" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-purple" custom_minimum_size="72,72">
                                    <Label text="🐗" anchor="full" align="center" valign="center" font_size="28" />
                                    <Label text="X2" font_size="16" font_color="#ffffff" anchor_left="0.5" anchor_right="1.0" anchor_top="0.5" anchor_bottom="1.0" offset_right="-4" offset_bottom="-2" align="right" valign="bottom" />
                                </Panel>
                                <Panel class="item-purple" custom_minimum_size="72,72">
                                    <Label text="🔫" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-purple" custom_minimum_size="72,72">
                                    <Label text="🔺" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-purple" custom_minimum_size="72,72">
                                    <Label text="🌾" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-blue" custom_minimum_size="72,72">
                                    <Label text="💍" anchor="full" align="center" valign="center" font_size="28" />
                                    <Label text="X2" font_size="16" font_color="#ffffff" anchor_left="0.5" anchor_right="1.0" anchor_top="0.5" anchor_bottom="1.0" offset_right="-4" offset_bottom="-2" align="right" valign="bottom" />
                                </Panel>
                                <Panel class="item-black" custom_minimum_size="72,72">
                                    <Label text="🪽" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-black" custom_minimum_size="72,72">
                                    <Label text="🎩" anchor="full" align="center" valign="center" font_size="28" />
                                    <Label text="X2" font_size="16" font_color="#ffffff" anchor_left="0.5" anchor_right="1.0" anchor_top="0.5" anchor_bottom="1.0" offset_right="-4" offset_bottom="-2" align="right" valign="bottom" />
                                </Panel>
                            </HBoxContainer>

                            <!-- 第二行道具 -->
                            <HBoxContainer h_separation="12">
                                <Panel class="item-black" custom_minimum_size="72,72">
                                    <Label text="🌱" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-black" custom_minimum_size="72,72">
                                    <Label text="🎩" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-purple" custom_minimum_size="72,72">
                                    <Label text="🧙‍♂️" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-purple" custom_minimum_size="72,72">
                                    <Label text="🐒" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-blue" custom_minimum_size="72,72">
                                    <Label text="🦑" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-blue" custom_minimum_size="72,72">
                                    <Label text="👞" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-blue" custom_minimum_size="72,72">
                                    <Label text="🦴" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-blue" custom_minimum_size="72,72">
                                    <Label text="🥾" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-black" custom_minimum_size="72,72">
                                    <Label text="🍅" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-black" custom_minimum_size="72,72">
                                    <Label text="🌙" anchor="full" align="center" valign="center" font_size="28" />
                                    <Label text="X3" font_size="16" font_color="#ffffff" anchor_left="0.5" anchor_right="1.0" anchor_top="0.5" anchor_bottom="1.0" offset_right="-4" offset_bottom="-2" align="right" valign="bottom" />
                                </Panel>
                            </HBoxContainer>

                            <!-- 第三行道具 -->
                            <HBoxContainer h_separation="12">
                                <Panel class="item-blue" custom_minimum_size="72,72">
                                    <Label text="🥋" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-blue" custom_minimum_size="72,72">
                                    <Label text="📜" anchor="full" align="center" valign="center" font_size="28" />
                                    <Label text="X3" font_size="16" font_color="#ffffff" anchor_left="0.5" anchor_right="1.0" anchor_top="0.5" anchor_bottom="1.0" offset_right="-4" offset_bottom="-2" align="right" valign="bottom" />
                                </Panel>
                                <Panel class="item-purple" custom_minimum_size="72,72">
                                    <Label text="🦋" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-purple" custom_minimum_size="72,72">
                                    <Label text="🦎" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-purple" custom_minimum_size="72,72">
                                    <Label text="👁️" anchor="full" align="center" valign="center" font_size="28" />
                                    <Label text="X2" font_size="16" font_color="#ffffff" anchor_left="0.5" anchor_right="1.0" anchor_top="0.5" anchor_bottom="1.0" offset_right="-4" offset_bottom="-2" align="right" valign="bottom" />
                                </Panel>
                                <Panel class="item-purple" custom_minimum_size="72,72">
                                    <Label text="🗡️" anchor="full" align="center" valign="center" font_size="28" />
                                    <Label text="X2" font_size="16" font_color="#ffffff" anchor_left="0.5" anchor_right="1.0" anchor_top="0.5" anchor_bottom="1.0" offset_right="-4" offset_bottom="-2" align="right" valign="bottom" />
                                </Panel>
                                <Panel class="item-blue" custom_minimum_size="72,72">
                                    <Label text="🥤" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-black" custom_minimum_size="72,72">
                                    <Label text="💀" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                                <Panel class="item-gray" custom_minimum_size="72,72">
                                    <Label text="🥚" anchor="full" align="center" valign="center" font_size="28" />
                                </Panel>
                            </HBoxContainer>
                        </VBoxContainer>
                    </VBoxContainer>

                </VBoxContainer>

            </HBoxContainer>

            <!-- ====== 底部按钮 ====== -->
            <CenterContainer>
                <HBoxContainer h_separation="10">
                    <Button text="重试" class="btn-dark" custom_minimum_size="320,60" font_size="26" />
                    <Button text="新游戏" class="btn-light" custom_minimum_size="320,60" font_size="26" />
                    <Button text="返回主菜单" class="btn-dark" custom_minimum_size="320,60" font_size="26" />
                </HBoxContainer>
            </CenterContainer>

        </VBoxContainer>
    </Control>
</ui>
"""

func _ready() -> void:
	load_from_string(UI)
