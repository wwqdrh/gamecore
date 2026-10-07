<ui script="settings_panel.gd">
<!-- 游戏设置面板：ui_form.rs 表单组件 + 接口组件（bind → GdViewSetting）实战。
	 音量/全屏/垂直同步为 Rust 侧预定义接口组件（SettingSlider/SettingSwitch），
	 初值自动从 user://settings.data 读取，修改即时应用并持久化——本面板零接线；
	 画质下拉（SettingSelect custom:graphics）同样即时持久化。
	 难度单选/提示多选为基础表单组件（FormRadio/FormCheck），业务联动演示两条路：
	 @s_value_changed/@s_toggled 就近绑定本面板脚本 → GDSTATE 状态总线广播
	 运行验证：godot --headless --path . -s res://example/demo/xiuxian/check_demo_ui.gd -->
  <style>
	.window-bg {
	  background: #f0e8d2;
	  border_color: #7a6a4a;
	  border_width: 3;
	  border_radius: 16;
	}
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
	.hint-text { color: #a89878; font_size: 13; }
	.close-btn {
	  background: #3a7a5a;
	  color: #f2ead2;
	  border_radius: 20;
	  font_size: 18;
	}
	.section-text { color: #6b4a1e; font_size: 15; }
	.row-bg {
	  background: #e6dcc0;
	  border_color: #c8b890;
	  border_width: 1;
	  border_radius: 10;
	}
	.row-label { color: #4a3c26; font_size: 15; }
  </style>
  <Panel name="SettingsPanel" class="window-bg" anchor="full" margin="2%">
	<MarginContainer name="MainMargin" anchor="full" margin="14">
	  <VBoxContainer name="MainColumn" separation="8">
		<!-- 顶栏：标题 + ESC 提示 + 关闭 -->
		<Panel class="topbar-bg" custom_minimum_size="0,56">
		  <HBoxContainer anchor="full" margin="12 8 12 8">
			<Panel class="title-box" custom_minimum_size="140,40">
			  <Label class="title-text" text="设 置" anchor="full" align="center" valign="center" />
			</Panel>
			<Control size_flags_horizontal="expand_fill" />
			<Label class="hint-text" text="按 ESC 打开 / 关闭" valign="center" />
			<Control custom_minimum_size="8,0" />
			<Button name="CloseBtn" class="close-btn" text="✕" custom_minimum_size="40,40"
					@pressed="_on_close_pressed" />
		  </HBoxContainer>
		</Panel>

		<ScrollContainer size_flags_horizontal="expand_fill" size_flags_vertical="expand_fill">
		  <VBoxContainer name="SettingRows" size_flags_horizontal="expand_fill"
						 separation="10">

			<!-- ===== 音量（接口组件：bind → GdViewSetting.set_volume） ===== -->
			<Label class="section-text" text="—— 音量 ——" />
			<Panel class="row-bg" custom_minimum_size="0,56">
			  <HBoxContainer anchor="full" margin="16 8 16 8" separation="12">
				<Label class="row-label" text="主音量" custom_minimum_size="96,0" valign="center" />
				<SettingSlider name="VolMaster" bind="volume:Master"
							   min_value="0" max_value="1" step="0.01"
							   size_flags_horizontal="expand_fill" size_flags_vertical="shrink_center"
							   custom_minimum_size="300,24" />
			  </HBoxContainer>
			</Panel>
			<Panel class="row-bg" custom_minimum_size="0,56">
			  <HBoxContainer anchor="full" margin="16 8 16 8" separation="12">
				<Label class="row-label" text="音乐音量" custom_minimum_size="96,0" valign="center" />
				<SettingSlider name="VolMusic" bind="volume:Music"
							   min_value="0" max_value="1" step="0.01"
							   size_flags_horizontal="expand_fill" size_flags_vertical="shrink_center"
							   custom_minimum_size="300,24" />
			  </HBoxContainer>
			</Panel>
			<Panel class="row-bg" custom_minimum_size="0,56">
			  <HBoxContainer anchor="full" margin="16 8 16 8" separation="12">
				<Label class="row-label" text="音效音量" custom_minimum_size="96,0" valign="center" />
				<SettingSlider name="VolAudio" bind="volume:Audio"
							   min_value="0" max_value="1" step="0.01"
							   size_flags_horizontal="expand_fill" size_flags_vertical="shrink_center"
							   custom_minimum_size="300,24" />
			  </HBoxContainer>
			</Panel>

			<!-- ===== 画面（接口组件：fullscreen / vsync 走窗口管理） ===== -->
			<Label class="section-text" text="—— 画面 ——" />
			<Panel class="row-bg" custom_minimum_size="0,56">
			  <HBoxContainer anchor="full" margin="16 8 16 8" separation="12">
				<Label class="row-label" text="全屏显示" custom_minimum_size="96,0" valign="center" />
				<Control size_flags_horizontal="expand_fill" />
				<SettingSwitch name="SwFullscreen" bind="fullscreen"
							   size_flags_vertical="shrink_center" />
			  </HBoxContainer>
			</Panel>
			<Panel class="row-bg" custom_minimum_size="0,56">
			  <HBoxContainer anchor="full" margin="16 8 16 8" separation="12">
				<Label class="row-label" text="垂直同步" custom_minimum_size="96,0" valign="center" />
				<Control size_flags_horizontal="expand_fill" />
				<SettingSwitch name="SwVsync" bind="vsync" default_value="true"
							   size_flags_vertical="shrink_center" />
			  </HBoxContainer>
			</Panel>

			<!-- ===== 游戏 ===== -->
			<Label class="section-text" text="—— 游戏 ——" />
			<Panel class="row-bg" custom_minimum_size="0,56">
			  <HBoxContainer anchor="full" margin="16 8 16 8" separation="12">
				<Label class="row-label" text="画质" custom_minimum_size="96,0" valign="center" />
				<Control size_flags_horizontal="expand_fill" />
				<SettingSelect name="SelGraphics" bind="custom:graphics"
							   options="流畅,均衡,高清,影视" value="均衡"
							   size_flags_vertical="shrink_center" />
			  </HBoxContainer>
			</Panel>
			<Panel class="row-bg" custom_minimum_size="0,64">
			  <HBoxContainer anchor="full" margin="16 8 16 8" separation="12">
				<Label class="row-label" text="战斗提示" custom_minimum_size="96,0" valign="center" />
				<Control size_flags_horizontal="expand_fill" />
				<FormCheck name="ChkDamage" text="伤害数字" checked="true"
						   size_flags_vertical="shrink_center" @s_toggled="_on_damage_toggled" />
				<FormCheck name="ChkDrop" text="掉落提示" checked="true"
						   size_flags_vertical="shrink_center" @s_toggled="_on_drop_toggled" />
			  </HBoxContainer>
			</Panel>
			<Panel class="row-bg" custom_minimum_size="0,140">
			  <HBoxContainer anchor="full" margin="16 8 16 8" separation="12">
				<Label class="row-label" text="难度" custom_minimum_size="96,0" valign="top" />
				<Control size_flags_horizontal="expand_fill" />
				<FormRadio name="RadioDifficulty" options="历练,问道,渡劫" value="问道"
						   size_flags_horizontal="expand_fill" size_flags_vertical="shrink_center"
						   @s_value_changed="_on_difficulty_changed" />
			  </HBoxContainer>
			</Panel>
		  </VBoxContainer>
		</ScrollContainer>
	  </VBoxContainer>
	</MarginContainer>
  </Panel>
</ui>
