# 集合类型（Dictionary、Array、Variant）

## VarDictionary

```rust
use godot::builtin::{VarDictionary, VarArray, Variant};

let mut dict = VarDictionary::new();
dict.set(&"key".to_variant(), &42.to_variant());

// get() 返回 Option<Variant>，不是 Option<&Variant>
let value: i32 = dict.get(&"key".to_variant())
    .map(|v| v.to::<i32>())
    .unwrap_or(0);

// 检查 key 是否存在
if dict.contains_key(&"key".to_variant()) {
    // ...
}

// 遍历
for (key, value) in dict.iter_shared() {
    let k: String = key.to();
    let v: i32 = value.to();
}

// 获取所有 keys
let keys = dict.keys_array();  // Array<Variant>
```

**易错点**：`VarDictionary::get()` 返回 `Option<Variant>`（不是 `Option<&Variant>`），不能直接 `.to()`，需要先 `.unwrap()` 或 `.map(|v| v.to::<T>())`。

## Array

```rust
use godot::builtin::Array;

let mut arr: Array<Vector2i> = Array::new();
arr.push(Vector2i::new(0, 0));

// 遍历
for item in arr.iter_shared() {
    // ...
}

// 从 Array<Variant> 转换为 Array<Vector2i>
let mut edited: Array<Vector2i> = Array::new();
for v in variant_array.iter_shared() {
    edited.push(v.to::<Vector2i>());
}

// collect 不能直接从 Variant 迭代器收集为 Array<Vector2i>
// 错误：let arr: Array<Vector2i> = variants.iter_shared().collect();
// 正确：手动循环 push
```

## Variant 类型转换

```rust
let v = 42i32.to_variant();
let n: i32 = v.to::<i32>();
let s: String = v.to::<String>();
let vec: Vector2i = v.to::<Vector2i>();
let dict: VarDictionary = v.to::<VarDictionary>();
```

## VarArray（无类型数组）

```rust
use godot::builtin::VarArray;

let mut arr = VarArray::new();
arr.push(&42.to_variant());
arr.push(&"hello".to_variant());

// 遍历
for item in arr.iter_shared() {
    let n: i32 = item.to();
}
```

## 嵌套 Dictionary 转换

```rust
// 从 VarDictionary 中获取嵌套的 VarArray
let display_to_world: Vec<Vec<CellNeighbor>> = dict
    .get(&"key".to_variant())
    .map(|v| -> VarArray { v.to() })
    .unwrap_or(VarArray::new())
    .iter_shared()
    .map(|v| -> Vec<CellNeighbor> {
        let row: VarArray = v.to();
        row.iter_shared()
            .map(|n| -> CellNeighbor {
                let ord: i32 = n.to();
                CellNeighbor::from_ord(ord)
            })
            .collect()
    })
    .collect();
```
