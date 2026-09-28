// 测试框架核心
// - TestContext: 单个测试用例的断言上下文，失败会记录而不是立即 panic，
//   让一个用例能收集所有失败点，报告更完整。
// - TestCase / TestResult: 用例定义与执行结果。
// - registry: 全局静态注册表，suite 名 -> 一组用例。

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

/// 测试用例函数签名
pub type TestFn = fn(&mut TestContext);

/// 断言上下文：一个测试用例执行期间持有，收集失败信息
pub struct TestContext {
	suite: String,
	name: String,
	failures: Vec<String>,
}

impl TestContext {
	fn new(suite: &str, name: &str) -> Self {
		Self {
			suite: suite.to_string(),
			name: name.to_string(),
			failures: Vec::new(),
		}
	}

	/// 记录一条失败
	pub fn fail(&mut self, msg: impl Into<String>) {
		self.failures.push(msg.into());
	}

	/// 断言条件为真
	pub fn assert_true(&mut self, cond: bool, msg: impl Into<String>) {
		if !cond {
			self.failures.push(format!("[断言失败] {}", msg.into()));
		}
	}

	/// 断言条件为假
	pub fn assert_false(&mut self, cond: bool, msg: impl Into<String>) {
		if cond {
			self.failures.push(format!("[断言失败] {}", msg.into()));
		}
	}

	/// 断言相等（需要 PartialEq + Debug）
	pub fn assert_eq<T: PartialEq + std::fmt::Debug>(&mut self, left: T, right: T, msg: impl Into<String>) {
		if left != right {
			self.failures.push(format!(
				"[断言失败] {} (left: {left:?}, right: {right:?})",
				msg.into()
			));
		}
	}

	/// 断言不相等
	pub fn assert_ne<T: PartialEq + std::fmt::Debug>(&mut self, left: T, right: T, msg: impl Into<String>) {
		if left == right {
			self.failures.push(format!(
				"[断言失败] {} (两者相等: {left:?})",
				msg.into()
			));
		}
	}

	/// 断言浮点近似相等
	pub fn assert_near(&mut self, left: f64, right: f64, eps: f64, msg: impl Into<String>) {
		if (left - right).abs() > eps {
			self.failures.push(format!(
				"[断言失败] {} (left: {left}, right: {right}, eps: {eps})",
				msg.into()
			));
		}
	}

	/// 断言字符串包含子串
	pub fn assert_contains(&mut self, haystack: &str, needle: &str, msg: impl Into<String>) {
		if !haystack.contains(needle) {
			self.failures.push(format!(
				"[断言失败] {} (内容中未找到 {needle:?}: {haystack:?})",
				msg.into()
			));
		}
	}
}

/// 测试用例定义
#[derive(Clone, Copy)]
pub struct TestCase {
	pub suite: &'static str,
	pub name: &'static str,
	pub func: TestFn,
}

/// 单个用例的执行结果
pub struct TestResult {
	pub suite: String,
	pub name: String,
	pub passed: bool,
	pub failures: Vec<String>,
	pub elapsed_us: u128,
}

// ---------------------------------------------------------------------------
// 全局注册表
// ---------------------------------------------------------------------------

fn registry() -> &'static Mutex<Vec<TestCase>> {
	static REGISTRY: OnceLock<Mutex<Vec<TestCase>>> = OnceLock::new();
	REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

/// 首次访问注册表接口时注册全部内建测试套件（cargo test 与 Godot 运行器共用）
pub fn ensure_registered() {
	static ONCE: OnceLock<()> = OnceLock::new();
	if ONCE.set(()).is_ok() {
		crate::dev::suites::register_all();
	}
}

/// 注册一个测试用例（suite 相同的用例归入同一套件）
pub fn register(suite: &'static str, name: &'static str, func: TestFn) {
	let mut reg = registry().lock().unwrap();
	reg.push(TestCase { suite, name, func });
}

/// 运行全部用例（按 suite 分组、组内按注册顺序）
pub fn run_all() -> Vec<TestResult> {
	ensure_registered();
	let snapshot: Vec<TestCase> = registry().lock().unwrap().clone();
	run_cases(snapshot)
}

/// 运行指定 suite 的全部用例
pub fn run_suite(suite: &str) -> Vec<TestResult> {
	ensure_registered();
	let snapshot: Vec<TestCase> = registry()
		.lock()
		.unwrap()
		.iter()
		.copied()
		.filter(|c| c.suite == suite)
		.collect();
	run_cases(snapshot)
}

/// 列出全部 suite 名（去重、有序）
pub fn list_suites() -> Vec<String> {
	ensure_registered();
	let reg = registry().lock().unwrap();
	let mut suites: Vec<String> = Vec::new();
	for c in reg.iter() {
		if !suites.iter().any(|s| s == c.suite) {
			suites.push(c.suite.to_string());
		}
	}
	suites.sort();
	suites
}

/// 列出指定 suite 下的用例名
pub fn list_tests(suite: &str) -> Vec<String> {
	ensure_registered();
	let reg = registry().lock().unwrap();
	reg.iter()
		.filter(|c| c.suite == suite)
		.map(|c| c.name.to_string())
		.collect()
}

/// 按 suite 分组统计：suite -> (total, passed)
pub fn summary_by_suite(results: &[TestResult]) -> BTreeMap<String, (usize, usize)> {
	let mut map: BTreeMap<String, (usize, usize)> = BTreeMap::new();
	for r in results {
		let entry = map.entry(r.suite.clone()).or_insert((0, 0));
		entry.0 += 1;
		if r.passed {
			entry.1 += 1;
		}
	}
	map
}

fn run_cases(cases: Vec<TestCase>) -> Vec<TestResult> {
	let mut results = Vec::with_capacity(cases.len());
	for case in cases {
		let mut ctx = TestContext::new(case.suite, case.name);
		let start = Instant::now();
		// 捕获 panic：某个用例崩溃不应中断整个测试批次
		let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
			(case.func)(&mut ctx);
		}));
		let elapsed = start.elapsed().as_micros();

		let mut failures = ctx.failures;
		if let Err(panic_err) = panic_result {
			let msg = panic_message(panic_err);
			failures.push(format!("[PANIC] {msg}"));
		}

		results.push(TestResult {
			suite: case.suite.to_string(),
			name: case.name.to_string(),
			passed: failures.is_empty(),
			failures,
			elapsed_us: elapsed,
		});
	}
	results
}

fn panic_message(err: Box<dyn std::any::Any + Send>) -> String {
	if let Some(s) = err.downcast_ref::<&str>() {
		s.to_string()
	} else if let Some(s) = err.downcast_ref::<String>() {
		s.clone()
	} else {
		"未知 panic".to_string()
	}
}
