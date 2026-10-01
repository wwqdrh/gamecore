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

## GdShooter — 射击组件（Node）

挂宿主 Node2D 下，朝鼠标/指定方向发射子弹，子弹场景自动懒注册进对象池。

```gdscript
var shooter = GdShooter.new()
shooter.auto_fire_mouse = true      # 按住左键朝鼠标方向连发（悬停 UI 上不开火）
shooter.fire_cooldown = 0.18        # 射击间隔（秒）
shooter.bullet_speed = 400.0
shooter.bullet_damage = 15.0
shooter.bullet_lifetime = 1.0       # 寿命到期自动归池
shooter.muzzle_distance = 24.0      # 枪口离宿主中心距离
shooter.bullet_alias = "player_bullet"
shooter.bullet_scene_path = "res://example/combat/bullet.tscn"
host.add_child(shooter)

# 手动发射
shooter.fire(Vector2(1, 0))         # 朝方向发射 -> bool
shooter.fire_at_point(target_pos)   # 朝世界坐标
shooter.fire_toward_mouse()         # 朝鼠标

shooter.s_fired.connect(func(muzzle, dir): print("开火"))
```

## GdBullet — 池化子弹（Area2D）

子弹场景根节点类型设为 `GdBullet`，带 CollisionShape2D，`collision_mask = 4`（指向敌方受击盒）。
属性由 GdShooter 注入，无需手动设置；直接使用时：

```gdscript
bullet.velocity = Vector2(240, 0)
bullet.s_hit.connect(func(target): ...)
# 命中受击盒即毁并归池；寿命到期自动归池（走 GDSPAWNPOOL.despawn）
```

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
