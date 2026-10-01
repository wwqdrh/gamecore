# 角色系统：移动、动画、NPC AI

对应 Rust 源码：`rust/src/role/`（movement / animator / npc_ai）

## GdRoleMover — 移动控制（CharacterBody2D）

```gdscript
var player = GdRoleMover.new()
player.control_mode = 1   # 0=不响应 CONTROL_NONE / 1=键盘 CONTROL_KEYBOARD / 3=AI CONTROL_AI
player.move_mode = 0      # 0=四向 MODE_FOUR_WAY / 1=横向 MODE_HORIZONTAL / 2=网格 MODE_GRID
player.speed = 230.0
add_child(player)
```

网格移动 + BFS 寻路（配合 GdQuickMap，见 map.md）：

```gdscript
player.move_mode = 2
player.grid_cell_size = 32
player.grid_map_path = NodePath("../QuickMap")   # 指向 GdQuickMap 节点

var path: PackedVector2Array = quickmap.find_path(from_cell, to_cell)
player.set_grid_path(path)                        # 逐格走完
player.s_grid_path_finished.connect(_on_path_finished)
```

对话/过场时暂停玩家移动：`player.set_paused(true)`（GdDialogTrigger 自动调用）。

## GdRoleAnimator — 动画驱动

挂为 GdRoleMover 子节点，依据移动状态自动切换精灵动画，无需手写状态机。

```
Player (GdRoleMover)
├── AnimatedSprite2D   (sprite_frames 配好 idle/walk 等动画)
└── Animator (GdRoleAnimator)
```

## GdNpcBrain — NPC 行为决策

挂为 NPC 的 GdRoleMover 子节点（该 Mover 的 `control_mode = 3` AI 模式）。

```gdscript
var npc = GdRoleMover.new()
npc.control_mode = 3
var brain = GdNpcBrain.new()
npc.add_child(brain)

# behavior: 0=IDLE 1=WANDER 2=PATROL 3=FOLLOW 4=FLEE 5=HUNTER
brain.behavior = 2                                        # 巡逻
brain.patrol_points = PackedVector2Array([p1, p2])        # 巡逻点

brain.behavior = 3                                        # 跟随
brain.follow_target = player
brain.follow_stop_distance = 80.0

brain.behavior = 5                                        # 追击猎手（战斗用）
brain.sight_range = 300.0        # 视野：进入即追击
brain.chase_memory = 2.5         # 失去目标后的追击记忆（秒）
brain.attack_range = 46.0        # 进入攻击距离 -> attack 相位
brain.attack_cooldown = 1.0
brain.s_attack.connect(_on_npc_attack)                    # 攻击时机信号

# 状态监听（"wander"/"chase"/"attack"/...）
brain.s_ai_state_changed.connect(func(state):
    hitbox.enabled = (state == "attack"))
```

## 完整组装示例（战斗 NPC）

摘自 `example/combat/combat_demo.gd`：

```gdscript
var enemy = GdRoleMover.new()
enemy.control_mode = 3
enemy.position = spawn_pos

var brain = GdNpcBrain.new()
brain.behavior = 5
brain.sight_range = 300.0
brain.attack_range = 46.0
enemy.add_child(brain)

var health = GdHealth.new()
health.max_health = 60.0
enemy.add_child(health)

var hurt = GdHurtbox.new()
hurt.collision_layer = 4    # 敌方受击盒 layer3
hurt.collision_mask = 0
var cs = CollisionShape2D.new(); cs.shape = RectangleShape2D.new()
hurt.add_child(cs)
enemy.add_child(hurt)

var hitbox = GdHitbox.new()
hitbox.damage = 8.0
hitbox.cooldown = 1.0
hitbox.enabled = false              # 仅 attack 相位启用
hitbox.collision_mask = 2           # 指向玩家受击盒
enemy.add_child(hitbox)
```
