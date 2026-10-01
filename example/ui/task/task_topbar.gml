<ui theme="cartoon">
  <!-- 顶部资源栏：货币显示 + 右上角关闭按钮
       两侧 expand 占位使货币面板保持居中 -->
  <style>
    .currency-bar {
      background: #24352a;
      border_color: #b8a06a;
      border_width: 2;
      border_radius: 22;
    }
    .cur-icon { }
    .cur-name { color: #cdbf9a; }
    .cur-num { color: #ffffff; }
    .close-btn {
      background: #a83a2e;
      color: #ffffff;
      border_color: #7a2820;
      border_width: 2;
      border_radius: 22;
    }
  </style>
  <HBoxContainer name="TopBar" anchor="full">
    <Control size_flags_horizontal="expand_fill" />
    <Panel class="currency-bar" size_flags_vertical="shrink_center" custom_minimum_size="380,52">
      <MarginContainer anchor="full" margin="10 8 10 8">
        <HBoxContainer>
          <Label text="💎" class="cur-icon" font_size="20" valign="center" size_flags_vertical="shrink_center" />
          <Control custom_minimum_size="6,0" />
          <Label text="灵石" class="cur-name" font_size="16" valign="center" size_flags_vertical="shrink_center" />
          <Control custom_minimum_size="8,0" />
          <Label name="LingShiNum" text="1,240" class="cur-num" font_size="17" valign="center" size_flags_vertical="shrink_center" />
          <Control custom_minimum_size="28,0" />
          <Label text="🪙" class="cur-icon" font_size="20" valign="center" size_flags_vertical="shrink_center" />
          <Control custom_minimum_size="6,0" />
          <Label text="宗门贡献" class="cur-name" font_size="16" valign="center" size_flags_vertical="shrink_center" />
          <Control custom_minimum_size="8,0" />
          <Label name="GongXianNum" text="1,240" class="cur-num" font_size="17" valign="center" size_flags_vertical="shrink_center" />
        </HBoxContainer>
      </MarginContainer>
    </Panel>
    <Control size_flags_horizontal="expand_fill" />
    <Button name="CloseBtn" text="✕" class="close-btn" custom_minimum_size="44,44" size_flags_vertical="shrink_center" on_pressed="_on_close_pressed" mouse_default_cursor_shape="pointing_hand" />
  </HBoxContainer>
</ui>
