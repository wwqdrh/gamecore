// GdUIRichText —— 富文本组件（GML 标签 <RichText>）
//
// 游戏内手册/教程用途：解析自定义 .gdoc 文档格式（Markdown 子集 + BBCode 直通），
// 转换为 BBCode 交给引擎 RichTextLabel 渲染。支持图片（[img] 直通）、列表、
// 标题层级、引用、分隔线、围栏代码块（```bbcode 原样透传，可用 [table] 等高级标签）。
// 不用普通 Label 的原因：手册需要图文混排 + 分页滚动，textarea 类组件无法承载。
//
// .gdoc 格式（UTF-8 文本）：
//   # 标题        → 加大加粗标题（h1）
//   ## 标题       → h2 / ### → h3
//   **加粗** *斜体* `代码`   → 行内样式
//   - 项目        → 无序列表（bullet + indent；引擎无 [ul]/[li] 标签）
//   | a | b |     → markdown 表格 → [table=N]（首行加粗作表头，| --- | 分隔行跳过）
//   > 引用        → 斜体缩进强调段
//   ---           → 水平分隔线
//   [img]res://...[/img]     → 原样透传（图片；可带 width= 参数）
//   ```bbcode ... ```        → 围栏内 BBCode 原样透传（[table] 等高级用法）
//
// GML 用法：
//   <RichText name="DocView" src="res://.../xxx.gdoc"
//             font_size="15" text_color="#3a2f1e" heading_color="#7a4a1e" />
//
// 运行期 API：
//   load_doc(src) -> bool     加载并渲染 .gdoc 文件
//   set_doc_text(text)        渲染 gdoc 文本（含格式转换）
//   set_bbcode(text)          直接设置 BBCode（跳过转换）
//
// 注：src 属性构建期只记录，ready() 时才读文件（编辑器/构建期不落盘 IO）。

use godot::builtin::{GString, StringName, Color};
use godot::classes::{IRichTextLabel, RichTextLabel, FileAccess};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=RichTextLabel, tool)]
pub struct GdUIRichText {
    base: Base<RichTextLabel>,

    /// 文档路径（.gdoc）；构建期由 src 属性写入，ready() 加载
    #[export]
    doc_src: GString,
    /// 正文字号（标题在其基础上放大）
    #[export]
    base_font_size: i32,
    /// 正文颜色（引擎 default_color）
    #[export]
    text_color: Color,
    /// 标题颜色
    #[export]
    heading_color: Color,

    /// ready 时是否已按 doc_src 加载过（防重复 IO）
    loaded: bool,
}

#[godot_api]
impl IRichTextLabel for GdUIRichText {
    fn init(base: Base<RichTextLabel>) -> Self {
        Self {
            base,
            doc_src: GString::new(),
            base_font_size: 15,
            text_color: Color::from_rgb(0.24, 0.2, 0.13),
            heading_color: Color::from_rgb(0.48, 0.29, 0.12),
            loaded: false,
        }
        // 注意：init() 阶段对象未构造完，禁调 base_mut()（panic 红线）；
        // bbcode/滚动开关在 ready() 里设置
    }

    fn ready(&mut self) {
        // 先取标量再借 base（借用红线）
        let (fs, tc) = (self.base_font_size, self.text_color);
        let mut base = self.base_mut();
        // 手册页内容超长：BBCode 解析 + 滚动必需
        base.set_use_bbcode(true);
        base.set_scroll_active(true);
        base.add_theme_color_override(&StringName::from("default_color"), tc);
        base.add_theme_font_size_override(&StringName::from("normal_font_size"), fs);
        base.add_theme_font_size_override(&StringName::from("bold_font_size"), fs);
        base.add_theme_font_size_override(&StringName::from("italics_font_size"), fs);
        drop(base);
        // src 有值且从未加载 → 立即加载（tscn 直开路径）
        if !self.doc_src.is_empty() && !self.loaded {
            let src = self.doc_src.clone();
            self.loaded = self.load_doc(src);
        }
    }
}

#[godot_api]
impl GdUIRichText {
    /// 加载 .gdoc 文档并渲染。返回是否成功。
    #[func]
    pub fn load_doc(&mut self, src: GString) -> bool {
        let path = src.to_string();
        let Some(fa) = FileAccess::open(
            &GString::from(path.as_str()),
            godot::classes::file_access::ModeFlags::READ,
        ) else {
            godot_warn!("[GdUIRichText] 无法打开文档: {}", path);
            return false;
        };
        let text = fa.get_as_text().to_string();
        self.doc_src = src;
        self.loaded = true;
        self.set_doc_text(GString::from(text.as_str()));
        true
    }

    /// 渲染 gdoc 文本（Markdown 子集 + BBCode 直通 → 转换后显示）
    #[func]
    pub fn set_doc_text(&mut self, text: GString) {
        let bb = self.convert_gdoc(&text.to_string());
        self.base_mut().set_text(&GString::from(bb.as_str()));
    }

    /// 直接设置 BBCode（跳过格式转换）
    #[func]
    pub fn set_bbcode(&mut self, text: GString) {
        self.base_mut().set_text(&text);
    }

    /// 当前 BBCode（调试用）
    #[func]
    pub fn get_bbcode(&self) -> GString {
        self.base().get_text()
    }

    // ------------------------------------------------------------------
    // gdoc → BBCode 转换
    // ------------------------------------------------------------------

    fn heading_bb(&self, level: i32, text: &str) -> String {
        let size = self.base_font_size
            + match level {
                1 => 8,
                2 => 5,
                _ => 3,
            };
        let color = Self::color_hex(&self.heading_color);
        format!(
            "[b][color={}][font_size={}]{}[/font_size][/color][/b]",
            color, size, text
        )
    }

    fn color_hex(c: &Color) -> String {
        format!(
            "#{:02x}{:02x}{:02x}",
            (c.r * 255.0).round() as i32,
            (c.g * 255.0).round() as i32,
            (c.b * 255.0).round() as i32
        )
    }

    /// gdoc 全文转换
    fn convert_gdoc(&self, src: &str) -> String {
        let mut out_lines: Vec<String> = Vec::new();
        let mut in_fence = false;
        let mut fence_buf: Vec<String> = Vec::new();
        let mut list_buf: Vec<String> = Vec::new();
        let mut table_buf: Vec<String> = Vec::new();

        // 列表 → bullet + indent（Godot RichTextLabel 无 [ul]/[li] 标签，禁用）
        let flush_list = |out_lines: &mut Vec<String>, list_buf: &mut Vec<String>| {
            let had = !list_buf.is_empty();
            for item in list_buf.drain(..) {
                out_lines.push(format!("[indent]• {}[/indent]", item));
            }
            if had {
                out_lines.push(String::new());
            }
        };
        // markdown 管道表格 → [table=N]（首行加粗作表头，分隔行跳过）
        let flush_table = |out_lines: &mut Vec<String>, table_buf: &mut Vec<String>| {
            if table_buf.is_empty() {
                return;
            }
            let mut rows: Vec<Vec<String>> = Vec::new();
            for l in table_buf.drain(..) {
                let t = l.trim();
                let inner = t.trim_start_matches('|').trim_end_matches('|');
                let cells: Vec<String> = inner
                    .split('|')
                    .map(|c| convert_inline(c.trim()))
                    .collect();
                // 分隔行（| --- | --- |）
                let is_sep = !cells.is_empty()
                    && cells.iter().all(|c| {
                        let ct = c.trim();
                        !ct.is_empty() && ct.chars().all(|ch| ch == '-' || ch == ':')
                    });
                if !is_sep {
                    rows.push(cells);
                }
            }
            if let Some(first) = rows.first() {
                let ncols = first.len().max(1);
                out_lines.push(format!("[table={}]", ncols));
                for (ri, r) in rows.iter().enumerate() {
                    for c in r {
                        if ri == 0 {
                            out_lines.push(format!("[cell][b]{}[/b][/cell]", c));
                        } else {
                            out_lines.push(format!("[cell]{}[/cell]", c));
                        }
                    }
                }
                out_lines.push("[/table]".to_string());
                out_lines.push(String::new());
            }
        };

        for raw in src.lines() {
            let line = raw.trim_end();

            // 围栏（``` 开头）：bbcode 直通块
            if line.trim_start().starts_with("```") {
                if in_fence {
                    // 收 fence：内容原样输出
                    out_lines.push(fence_buf.join("\n"));
                    out_lines.push(String::new());
                    fence_buf.clear();
                    in_fence = false;
                } else {
                    flush_list(&mut out_lines, &mut list_buf);
                    flush_table(&mut out_lines, &mut table_buf);
                    in_fence = true;
                }
                continue;
            }
            if in_fence {
                fence_buf.push(line.to_string());
                continue;
            }

            let t = line.trim();
            // 空行 → 段落分隔（先冲掉块缓冲）
            if t.is_empty() {
                flush_list(&mut out_lines, &mut list_buf);
                flush_table(&mut out_lines, &mut table_buf);
                out_lines.push(String::new());
                continue;
            }
            // markdown 表格行（| 开头）→ 暂存待合并
            if t.starts_with('|') {
                flush_list(&mut out_lines, &mut list_buf);
                table_buf.push(t.to_string());
                continue;
            }
            // 标题
            if let Some(rest) = t.strip_prefix("### ") {
                flush_list(&mut out_lines, &mut list_buf);
                flush_table(&mut out_lines, &mut table_buf);
                out_lines.push(self.heading_bb(3, &convert_inline(rest)));
                continue;
            }
            if let Some(rest) = t.strip_prefix("## ") {
                flush_list(&mut out_lines, &mut list_buf);
                flush_table(&mut out_lines, &mut table_buf);
                out_lines.push(self.heading_bb(2, &convert_inline(rest)));
                continue;
            }
            if let Some(rest) = t.strip_prefix("# ") {
                flush_list(&mut out_lines, &mut list_buf);
                flush_table(&mut out_lines, &mut table_buf);
                out_lines.push(self.heading_bb(1, &convert_inline(rest)));
                continue;
            }
            // 分隔线
            if t.len() >= 3 && t.chars().all(|c| c == '-') {
                flush_list(&mut out_lines, &mut list_buf);
                flush_table(&mut out_lines, &mut table_buf);
                out_lines.push("[hr]".to_string());
                continue;
            }
            // 列表项
            if let Some(rest) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) {
                flush_table(&mut out_lines, &mut table_buf);
                list_buf.push(convert_inline(rest));
                continue;
            }
            // 引用
            if let Some(rest) = t.strip_prefix("> ") {
                flush_list(&mut out_lines, &mut list_buf);
                flush_table(&mut out_lines, &mut table_buf);
                out_lines.push(format!(
                    "[i][color=#8a7a5a]{}[/color][/i]",
                    convert_inline(rest)
                ));
                continue;
            }
            // [img] 单行直通（可带 width 参数）
            if t.starts_with("[img") {
                flush_list(&mut out_lines, &mut list_buf);
                flush_table(&mut out_lines, &mut table_buf);
                out_lines.push(t.to_string());
                continue;
            }
            // 普通段落
            flush_list(&mut out_lines, &mut list_buf);
            flush_table(&mut out_lines, &mut table_buf);
            out_lines.push(convert_inline(t));
        }
        // 收尾：未闭合的 fence / 块缓冲也输出
        if in_fence {
            out_lines.push(fence_buf.join("\n"));
        }
        flush_list(&mut out_lines, &mut list_buf);
        flush_table(&mut out_lines, &mut table_buf);
        out_lines.join("\n")
    }
}

fn find_char(chars: &[char], from: usize, target: char) -> Option<usize> {
    (from..chars.len()).find(|&i| chars[i] == target)
}

fn find_sub(chars: &[char], from: usize, pat: &[char]) -> Option<usize> {
    if from + pat.len() > chars.len() {
        return None;
    }
    (from..=chars.len() - pat.len()).find(|&i| {
        (0..pat.len()).all(|j| chars[i + j] == pat[j])
    })
}

/// 行内转换：**粗** *斜* `码`（简单成对扫描，不做嵌套优先级，手册文本足够）
fn convert_inline(s: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let n = chars.len();
    while i < n {
        if i + 1 < n && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(close) = find_sub(&chars, i + 2, &['*', '*']) {
                let inner: String = chars[i + 2..close].iter().collect();
                out.push_str(&format!("[b]{}[/b]", convert_inline(&inner)));
                i = close + 2;
                continue;
            }
        }
        if chars[i] == '*' {
            if let Some(close) = find_char(&chars, i + 1, '*') {
                let inner: String = chars[i + 1..close].iter().collect();
                out.push_str(&format!("[i]{}[/i]", convert_inline(&inner)));
                i = close + 1;
                continue;
            }
        }
        if chars[i] == '`' {
            if let Some(close) = find_char(&chars, i + 1, '`') {
                let inner: String = chars[i + 1..close].iter().collect();
                out.push_str(&format!("[code]{}[/code]", inner));
                i = close + 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}
