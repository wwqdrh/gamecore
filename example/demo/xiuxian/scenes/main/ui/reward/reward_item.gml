<ui script="reward_item.gd">
  <!-- 领奖弹窗奖励格子条目模板：作为 reward_panel.gml 中 UIHList 的 slot 模板
	   （构建期注入）。数据契约见 reward_item.gd 的 @export 列表：
	   icon(emoji) / icon_texture(贴图路径，非空时切换 TextureRect) /
	   count(数量角标 "x2") / tips_title / tips(悬停用途说明)
	   悬停 tips 运行期接线：条目重建后由本脚本 _apply_tips 沿父链找
	   面板控制器的 gml_tips_host() 宿主（GdUITooltip）调 setup_hover -->
  <style>
	.cell-bg {
	  background: #2a2418d9;
	  border_color: #b7964a;
	  border_width: 1;
	  border_radius: 8;
	}
	.cell-icon { font_size: 30; }
	.count-text { color: #f2e3b8; font_size: 12; }
	.cell-name { color: #d8cba8; font_size: 11; }
  </style>
  <!-- 根节点：shrink_center 防被列表行拉伸，固定格子尺寸 -->
  <Panel name="RewardCell" class="cell-bg" custom_minimum_size="78,96"
		 size_flags_horizontal="shrink_center"
		 tips_title="{{tips_title}}" tips="{{tips}}">
	<!-- 物品图标：字符图标（Label emoji）与贴图（TextureRect）二选一显隐 -->
	<CenterContainer name="IconBox" anchor="full" margin="5 8 5 24">
	  <Label name="CellIcon" class="cell-icon" text="{{icon}}" />
	  <TextureRect name="CellTex" visible="false" expand_mode="ignore_size"
	               stretch_mode="keep_aspect_centered" custom_minimum_size="34,34" />
	</CenterContainer>
	<!-- 物品名（格子底部） -->
	<Label name="NameLabel" class="cell-name" text="{{name}}"
	       anchor="bottom_wide" margin="4 -30 4 3" align="center" />
	<!-- 数量角标：右下（bottom_wide + align right，点锚会推出格子外） -->
	<Label name="CountLabel" class="count-text" text="{{count}}"
	       anchor="bottom_wide" margin="0 -18 6 2" align="right" />
  </Panel>
</ui>
