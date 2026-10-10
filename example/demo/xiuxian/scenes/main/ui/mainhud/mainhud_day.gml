<ui>
  <!-- 顶部中央天数徽章：显示当前游戏第几天（XiuCalendarState.day，
	   CalendarManager 按 WeatherManager 游戏时长推算写入，mainhud.gd watch 驱动） -->
  <style>
	.day-box {
	  background: #3a3450cc;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 22;
	}
	.day-num { color: #f2d98c; font_size: 17; }
  </style>
  <Panel name="DayBadge" class="day-box" custom_minimum_size="128,44">
	<HBoxContainer anchor="full" margin="14 6 14 6" separation="8">
	  <Label text="📅" valign="center" font_size="16" />
	  <Label name="DayText" class="day-num" text="第 1 天" valign="center" />
	</HBoxContainer>
  </Panel>
</ui>
