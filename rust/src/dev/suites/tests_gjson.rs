// suite: gjson - GJson 文档存储纯逻辑测试
// 覆盖: 路径读写、加密、订阅通知、持久化 store

use std::cell::RefCell;
use std::rc::Rc;

use serde_json::json;

use crate::dev::framework::TestContext;
use crate::dev::framework;
use crate::state::gjson::{FileStore, GJson};

fn noop_store() -> FileStore {
	FileStore::new(Box::new(|| Vec::new()), Box::new(|_| {}))
}

fn suite_gjson(ctx: &mut TestContext) {
	// 路径写入与查询
	let mut g = GJson::new(noop_store());
	g.update("init;player;health", "~", json!(100));
	g.update("init;player;name", "~", json!("勇者"));

	ctx.assert_eq(g.query("init;player;health"), "100".to_string(), "路径查询 health 应为 100");
	ctx.assert_eq(
		g.query("init;player;name"),
		"\"勇者\"".to_string(),
		"路径查询 name 应为字符串",
	);
	ctx.assert_true(g.query_value("init;player").is_some(), "嵌套父路径应可查询");
	ctx.assert_true(g.query_value("init;player;missing").is_none(), "不存在的路径应返回 None");
}

fn encrypt_roundtrip(ctx: &mut TestContext) {
	let raw = r#"{"init":{"gold":42},"config":{"volume":0.8}}"#;
	let encrypted = GJson::encrypt(raw);
	ctx.assert_true(encrypted != raw.as_bytes(), "加密后字节流应与原文不同");

	let decrypted = GJson::decrypt(&encrypted);
	ctx.assert_eq(decrypted, raw.to_string(), "解密后应还原原文");
}

fn subscribe_notify(ctx: &mut TestContext) {
	let mut g = GJson::new(noop_store());

	let hits = Rc::new(RefCell::new(Vec::<String>::new()));
	let hits_clone = hits.clone();
	g.subscribe("init;player;health", Box::new(move |path, _value| {
		hits_clone.borrow_mut().push(path.to_string());
		true
	}));

	// action = "~" 强制写入，应触发订阅
	g.update("init;player;health", "~", json!(80));
	ctx.assert_eq(hits.borrow().len(), 1, "强制写入应触发一次订阅回调");

	// action != "~" 且路径已存在时不写入，不触发订阅
	g.update("init;player;health", "set", json!(50));
	ctx.assert_eq(hits.borrow().len(), 1, "非强制写入已存在路径不应触发回调");

	// action != "~" 但路径不存在时应写入并触发
	g.update("init;player;mana", "set", json!(30));
	ctx.assert_eq(hits.borrow().len(), 1, "订阅只监听 health 路径，mana 写入不应触发");
	ctx.assert_true(g.query_value("init;player;mana").is_some(), "新路径应被写入");
}

fn file_store_persistence(ctx: &mut TestContext) {
	// 用 Rc 模拟磁盘：save_fn 写入、load_fn 读出
	let disk: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));

	let make_store = |disk: Rc<RefCell<Vec<u8>>>| {
		let load_disk = disk.clone();
		let save_disk = disk.clone();
		FileStore::new(
			Box::new(move || load_disk.borrow().clone()),
			Box::new(move |data: Vec<u8>| *save_disk.borrow_mut() = data),
		)
	};

	let mut g = GJson::new(make_store(disk.clone()));
	g.enable_encrypt();
	g.update("init;gold", "~", json!(999));

	ctx.assert_true(!disk.borrow().is_empty(), "写入后应触发持久化保存");

	// 新实例从同一 store 恢复（加密数据需在内部解密）
	let mut g2 = GJson::new(make_store(disk.clone()));
	g2.enable_encrypt();
	g2.load_by_store();
	ctx.assert_eq(
		g2.query("init;gold"),
		"999".to_string(),
		"加密存档经新实例加载后应还原数据",
	);

	ctx.assert_contains(
		&g.duplicate_all_string(),
		"999",
		"duplicate_all_string 应序列化出已写入的值",
	);
}

fn reload_data(ctx: &mut TestContext) {
	let mut g = GJson::new(noop_store());
	g.update("init;hp", "~", json!(1));
	g.reload_data(r#"{"init":{"hp":55,"mp":10}}"#);
	ctx.assert_eq(g.query("init;hp"), "55".to_string(), "reload_data 后 hp 应为新值");
	ctx.assert_eq(g.query("init;mp"), "10".to_string(), "reload_data 后 mp 应存在");
}

pub fn register() {
	framework::register("gjson", "suite_gjson", suite_gjson);
	framework::register("gjson", "encrypt_roundtrip", encrypt_roundtrip);
	framework::register("gjson", "subscribe_notify", subscribe_notify);
	framework::register("gjson", "file_store_persistence", file_store_persistence);
	framework::register("gjson", "reload_data", reload_data);
}
