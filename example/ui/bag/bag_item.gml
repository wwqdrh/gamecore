<ui script="bag_item.gd">
  <!-- 背包格子条目模板：作为 bag_grid.gml 中 UIGrid 的 slot 模板（构建期注入）。
	   数据契约见 bag_item.gd 的 @export 列表：
	   icon / count(数量角标，空串隐藏) / quality("稀有" 紫底，空串普通)
	   / locked(锁定格：置灰+锁标) / selected(选中态：金边框)
	   {{key}} 模板绑定负责静态文本渲染，契约注入负责动态状态联动 -->
  <style>
	.cell-bg {
	  background: #dfe9cc;
	  border_color: #a8b98a;
	  border_width: 1;
	  border_radius: 6;
	}
	.rare-bg {
	  background: #b096e0;
	  border_color: #8a6fc0;
	  border_width: 1;
	  border_radius: 6;
	}
	.sel-frame {
	  background: #f0e8d200;
	  border_color: #e8b93e;
	  border_width: 3;
	  border_radius: 6;
	}
	.count-text { color: #4a3a22; font_size: 11; }
	.cell-icon { font_size: 30; }
	.lock-icon { font_size: 26; }
  </style>
  <!-- 根节点双向 expand_fill：格子拉伸填满 UIGrid 列宽/行高 -->
  <Panel name="BagCell" class="cell-bg" custom_minimum_size="58,58"
         size_flags_horizontal="expand_fill" size_flags_vertical="expand_fill">
	<!-- 稀有品质紫底（quality 契约联动显示） -->
	<Panel name="RareFrame" class="rare-bg" anchor="full" visible="false" />
	<!-- 物品图标 -->
	<CenterContainer name="IconBox" anchor="full" margin="5">
	  <Label name="CellIcon" class="cell-icon" text="{{icon}}" />
	</CenterContainer>
	<!-- 锁定图标（locked 契约联动显示，配合整格置灰） -->
	<Label name="LockIcon" class="lock-icon" text="🔒" anchor="full"
	       align="center" valign="center" visible="false" />
	<!-- 数量角标：右下（count 契约非空时显示） -->
	<Label name="CountLabel" class="count-text" text="{{count}}"
	       anchor="bottom_right" margin="2 0 4 2" visible="false" />
	<!-- 选中金边框（selected 契约联动显示，置于最上层） -->
	<Panel name="SelectedFrame" class="sel-frame" anchor="full" visible="false" />
  </Panel>
</ui>
