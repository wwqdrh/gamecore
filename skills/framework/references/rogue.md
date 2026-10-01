# 肉鸽卡牌引擎

对应 Rust 源码：`rust/src/rogue/`（engine / card / card_pile）
可运行示例：`example/rogue/rogue_game.gd`

底层为 gamealgo 算法库（带种子随机，可复现），RogueEngine 负责 Godot 暴露层。

## RogueEngine — 肉鸽引擎

```gdscript
var engine = RogueEngine.new()

engine.init_with_seed(42)                # 种子随机（同种子同结果）
engine.set_depth(3)                      # 当前层数
engine.advance_depth()                   # 推进一层
engine.get_depth(); engine.get_seed()

# 注册实体池：{"entities":[{id, type, weight, min_depth, stats}, ...]}
var ok: bool = engine.load_entities_from_json(ENTITIES_JSON)

# 生成本层卡堆：{"piles":[{id, cards:[...]}], "exit_pile_id": n}
var result = engine.generate_piles(config)
var piles: Array = result["piles"]
```

## 实体生成

```gdscript
var entity = engine.generate_entity(template_id)   # 按模板生成
var monster = engine.roll_entity("monster")        # 按类型加权随机
# 返回 Dictionary，直接读字段：entity["name"] / entity["type"] / entity["stats"]

var snapshot: String = engine.get_snapshot_json()  # 存档快照
engine.restore_from_json(snapshot)                 # 读档恢复
```

## 读取卡牌数据

`generate_piles` 返回的卡牌是 Dictionary，直接取字段：

```gdscript
for pile in result["piles"]:
    for card in pile["cards"]:
        var entity: Dictionary = card["entity"]
        print(entity["name"], " / ", entity["type"], " / ", entity["stats"])
```

包装类（需要对象接口时使用）：

```gdscript
# RogueCard: get_card_id/get_template_id/get_name/get_entity_type/get_stats
#            get_stat(name) -> f64 / is_monster/is_weapon/is_armor/is_item/is_exit
# RogueCardPile: get_pile_id/get_card_count/get_top_card/has_exit_card/get_all_cards
```

## 典型流程

1. `load_entities_from_json` 注册卡牌池（含权重与 min_depth 深度门槛）
2. 每层 `set_depth(depth)` → `generate_piles(config)` 生成本层卡堆
3. 玩家操作后 `get_snapshot_json()` 存档，读档 `restore_from_json`
