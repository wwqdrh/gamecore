# 战斗组件、对象池、事件总线、调试绘制

对应 Rust 源码：`rust/src/components/`、`rust/src/pool/`、`rust/src/event/`、`rust/src/debug/`

## GdHealth — 血量组件（Node）

```gdscript
var health = GdHealth.new()
health.max_health = 100.0
health.invincible_time = 0.6        # 受击无敌帧
health.god_mode = false
add_child(health)

health.take_damage(10.0)
health.heal(20.0)
health.revive(100.0)
health.kill()
health.get_health_ratio()           # -> float 0..1

health.s_damaged.connect(func(amount, current): print("扣血 ", amount, " 余 ", current))
health.s_healed.connect(func(amount, current): ...)
health.s_died.connect(_on_died)
```

**伤害自动转发**：GdHurtbox 找到宿主或祖先链上的 GdHealth 后自动扣血，业务层只需监听信号。

## GdHurtbox / GdHitbox — 受击与攻击判定（Area2D）

```gdscript
# 受击盒（挂在被打的一方）
var hurt = GdHurtbox.new()
hurt.collision_layer = 2    # 玩家受击盒=2 / 敌方=4
hurt.collision_mask = 0
hurt.defense = 4.0          # 防御减伤：实际伤害 = max(1, 伤害 - 防御)
var cs = CollisionShape2D.new(); cs.shape = RectangleShape2D.new()
hurt.add_child(cs)
body.add_child(hurt)

# 攻击盒（挂在打人的一方）
var hitbox = GdHitbox.new()
hitbox.damage = 8.0
hitbox.cooldown = 1.0       # 对同一目标触发冷却
hitbox.enabled = false      # 常关，攻击相位再开
hitbox.collision_mask = 2   # 指向目标受击盒 layer
enemy.add_child(hitbox)
```

约定：Hitbox 的 mask 指向**目标** Hurtbox 的 layer（玩家受击盒=2，敌方受击盒=4）。

## 修仙 demo 视野追击小怪范本（推荐参考）

`example/demo/xiuxian/role/enemy/enemy.gd`（`class_name XiuEnemyBase extends GdRoleMover`）：
一张基类脚本组装完整敌人，变体小怪**不需要子脚本**——不同 tscn 覆写导出值即可
（slime/fox/golem：血量/攻击/防御/速度/视野/颜色不同）：

- **装配**（_ready 代码组装，零场景编辑）：GdHealth（max_health + 受击无敌帧）
  → GdHurtbox（layer=4，defense=自身防御）→ GdHitbox（mask=2 常开接触伤害，
  damage=attack，cooldown 1.0）→ GdNpcBrain（AI_HUNTER：sight_range 视野追击、
  attack_range 停下、无目标时 wander_radius 游走）
- **目标绑定**：`brain.follow_target = player`（"player" 分组解析，_process 重试）
- **网格地图**：`grid_map_path = NodePath("..")`（敌人是地图场景子节点），
  Brain 网格适配自动走 BFS 寻路不穿地形；落位 BFS 吸附后 `brain.restart()`
- **表现**：受击闪白（tween modulate）+ 迷你血条（health.get_health_ratio）；
  死亡 s_died → 关 Brain → 淡出 → queue_free
- 玩家远程射击：`role/player/player.gd`（GdShooter unhandled 路由自动开火：
  持枪=auto_fire_mouse 随装备联动开关，左键/右键都开火；未持枪时左键留给寻路）
  + `role/player/bullet.tscn`
  （根 type="GdBullet"、mask=4、Polygon2D 圆形视觉）
- 玩家近战/装备系统：同脚本 GdMelee + WeaponMount 武器挂点——
  mainhud 装备栏选中槽位写 GdState "mainhud.equip"，player watch 后按
  物品 id 切换攻击模式（gun=射击 / sword_qingfeng=近战 / 其他=徒手），
  并把对应武器图形节点装配到 WeaponMount（剑=Polygon2D 剑刃 + 护手握柄，
  枪=矩形枪身枪口，简单基础图形占位）；挥击时挂点转向回弹做动感反馈
- 验收：`check_enemy_flow.gd`（装配/追击/接触伤害/子弹减伤数值/击杀回收/连通性）、
  `check_equip_flow.gd`（装备栏数字键/点击选择/模式互斥/武器图形装配/挥击信号）、
  `check_melee.gd`（近战组件单元验收）

## GdShooter — 射击组件（Node）

挂宿主 Node2D 下，朝鼠标/指定方向发射子弹，子弹场景自动懒注册进对象池。

**开火输入路由（框架红线）**：`auto_fire_mouse = true` 时开火输入走
`unhandled_input` 事件路由——被 mouse_filter=STOP 的 UI 控件消费的点击
到不了 unhandled 阶段，引擎保证「点击 UI 不开火、点击世界开火」，
无需任何 hover 猜测。**严禁在业务侧用 `Input.is_mouse_button_pressed`
轮询 + `gui_get_hovered_control()` 猜测**（hover 会被全屏 PASS/STOP
壳层污染，且轮询与"这一次点击落在哪"脱钩，行为必然失灵）。
按下即开火并按住连射（冷却内置）；release 被 UI 吞掉时由 process 中
Input 轮询校准复位（防卡死）。宿主实现 `is_paused()`（如 GdRoleMover
对话锁）时暂停期不接开火输入。`fire_button_left/fire_button_right`
配置开火按键（默认左键开、右键关）。

```gdscript
var shooter = GdShooter.new()
shooter.auto_fire_mouse = true      # unhandled 路由自动开火：点 UI 不开火/点世界开火
shooter.fire_button_right = true    # 追加右键开火（默认仅左键）
shooter.fire_cooldown = 0.18        # 射击间隔（秒）
shooter.bullet_speed = 400.0
shooter.bullet_damage = 15.0
shooter.bullet_lifetime = 1.0       # 寿命到期自动归池
shooter.muzzle_distance = 24.0      # 枪口离宿主中心距离
shooter.max_distance = 320.0        # 最大射程（像素，超程销毁；0 = 不限）
shooter.bullet_alias = "player_bullet"
shooter.bullet_scene_path = "res://example/combat/bullet.tscn"
host.add_child(shooter)

# 手动发射（auto_fire_mouse=false 时完全由外部驱动）
shooter.fire(Vector2(1, 0))         # 朝方向发射 -> bool
shooter.fire_at_point(target_pos)   # 朝世界坐标
shooter.fire_toward_mouse()         # 朝鼠标

shooter.s_fired.connect(func(muzzle, dir): print("开火"))
```

验收：`test/check_input_routing.gd`（点 UI 不开火/点世界开火/按住连射/
宿主暂停拦截/未配置按键不响）、`test/check_bullet_terrain.gd`
（射程销毁/撞山销毁/高海拔飞越/lifetime 统一出口）。

## GdMelee — 近战攻击组件（Node2D）

挂宿主 Node2D 下，ready 自动创建内嵌挥击盒 `SwingHitbox`（GdHitbox 子节点 +
矩形判定形状，layer=0 / monitorable=false / mask=target_mask 默认 4 指向敌方
受击盒）。挥击 = 判定盒定位到 `dir * attack_offset`（rotation = dir.angle()）
并使能 `swing_window` 秒，重叠扫描与伤害由内嵌 GdHitbox 完成，窗口结束复位。

**注意 base 必须是 Node2D**：Area2D 挂在普通 Node（非 CanvasItem）下变换
继承会断链（global_position 不含祖先 Node2D 偏移），组件因此继承 Node2D。

攻击输入路由与 GdShooter 同套约定：`auto_attack_mouse = true` 时走
unhandled 事件路由（点 UI 不攻击/点世界攻击、按住连击、release 被吞时
Input 校准复位、宿主 is_paused() 暂停期不接输入）。

```gdscript
var melee = GdMelee.new()
melee.auto_attack_mouse = true      # unhandled 路由自动挥击：点 UI 不攻击/点世界攻击
melee.attack_button_left = true     # 攻击按键（默认左键开、右键关）
melee.attack_damage = 30.0          # 单次挥击伤害（写入内嵌 GdHitbox）
melee.attack_cooldown = 0.45        # 挥击间隔（秒）
melee.attack_range = 56.0           # 判定盒长度（沿攻击方向，像素）
melee.attack_width = 48.0           # 判定盒宽度（垂直攻击方向）
melee.swing_window = 0.12           # 判定盒使能窗口（秒）
melee.target_mask = 4               # 目标受击盒掩码（4 = 敌方 layer3）
melee.hit_cooldown = 0.1            # 窗口期命中冷却（影响一次挥击命中数）
host.add_child(melee)

# 手动挥击（auto_attack_mouse=false 时完全由外部驱动）
melee.swing(Vector2(1, 0))          # 朝方向挥击 -> bool
melee.swing_at_point(target_pos)    # 朝世界坐标
melee.swing_toward_mouse()          # 朝鼠标
melee.stop()                        # 立即终止（装备切换时）
melee.is_swinging()                 # 是否在挥击窗口中

melee.s_swing.connect(func(dir): print("挥击 ", dir))
```

验收：`test/check_melee.gd`（装配/挥击定位/冷却/窗口复位/范围内命中掉血/
范围外不误伤/stop 终止）。

## GdBullet — 池化子弹（Area2D）

子弹场景根节点类型设为 `GdBullet`，带 CollisionShape2D，`collision_mask = 4`（指向敌方受击盒）。
属性由 GdShooter 注入，无需手动设置；直接使用时：

```gdscript
bullet.velocity = Vector2(240, 0)
bullet.s_hit.connect(func(target): ...)        # 命中受击盒即毁并归池
bullet.s_destroyed.connect(func(reason): ...)  # 非命中销毁：range/terrain/lifetime
```

- **射程**：`max_distance`（像素，飞行距离超限销毁；0 = 不限），由 GdShooter 每发写入。
- **地形高度检测**：`fly_height`（子弹飞行海拔，格）——沿运动线段按 8px 步长采样
  地图高度场，`地形海拔 > fly_height` 即撞毁（`check_terrain = false` 可关）。
  依赖 **terrain_provider 协议**：场景树内 `terrain_provider` 分组中实现
  `get_height_at_world(world: Vector2) -> i32` 的节点即为高度场数据源
  （GdQuickMap 内置支持；自定义地图/GDScript 地图入组即可）。子弹与地图零耦合，
  换图后旧 provider 失效自动重查。

## GdSpawnPool — 对象池（单例 "GDSPAWNPOOL"）

```gdscript
var pool = Engine.get_singleton("GDSPAWNPOOL")

pool.register("bullet", "res://example/combat/bullet.tscn", 8)  # (alias, 路径, 预热数)
# 或注册已实例化的场景：pool.register_scene("bullet", packed_scene, 3)

var bullet = pool.spawn("bullet", self, Vector2.ZERO)  # (alias, 父节点, 位置)
bullet.velocity = Vector2(240, 0)

pool.despawn(bullet)                    # 归池不 free（按 scene_file_path 匹配）
pool.idle_count("bullet")               # 池中空闲数
pool.created_count("bullet")            # 累计创建数
pool.clear("bullet")                    # 清空该池
```

## GdEventBus — 全局事件总线（单例 "GDEVENTBUS"）

跨场景解耦广播，发布/订阅：

```gdscript
var bus = Engine.get_singleton("GDEVENTBUS")

var id: int = bus.subscribe("enemy_died", func(args): score += 1)  # args 为 Array
bus.subscribe_once("game_over", func(args): ...)   # 触发一次自动退订
bus.publish("enemy_died", [enemy_id, "slime"])     # 发布
bus.unsubscribe("enemy_died", id)                  # -> bool
bus.subscriber_count("enemy_died")                 # -> int
bus.clear("enemy_died")
```

## GdDebugDraw — 调试绘制（单例 "GDDEBUGDRAW"）

运行时画 2D 调试图形，`life` 秒后自动消失（life <= 0 表示常驻）。

```gdscript
var dd = Engine.get_singleton("GDDEBUGDRAW")

dd.draw_line(from, to, Color(0.4, 1, 0.4, 0.8), 2.0)              # 线段 (起, 终, 颜色, life)
dd.draw_ray(pos, Vector2(1, 0), 160.0, Color(1, 0.9, 0.2), 2.0)   # 射线（带箭头）
dd.draw_circle(pos, 120.0, true, Color(1, 0, 0, 0.12), 2.0)       # (圆心, 半径, filled, 颜色, life)
dd.draw_rect(a, b, false, Color(0.5, 0.7, 1, 0.5), 2.0)           # 矩形 (对角点, filled, ...)
```

调试日志确认问题后记得删除；调试绘制仅用于开发期，随包发布前应移除调用。
