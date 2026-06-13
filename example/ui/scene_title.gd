# 游戏标题界面 GML 控制器 - 继承 GdGmlScene，处理 GML 中的事件回调
# PopupPanel 的显示/隐藏由 GML 内部信号绑定自动处理
# 此脚本仅处理游戏逻辑回调
extends GdGmlScene

var UI = """
<ui theme="dark">
  <style>
    .title-text {
      color: $text_white;
    }
    .subtitle-text {
      color: $text_secondary;
    }
    .menu-button {
      texture: res://example/ui/assets/btn_green.png;
      color: $text_white;
      padding: 12;
    }
    .settings-btn {
      background: $bg_button;
      color: $text_primary;
      border_radius: 4;
      padding: 8;
    }
    .section-title {
      color: $text_title;
    }
    .row-bg {
      background: $bg_secondary;
      border_radius: 4;
      padding: 10;
    }
    .label-text {
      color: $text_primary;
    }
    .value-text {
      color: $text_accent;
    }
    .apply-btn {
      background: $bg_button_primary;
      color: $text_white;
      border_radius: 6;
      padding: 10;
    }
    .cancel-btn {
      background: $bg_button_danger;
      color: $text_white;
      border_radius: 6;
      padding: 10;
    }
  </style>
  <VBoxContainer anchor="full">
    <!-- 右上角设置按钮 -->
    <HBoxContainer>
	  <Control size_flags_horizontal="expand_fill" />
	  <Button name="SettingsBtn" text="Settings" class="settings-btn" on_pressed="show:SettingsPopup" mouse_default_cursor_shape="pointing_hand" />
    </HBoxContainer>

    <!-- 居中标题区域 -->
    <CenterContainer>
      <VBoxContainer>
		<Label text="GAME TITLE" font_size="48" align="center" class="title-text" />
		<Label text="A Game Made With GameCore" font_size="16" align="center" class="subtitle-text" />
      </VBoxContainer>
    </CenterContainer>

    <!-- 居中按钮组 -->
    <CenterContainer>
	  <VBoxContainer v_separation="12">
		<TextureButton name="StartBtn" text="Start Game" class="menu-button" custom_minimum_size="30%,6%" on_pressed="_on_start_game" mouse_default_cursor_shape="pointing_hand" />
		<TextureButton name="ContinueBtn" text="Continue" class="menu-button" custom_minimum_size="30%,6%" on_pressed="_on_continue_game" mouse_default_cursor_shape="pointing_hand" />
		<TextureButton name="QuitBtn" text="Quit" class="menu-button" custom_minimum_size="30%,6%" on_pressed="_on_quit_game" mouse_default_cursor_shape="pointing_hand" />
      </VBoxContainer>
    </CenterContainer>
  </VBoxContainer>

  <!-- 设置弹窗（PopupPanel 节点，初始隐藏） -->
  <PopupPanel name="SettingsPopup" popup_title="Settings" width="50%" height="60%" close_on_overlay="true">
	<ScrollContainer size_flags_vertical="expand_fill" horizontal="disabled" vertical="auto">
	  <VBoxContainer v_separation="8">
        <!-- 音频设置 -->
		<Label text="Audio" font_size="18" class="section-title" />
		<HBoxContainer class="row-bg">
		  <Label text="Master Volume" class="label-text" size_flags_horizontal="expand_fill" />
		  <Label name="MasterVolLabel" text="80" class="value-text" />
		  <HSlider name="MasterVolSlider" min_value="0" max_value="100" value="80" step="1" custom_minimum_size="15%,0" />
        </HBoxContainer>
		<HBoxContainer class="row-bg">
		  <Label text="Music Volume" class="label-text" size_flags_horizontal="expand_fill" />
		  <Label name="MusicVolLabel" text="70" class="value-text" />
		  <HSlider name="MusicVolSlider" min_value="0" max_value="100" value="70" step="1" custom_minimum_size="15%,0" />
        </HBoxContainer>
		<HBoxContainer class="row-bg">
		  <Label text="SFX Volume" class="label-text" size_flags_horizontal="expand_fill" />
		  <Label name="SfxVolLabel" text="90" class="value-text" />
		  <HSlider name="SfxVolSlider" min_value="0" max_value="100" value="90" step="1" custom_minimum_size="15%,0" />
        </HBoxContainer>

        <HSeparator />

        <!-- 显示设置 -->
		<Label text="Display" font_size="18" class="section-title" />
		<HBoxContainer class="row-bg">
		  <Label text="Fullscreen" class="label-text" size_flags_horizontal="expand_fill" />
		  <CheckButton name="FullscreenCheck" text="Fullscreen" on_pressed="_on_fullscreen_toggle" />
        </HBoxContainer>

        <HSeparator />

        <!-- 语言设置 -->
		<Label text="Language" font_size="18" class="section-title" />
		<HBoxContainer class="row-bg">
		  <Label text="Language" class="label-text" size_flags_horizontal="expand_fill" />
		  <OptionButton name="LangOption" items="中文,English,日本語" selected="0" />
        </HBoxContainer>

        <HSeparator />

        <!-- 操作按钮 -->
		<HBoxContainer margin="0 8 0 0">
		  <Control size_flags_horizontal="expand_fill" />
		  <Button text="Apply" class="apply-btn" custom_minimum_size="12%,5%" on_pressed="hide:SettingsPopup" />
		  <Button text="Cancel" class="cancel-btn" custom_minimum_size="12%,5%" on_pressed="hide:SettingsPopup" />
        </HBoxContainer>
      </VBoxContainer>
    </ScrollContainer>
  </PopupPanel>
</ui>
"""

func _ready() -> void:
	load_from_string(UI)
	
func _on_start_game() -> void:
	print("Start Game clicked")


func _on_continue_game() -> void:
	print("Continue Game clicked")


func _on_quit_game() -> void:
	get_tree().quit()


func _on_fullscreen_toggle() -> void:
	var check = find_node("FullscreenCheck")
	if check:
		get_window().mode = Window.MODE_FULLSCREEN if check.button_pressed else Window.MODE_WINDOWED
