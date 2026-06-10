# 游戏主界面 GML 控制器 - 继承 GdGmlScene，处理装备栏数据渲染
# Tooltip 显示由 Rust 内置的 tooltip 属性自动处理，无需手动绑定信号
# 数据通过 GdBean 响应式绑定，GML 中 data="bean:scene_main:equip_data" 格式引用
# Bean 属性变更时自动更新绑定的 UI 节点
extends GdGmlScene

var scene_main = SUIMain.ins()
#
#func _ready() -> void:
	## 初始化 GdBean，在 load_gml() 之前确保 bean 已注册
	## GdGmlScene 的 on_notification(READY) 通过 call_deferred 延迟加载
	## 所以 _ready() 中的初始化会在 GML 加载前完成
	#load_gml()
