// DevTestRunner - 暴露给 GDScript 的测试运行器
// 在 Godot 引擎内触发 Rust 侧已注册的测试套件（纯逻辑用例），
// 结果以 Dictionary 数组返回，便于 GDScript 侧进一步断言或输出报告。
//
// GDScript 用法:
//   var runner := DevTestRunner.new()
//   var suites := runner.list_suites()          # PackedStringArray
//   var results := runner.run_suite("gjson")    # Array[Dictionary]
//   var report := runner.run_all_report()       # {total, passed, failed, suites, results}

use godot::builtin::{GString, PackedStringArray, VarArray, VarDictionary};
use godot::prelude::*;

use super::framework;

#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct DevTestRunner {
	base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for DevTestRunner {
	fn init(base: Base<RefCounted>) -> Self {
		Self { base }
	}
}

#[godot_api]
impl DevTestRunner {
	/// 列出全部测试套件名
	#[func]
	pub fn list_suites(&self) -> PackedStringArray {
		framework::list_suites()
			.iter()
			.map(|s| GString::from(s.as_str()))
			.collect::<PackedStringArray>()
	}

	/// 列出指定套件下的用例名
	#[func]
	pub fn list_tests(&self, suite: GString) -> PackedStringArray {
		framework::list_tests(&suite.to_string())
			.iter()
			.map(|s| GString::from(s.as_str()))
			.collect::<PackedStringArray>()
	}

	/// 运行全部套件，返回 Array[Dictionary]
	/// 每项: { suite: String, name: String, passed: bool, failures: PackedStringArray, elapsed_us: int }
	#[func]
	pub fn run_all(&self) -> VarArray {
		results_to_array(framework::run_all())
	}

	/// 运行指定套件，返回 Array[Dictionary]
	#[func]
	pub fn run_suite(&self, suite: GString) -> VarArray {
		results_to_array(framework::run_suite(&suite.to_string()))
	}

	/// 运行全部套件并返回汇总报告 Dictionary:
	/// { total: int, passed: int, failed: int, suites: Dictionary, results: Array }
	#[func]
	pub fn run_all_report(&self) -> VarDictionary {
		let results = framework::run_all();
		let total = results.len();
		let passed = results.iter().filter(|r| r.passed).count();

		let mut report = VarDictionary::new();
		report.set("total", total as i64);
		report.set("passed", passed as i64);
		report.set("failed", (total - passed) as i64);

		let mut suites = VarDictionary::new();
		for (suite, (t, p)) in framework::summary_by_suite(&results) {
			let mut s = VarDictionary::new();
			s.set("total", t as i64);
			s.set("passed", p as i64);
			suites.set(suite.as_str(), &s);
		}
		report.set("suites", &suites);
		report.set("results", &results_to_array(results));

		report
	}
}

fn results_to_array(results: Vec<framework::TestResult>) -> VarArray {
	let mut arr = VarArray::new();
	for r in results {
		let mut d = VarDictionary::new();
		d.set("suite", r.suite.clone());
		d.set("name", r.name.clone());
		d.set("passed", r.passed);
		let failures = r
			.failures
			.iter()
			.map(|s| GString::from(s.as_str()))
			.collect::<PackedStringArray>();
		d.set("failures", &failures);
		d.set("elapsed_us", r.elapsed_us as i64);
		arr.push(&d);
	}
	arr
}
