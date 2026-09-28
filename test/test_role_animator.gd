# suite: role - GdRoleAnimator 动画/贴图管理测试
# 覆盖: AnimatedSprite2D 命名约定自动播放、翻转复用动画、手动播放、
#        自动创建精灵、Sprite2D 状态贴图表与 fallback
# 依赖: GDExtension 类 GdRoleMover / GdRoleAnimator
extends "res://test/test_case.gd"


func _tree() -> SceneTree:
	return Engine.get_main_loop() as SceneTree


func wait_phys(frames: int) -> void:
	var t := _tree()
	for i in frames:
		await t.physics_frame


func wait_phys_until(cond: Callable, timeout_ticks: int = 600) -> bool:
	var t := _tree()
	var n := 0
	while n < timeout_ticks:
		if cond.call():
			return true
		await t.physics_frame
		n += 1
	return cond.call()


## 给 SpriteFrames 添加一个动画（PlaceholderTexture 帧即可）
func _add_anim(frames: SpriteFrames, anim_name: String, frame_count: int = 1) -> void:
	if not frames.has_animation(anim_name):
		frames.add_animation(anim_name)
	var tex := PlaceholderTexture2D.new()
	tex.size = Vector2(16, 16)
	for i in frame_count:
		frames.add_frame(anim_name, tex)


## 组装 mover + 已带帧动画的 AnimatedSprite2D + animator
func _make_animated_rig(nm: String, frames: SpriteFrames) -> Dictionary:
	var mover := GdRoleMover.new()
	mover.name = nm
	var sprite := AnimatedSprite2D.new()
	sprite.sprite_frames = frames
	var animator := GdRoleAnimator.new()
	mover.add_child(sprite)
	mover.add_child(animator)
	_tree().root.add_child(mover)
	return {"mover": mover, "sprite": sprite, "animator": animator}


func test_auto_create_sprite() -> void:
	var mover := GdRoleMover.new()
	mover.name = "AnimAutoCreate"
	var animator := GdRoleAnimator.new()
	mover.add_child(animator)
	_tree().root.add_child(mover)
	await wait_phys(2)

	var created: Node = mover.get_node_or_null("AnimatedSprite2D")
	assert_not_null(created, "未找到精灵时应在父节点下自动创建 AnimatedSprite2D")
	mover.free()


func test_animated_four_way_naming() -> void:
	var frames := SpriteFrames.new()
	_add_anim(frames, "idle_down")
	_add_anim(frames, "idle_right")
	_add_anim(frames, "walk_down", 2)
	_add_anim(frames, "walk_right", 2)
	var rig := _make_animated_rig("Anim4Way", frames)
	var mover: GdRoleMover = rig["mover"]
	var animator: GdRoleAnimator = rig["animator"]
	await wait_phys(2)

	# 默认朝向 right + 待机
	var ok := await wait_until(func(): return animator.get_current_anim() == "idle_right", 120)
	assert_true(ok, "初始应播放 idle_right (actual: %s)" % animator.get_current_anim())

	# 朝向下 + 待机
	mover.set_facing("down")
	ok = await wait_until(func(): return animator.get_current_anim() == "idle_down", 120)
	assert_true(ok, "朝向下待机应播放 idle_down (actual: %s)" % animator.get_current_anim())

	# 向右移动
	mover.set_ai_direction(Vector2.RIGHT)
	ok = await wait_until(func(): return animator.get_current_anim() == "walk_right", 240)
	assert_true(ok, "向右移动应播放 walk_right (actual: %s)" % animator.get_current_anim())

	# 停止后回到待机
	mover.stop()
	ok = await wait_until(func(): return animator.get_current_anim() == "idle_right", 240)
	assert_true(ok, "停止后应回到 idle_right (actual: %s)" % animator.get_current_anim())
	mover.free()


func test_flip_reuse_horizontal() -> void:
	# 只有 _right 后缀动画: 朝左时应复用 _right 并翻转
	var frames := SpriteFrames.new()
	_add_anim(frames, "idle_right")
	_add_anim(frames, "walk_right", 2)
	var rig := _make_animated_rig("AnimFlip", frames)
	var mover: GdRoleMover = rig["mover"]
	var sprite: AnimatedSprite2D = rig["sprite"]
	var animator: GdRoleAnimator = rig["animator"]
	await wait_phys(2)

	mover.set_ai_direction(Vector2.LEFT)
	var ok := await wait_until(
		func(): return animator.get_current_anim() == "walk_right" and sprite.flip_h,
		240)
	assert_true(ok, "朝左移动应复用 walk_right 并翻转 flip_h (anim=%s, flip=%s)"
		% [animator.get_current_anim(), sprite.flip_h])

	mover.stop()
	ok = await wait_until(func(): return animator.get_current_anim() == "idle_right", 240)
	assert_true(ok, "停止后应回到 idle_right")
	mover.free()


func test_manual_play_anim_and_signal() -> void:
	var frames := SpriteFrames.new()
	_add_anim(frames, "idle_right")
	_add_anim(frames, "walk_down", 2)
	var rig := _make_animated_rig("AnimManual", frames)
	var animator: GdRoleAnimator = rig["animator"]
	await wait_phys(2)

	var changed := {"fired": false}
	animator.s_anim_changed.connect(func(n): changed["fired"] = true)

	animator.play_anim("walk_down")
	assert_eq(animator.get_current_anim(), "walk_down", "手动播放应立即更新 current_anim")
	await wait_frames(1)
	assert_true(changed["fired"], "手动播放应触发 s_anim_changed 信号")
	rig["mover"].free()


func test_sprite2d_texture_table() -> void:
	var mover := GdRoleMover.new()
	mover.name = "AnimSprite2D"
	var sprite := Sprite2D.new()
	var animator := GdRoleAnimator.new()
	var tex_idle := PlaceholderTexture2D.new()
	tex_idle.size = Vector2(16, 16)
	var tex_walk := PlaceholderTexture2D.new()
	tex_walk.size = Vector2(16, 16)

	# idle 状态任意朝向 -> tex_idle; walk 未注册 -> fallback_tex
	animator.set_state_texture("idle", "", tex_idle)
	animator.fallback_tex = tex_walk

	mover.add_child(sprite)
	mover.add_child(animator)
	_tree().root.add_child(mover)
	await wait_phys(2)

	var ok := await wait_until(func(): return sprite.texture == tex_idle, 120)
	assert_true(ok, "待机状态应使用注册的 idle 贴图")

	# 朝左应翻转静态贴图
	mover.set_facing("left")
	ok = await wait_until(func(): return sprite.flip_h, 120)
	assert_true(ok, "朝左应翻转 Sprite2D (flip_h)")
	assert_true(sprite.texture == tex_idle, "翻转不改变贴图")

	# 移动进入 walk 状态: 未注册贴图 -> fallback
	mover.set_ai_direction(Vector2.RIGHT)
	ok = await wait_until(func(): return sprite.texture == tex_walk, 240)
	assert_true(ok, "未注册的 walk 状态应使用 fallback_tex")
	mover.stop()
	mover.free()


func test_clear_state_texture() -> void:
	var mover := GdRoleMover.new()
	mover.name = "AnimClearTex"
	var sprite := Sprite2D.new()
	var animator := GdRoleAnimator.new()
	var tex_a := PlaceholderTexture2D.new()
	tex_a.size = Vector2(16, 16)
	animator.set_state_texture("idle", "", tex_a)
	mover.add_child(sprite)
	mover.add_child(animator)
	_tree().root.add_child(mover)
	await wait_phys(2)

	var ok := await wait_until(func(): return sprite.texture == tex_a, 120)
	assert_true(ok, "注册后应使用该贴图")

	var tex_b := PlaceholderTexture2D.new()
	tex_b.size = Vector2(16, 16)
	animator.fallback_tex = tex_b
	animator.clear_state_texture("idle", "")
	ok = await wait_until(func(): return sprite.texture == tex_b, 120)
	assert_true(ok, "清除后应回退到 fallback_tex")
	mover.free()
