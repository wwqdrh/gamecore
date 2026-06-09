# 控制台示例 - 演示命令注册和调用
# 按 ` 键打开控制台，输入 Lua 命令或已注册的 GDScript 命令
extends Node2D

var player_hp: int = 100
var player_name: String = "Hero"
var score: int = 0


func _ready() -> void:
	var console = Engine.get_singleton("GdConsole")

	# 注册 GDScript 命令
	console.register_command("heal", _cmd_heal, "Heal player by amount (e.g. heal(50))")
	console.register_command("damage", _cmd_damage, "Damage player by amount (e.g. damage(30))")
	console.register_command("status", _cmd_status, "Show player status")
	console.register_command("set_name", _cmd_set_name, "Set player name (e.g. set_name('Alice'))")
	console.register_command("add_score", _cmd_add_score, "Add score (e.g. add_score(100))")
	console.register_command("reset", _cmd_reset, "Reset player state")

	# 添加控制台面板到场景（运行时方式）
	var panel = load("res://addons/gamecore/ui/console_panel.gd").new()
	add_child(panel)

	print("Console example ready! Press ` to open console.")
	print("Try: help(), fps(), status(), heal(50)")


func _cmd_heal(amount: int) -> String:
	player_hp = mini(player_hp + amount, 999)
	return "Healed by %d, HP: %d" % [amount, player_hp]


func _cmd_damage(amount: int) -> String:
	player_hp = maxi(player_hp - amount, 0)
	return "Damaged by %d, HP: %d" % [amount, player_hp]


func _cmd_status() -> String:
	return JSON.stringify({
		"name": player_name,
		"hp": player_hp,
		"score": score,
	})


func _cmd_set_name(new_name: String) -> String:
	player_name = new_name
	return "Name set to: %s" % player_name


func _cmd_add_score(amount: int) -> String:
	score += amount
	return "Score: %d (+%d)" % [score, amount]


func _cmd_reset() -> String:
	player_hp = 100
	player_name = "Hero"
	score = 0
	return "Player state reset!"
