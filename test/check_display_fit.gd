# 校验脚本：GdDisplayFit 自适应分辨率（content scale）
# 运行：godot --headless --path . -s res://test/check_display_fit.gd
# 职责：1) headless 未强制调用 → 守卫生效不应用（窗口属性保持默认，
#          保障合成输入事件坐标语义不变的测试基线）
#       2) force=true 强制应用 → 根 Window 的 content_scale 属性按
#          game_config.json display 配置生效（canvas_items / keep / 1920x1080）
#       3) 重复调用幂等（同值重写无副作用）
extends SceneTree


func _initialize() -> void:
	_run()


func _run() -> void:
	var ok := true

	# 1. headless 未强制：守卫生效，不应用
	var applied: bool = GdDisplayFit.apply_display_fit(false)
	print("[Fit] headless unforced applied=%s (expect false)" % applied)
	if applied:
		push_error("[Fit] headless 未强制调用不应应用")
		ok = false

	# 2. 强制应用：display 配置写入根 Window
	applied = GdDisplayFit.apply_display_fit(true)
	print("[Fit] forced applied=%s mode=%s aspect=%s size=%s" % [
		applied, root.content_scale_mode, root.content_scale_aspect,
		root.content_scale_size])
	if not applied:
		push_error("[Fit] force=true 应应用成功（game_config.json 已配置 display 块）")
		ok = false
	if int(root.content_scale_mode) != int(Window.CONTENT_SCALE_MODE_CANVAS_ITEMS):
		push_error("[Fit] content_scale_mode 应为 canvas_items")
		ok = false
	if int(root.content_scale_aspect) != int(Window.CONTENT_SCALE_ASPECT_KEEP):
		push_error("[Fit] content_scale_aspect 应为 keep")
		ok = false
	if root.content_scale_size != Vector2i(1920, 1080):
		push_error("[Fit] content_scale_size 应为 1920x1080")
		ok = false

	# 3. 幂等：重复调用同值重写，无副作用
	GdDisplayFit.apply_display_fit(true)
	if root.content_scale_size != Vector2i(1920, 1080):
		push_error("[Fit] 重复调用应幂等")
		ok = false

	print("[Fit] RESULT=%s" % ("PASS" if ok else "FAIL"))
	quit(0 if ok else 1)
