<ui>
  <!-- 五行灵根条目模板：作为 profile_avatar.gml 中 UIVList 的 slot 模板
	   {{key}} 模板绑定渲染数据（value 同时绑到 RootValue 文本与 RootBar 数值）；
	   进度条填充色统一玉青色，五行差异由 icon emoji 与名称区分
	   （后期可加条目脚本按 element 字段动态上色） -->
  <style>
	.root-name { color: #3c2f1c; }
	.root-value { color: #6b5a3e; }
	.root-bar {
	  background: #5e9c86;
	  track: #e2d8bc;
	  border_radius: 6;
	}
  </style>
  <Panel name="RootItem" custom_minimum_size="0,26">
	<HBoxContainer name="RootRow" anchor="full">
	  <Label name="RootIcon" text="{{icon}}" font_size="14" valign="center" />
	  <Label name="RootName" text="{{element}}" class="root-name" font_size="15" valign="center" />
	  <Control custom_minimum_size="6,0" />
	  <ProgressBar name="RootBar" value="{{value}}" max_value="{{max}}" class="root-bar"
	               custom_minimum_size="0,12" percent_visible="false"
	               size_flags_horizontal="expand_fill" size_flags_vertical="shrink_center" />
	  <Control custom_minimum_size="8,0" />
	  <Label name="RootValue" text="{{value}}" class="root-value" font_size="15" valign="center" />
	</HBoxContainer>
  </Panel>
</ui>
