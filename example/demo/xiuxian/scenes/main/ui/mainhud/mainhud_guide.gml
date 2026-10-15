<ui script="mainhud_guide.gd">
<!-- 引导与演出层：教学提示（Toast 队列）/ 突破与通关演出（Banner 字幅）/
     传送点门控提示（转发 player.s_portal_locked）。整体鼠标穿透，
     播放期挡输入由 Banner 运行期自管（STOP↔IGNORE）。
     Toast/Banner 均为 rust 侧 GML 组件（ui_toast.rs / ui_banner.rs），
     GD 侧只做状态接线不碰布局 -->
  <Control name="GuideLayer" anchor="full" mouse_filter="ignore">
	<!-- 演出字幅：突破成功 / 阶段通关 -->
	<Banner name="PerfBanner" anchor="full" hold_secs="1.8" anim_secs="0.55"
	        title_font="46" sub_font="20" dim_bg="true" />
	<!-- 教学提示条：屏幕上缘居中，队列逐条播放 -->
	<Toast name="HintToast" anchor="top_center" margin="0 64 0 64"
	       duration="2.8" fade_secs="0.3" font_size="18" />
  </Control>
</ui>
