// GdUIFitLabel - 按文本长度自适应宽度的标签（单行）
//
// GML 用法：
//   <FitLabel name="QuestText" text="..." font_size="16" fit_width="true" fit_padding="8" />
//
// 能力：
//   - fit_width=true 时 custom_minimum_size.x 跟随文本像素宽自动增长/收缩
//     （含 fit_padding 余量），父容器（HBox/PanelContainer 等）随之撑开，
//     解决固定宽度面板里长文本溢出的问题
//   - 运行期文本变化（GDScript set text / 模板绑定）自动重新测量
//     （_process 脏检查：文本未变时零开销）
//   - fit_width=false（默认）行为等同普通 Label
//   - 鼠标穿透：纯展示组件，构建期 is_transparent_tag 白名单兜底
//
// 单行语义：自适应宽度基于文本不换行测量；需要多行请用 autowrap 的普通
// Label（固定宽度）而非本组件。
//
// demo 范本 example/demo/xiuxian/scenes/main/ui/mainhud/mainhud_quest.gml：
//   HUD 任务横幅（PanelContainer + FitLabel，任务名/指引文案长度自适应）

use godot::prelude::*;
use godot::builtin::StringName;
use godot::classes::{Engine, Font, ILabel, Label};
use godot::global::HorizontalAlignment;

#[derive(GodotClass)]
#[class(base = Label, tool)]
pub struct GdUIFitLabel {
    /// 是否按文本长度自适应宽度（custom_minimum_size.x = 文本宽 + fit_padding）
    #[export]
    fit_width: bool,
    /// 自适应宽度的额外余量（像素，给字侧留空）
    #[export]
    fit_padding: i32,

    /// 上次测量过的文本（脏检查，避免每帧重测量）
    last_text: GString,

    base: Base<Label>,
}

#[godot_api]
impl ILabel for GdUIFitLabel {
    fn init(base: Base<Label>) -> Self {
        Self {
            fit_width: false,
            fit_padding: 0,
            last_text: GString::new(),
            base,
        }
    }

    fn ready(&mut self) {
        // 首次测量（builder 已在此前应用 text/font_size 属性与主题覆盖）
        self.refit();
    }

    fn process(&mut self, _delta: f64) {
        if !self.fit_width {
            return;
        }
        // 脏检查：文本变化才重测量（编辑器 tool 下同样生效，属性面板改字即时预览）
        if !self.in_tree() {
            return;
        }
        let text = self.base().get_text();
        if text != self.last_text {
            self.measure_and_apply(&text);
        }
    }
}

#[godot_api]
impl GdUIFitLabel {
    /// 立即按当前文本重新测量（文本变化后想跳过一帧等待时可用）
    #[func]
    pub fn refit(&mut self) {
        if !self.fit_width {
            return;
        }
        let text = self.base().get_text();
        self.measure_and_apply(&text);
    }
}

impl GdUIFitLabel {
    fn in_tree(&self) -> bool {
        if Engine::singleton().is_editor_hint() {
            return true;
        }
        self.base().is_inside_tree()
    }

    fn measure_and_apply(&mut self, text: &GString) {
        self.last_text = text.clone();
        if !self.fit_width {
            return;
        }
        let font_size_name = StringName::from("font_size");
        let font_name = StringName::from("font");
        let font_size = self.base().get_theme_font_size(&font_size_name);
        let width = match self.base().get_theme_font(&font_name) {
            Some(font) => font
                .get_string_size_ex(text)
                .alignment(HorizontalAlignment::LEFT)
                .width(-1.0)
                .font_size(font_size)
                .done()
                .x,
            None => 0.0,
        };
        let mut min = self.base().get_custom_minimum_size();
        min.x = width + self.fit_padding as f32;
        self.base_mut().set_custom_minimum_size(min);
    }
}
