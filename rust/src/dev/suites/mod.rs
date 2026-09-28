// 测试套件集合
// 每个文件对应一个 suite，通过 register_all 统一注册到框架

pub mod tests_easing;
pub mod tests_gjson;
pub mod tests_map;

/// 注册全部测试套件（在 lib 初始化时调用一次）
pub fn register_all() {
	tests_gjson::register();
	tests_easing::register();
	tests_map::register();
}
