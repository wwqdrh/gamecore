// UI 标记语言模块
// 提供类 HTML 的声明式 UI 描述语言，用于快速构建 Godot Control 节点树
// 包含解析器（parser）、构建器（builder）、GDScript API（gdui_builder）
// 以及列表扩展节点（ui_hlist/ui_vlist/ui_grid）和辅助工具（ui_list_helper）

mod parser;
mod builder;
mod gdui_builder;
mod ui_list_helper;
mod ui_hlist;
mod ui_vlist;
mod ui_grid;
mod ui_popup_panel;
mod ui_tooltip;
mod ui_drawer;
mod ui_gml_scene;
