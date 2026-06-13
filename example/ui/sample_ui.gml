<ui theme="dark">
  <style>
    .main-bg {
      background: $bg_secondary;
      border_radius: 12;
      padding: 20;
    }
    .action-button {
      background: $bg_button_danger;
      color: $text_white;
      border_radius: 6;
    }
    .info-label {
      color: $text_primary;
    }
  </style>
  <VBoxContainer anchor="full" margin="2%">
    <Label text="从 .gml 文件加载的 UI" font_size="28" align="center" class="info-label" />
    <HSeparator />
    <Panel class="main-bg" size="60%,30%">
      <MarginContainer margin="all:20">
        <VBoxContainer>
          <Label text="这是一个从外部文件加载的 UI 布局" class="info-label" />
          <Label text="使用 ui_builder.parse_file() 加载" class="info-label" font_size="14" />
          <HBoxContainer margin="10 0 0 0">
            <Button text="确认" class="action-button" on_pressed="_on_confirm" />
            <Button text="取消" class="action-button" on_pressed="_on_cancel" />
          </HBoxContainer>
        </VBoxContainer>
      </MarginContainer>
    </Panel>
    <HSeparator />
    <Label text="水平列表" class="info-label" font_size="18" />
    <UIHList count="4" highlight_mode="1" highlight_color="#ffff00">
      <Button text="Slot" custom_minimum_size="10%,5%" />
    </UIHList>
    <HSeparator />
    <Label text="网格列表" class="info-label" font_size="18" />
    <UIGrid count="6" columns="3" highlight_mode="1" highlight_color="#ff8800">
      <Panel custom_minimum_size="10%,10%">
        <MarginContainer margin="5">
          <Label name="GLabel" text="Grid" />
        </MarginContainer>
      </Panel>
    </UIGrid>
  </VBoxContainer>
</ui>
