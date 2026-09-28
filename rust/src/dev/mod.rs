// dev 模块 - 单元测试框架
//
// 提供轻量的 Rust 侧测试基础设施：
// - framework: 测试注册表 + 断言上下文 + 结果收集（可被 cargo test 和 Godot 内运行器复用）
// - gd_test_runner: 暴露给 GDScript 的 DevTestRunner 类，可在 Godot 引擎内触发 Rust 测试套件
// - suites: 纯逻辑测试套件（不依赖引擎运行时，cargo test 可直接跑）
//
// 用法（Rust 侧新增测试）:
//   1. 在 suites/ 下新建 tests_xxx.rs
//   2. 编写测试函数: fn my_test(ctx: &mut TestContext) { ... }
//   3. 在该文件的 register() 中调用 framework::register("xxx", "my_test", my_test)
//   4. 在 suites/mod.rs 中声明模块并加入 register_all
//
// 用法（cargo test）:
//   cargo test -p core   # 会执行全部已注册套件，任一断言失败则测试失败
//
// 用法（Godot 内）:
//   var runner = DevTestRunner.new()
//   var report = runner.run_all_report()   # 或 run_suite("gjson")
pub mod framework;
pub mod gd_test_runner;
pub mod suites;

#[cfg(test)]
mod cargo_test_bridge {
	/// cargo test 桥接：运行全部已注册测试套件，任何失败都会让 cargo test 失败。
	/// 这样纯逻辑测试可以直接 `cargo test -p core` 跑，无需启动 Godot。
	#[test]
	fn run_all_registered_suites() {
		let results = super::framework::run_all();

		assert!(
			!results.is_empty(),
			"没有注册任何测试用例，请检查 suites/mod.rs 的 register_all"
		);

		let mut failed = 0;
		for r in &results {
			if r.passed {
				println!("[PASS] {}/{}", r.suite, r.name);
			} else {
				failed += 1;
				println!("[FAIL] {}/{}", r.suite, r.name);
				for f in &r.failures {
					println!("       - {f}");
				}
			}
		}

		println!(
			"\n共 {} 个用例，通过 {}，失败 {}",
			results.len(),
			results.len() - failed,
			failed
		);

		assert_eq!(failed, 0, "有 {failed} 个测试用例失败");
	}
}
