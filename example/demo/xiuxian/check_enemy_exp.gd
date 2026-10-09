# 击败敌人获得修为 —— 端到端验收
#
# 运行：godot --headless --path . -s res://example/demo/xiuxian/check_enemy_exp.gd
#
# 覆盖：变体 tscn 覆写 exp_reward、击杀写入 XiuCharacterState 修为、
#       跨升级边界（大额经验自动升段）、reset_demo 基线还原。
extends SceneTree

var _total := 0
var _fails := 0


func check(cond: bool, msg: String) -> void:
	_total += 1
	if cond:
		print("[EnemyExp] PASS: ", msg)
	else:
		_fails += 1
		printerr("[EnemyExp] FAIL: ", msg)


func _init() -> void:
	call_deferred("_run")


func _run() -> void:
	var gs = XiuGameState.ins()
	gs.reset_demo()
	var character = XiuCharacterState.ins()
	check(character.level == 1 and int(character.exp) == 0,
		"reset_demo 后应为 1 级 0 修为")

	# 1. 变体 tscn 导出覆写：exp_reward 配置生效
	var slime: Node = (load("res://example/demo/xiuxian/role/enemy/slime.tscn") as PackedScene).instantiate()
	root.add_child(slime)
	await physics_frame
	check(int(slime.exp_reward) == 20, "slime.tscn exp_reward 应覆写为 20")
	check(slime.health != null, "敌人战斗组件（Health）应就绪")

	# 2. 击杀史莱姆 → +20 修为（1 级升 2 级需 100，不升级）
	slime.health.call("kill")
	await physics_frame
	check(int(character.exp) == 20,
		"击败史莱姆应 +20 修为（got %d）" % int(character.exp))
	check(character.level == 1, "20 修为不应升级")

	# 3. 击杀傀儡（+120）→ 跨升级边界自动升段
	var golem: Node = (load("res://example/demo/xiuxian/role/enemy/golem.tscn") as PackedScene).instantiate()
	root.add_child(golem)
	await physics_frame
	check(int(golem.exp_reward) == 120, "golem.tscn exp_reward 应覆写为 120")
	golem.health.call("kill")
	await physics_frame
	check(character.level == 2 and int(character.exp) == 40,
		"击杀傀儡后应 2 级余 40（20+120-100，got lv=%d exp=%d）"
		% [character.level, int(character.exp)])
	check(str(character.get_realm_title()) == "斗之气二段",
		"击杀后段位应显示 斗之气二段（got %s）" % str(character.get_realm_title()))

	# 4. exp_reward=0 的敌人不奖励、不报错
	var dumb: Node = (load("res://example/demo/xiuxian/role/enemy/enemy.gd") as GDScript).new()
	dumb.exp_reward = 0
	root.add_child(dumb)
	await physics_frame
	dumb.health.call("kill")
	await physics_frame
	check(int(character.exp) == 40, "exp_reward=0 击杀不应变动修为")

	# 还原基线
	gs.reset_demo()
	print("[EnemyExp] RESULT=%s（%d/%d PASS）"
		% ["FAIL" if _fails > 0 else "PASS", _total - _fails, _total])
	quit(1 if _fails > 0 else 0)
