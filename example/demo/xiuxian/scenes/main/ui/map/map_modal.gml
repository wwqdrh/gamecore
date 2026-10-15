<ui script="map_modal.gd">
  <!-- 世界舆图（ui/map/）：GdMiniMap 拼接缩略图（六图地形 + 传送点配对连线）
	   + 当前地图金框高亮 + 玩家呼吸光点。数据源 XiuMapThumbs（regen_map_thumbs
	   离线 dump），打开时经 MapManager 组件表取当前 alias 与玩家格子刷新 -->
  <style>
	.map-window {
	  background: #17201a;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 14;
	}
	.map-title { color: #f2ead2; font_size: 22; }
	.map-sub { color: #c9b98c; font_size: 15; }
	.map-hint { color: #9aa88f; font_size: 13; }
  </style>
  <Panel name="MapWindow" class="map-window" custom_minimum_size="960,600">
	<VBoxContainer name="MainColumn" anchor="full" margin="22 14 22 12" separation="6">
	  <HBoxContainer name="TopBar" separation="12">
		<Label name="Title" class="map-title" text="🗺️ 修仙界舆图" />
		<Label name="CurrentName" class="map-sub" text="当前：—"
		       size_flags_horizontal="expand_fill" align="right"
		       size_flags_vertical="shrink_end" />
	  </HBoxContainer>
	  <ScrollContainer name="MapScroll" size_flags_vertical="expand_fill"
	                   size_flags_horizontal="expand_fill">
		<MiniMap name="WorldMap" cell_px="8" padding="16" />
	  </ScrollContainer>
	  <Label name="Hint" class="map-hint"
	         text="◉ 青点=传送口（连线互为往返）    ● 红点=你所在的位置    金框=当前地图"
	         align="center" />
	</VBoxContainer>
  </Panel>
</ui>
