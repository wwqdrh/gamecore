<ui>
  <!-- 右上小地图：深色地图框 + 指北标 + 玩家方位箭头。
	   无脚本：静态展示区块（正式版替换为 SubViewport 世界缩略图） -->
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
	.player-arrow { font_size: 20; }
	.map-icon { font_size: 34; }
  </style>
  <Panel name="MiniMap" class="map-frame" custom_minimum_size="172,150">
	<Panel name="MapInner" class="map-inner" anchor="full" margin="6">
	  <Label class="map-icon" text="🗺️" anchor="full" align="center" valign="center" />
	  <!-- 玩家方位箭头 -->
	  <Label name="PlayerArrow" class="player-arrow" text="⬆" anchor="center"
	         align="center" valign="center" />
	  <!-- 指北标（top_wide + align right：点锚 top_right 会把节点推出右边界外） -->
	  <Label name="NorthMark" class="north-text" text="N" anchor="top_wide"
	         margin="0 4 8 0" align="right" />
	</Panel>
  </Panel>
</ui>
