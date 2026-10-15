<ui script="mainhud_minimap.gd">
  <!-- 右上小地图：GdMiniMap 聚焦模式——只画当前地图（地形/传送点/玩家光点），
	   按控件尺寸自适应缩放居中；数据源 XiuMapThumbs（regen_map_thumbs 离线 dump），
	   脚本随地图切换/玩家移动刷新。点击面板打开世界舆图（MapModal） -->
  <style>
	.map-frame {
	  background: #2e4a3a;
	  border_color: #e8b93e;
	  border_width: 3;
	  border_radius: 12;
	}
	.map-inner {
	  background: #243c2e;
	  border_color: #56705c;
	  border_width: 1;
	  border_radius: 8;
	}
	.north-text { color: #f2ead2; font_size: 13; }
  </style>
  <Panel name="MiniMap" class="map-frame" custom_minimum_size="176,150"
	     mouse_default_cursor_shape="pointing_hand" @pressed="show:MapModal">
	<Panel name="MapInner" class="map-inner" anchor="full" margin="6">
	  <!-- 聚焦模式：fit_current=true 只画当前地图并铺满；传送点青光点 +
		   玩家红点由组件绘制；地图名不靠组件（贴图顶部会被裁）→ 底部 Label -->
	  <MiniMap name="MiniMapView" anchor="full" margin="2" fit_current="true"
	           show_names="false" padding="4" />
	  <!-- 指北标（top_wide + align right：点锚 top_right 会把节点推出右边界外） -->
	  <Label name="NorthMark" class="north-text" text="N" anchor="top_wide"
	         margin="0 4 8 0" align="right" />
	  <Label name="MapName" class="north-text" text="—" anchor="bottom_wide"
	         margin="0 0 0 4" align="center" />
	</Panel>
  </Panel>
</ui>
