// ControlFlow: 控制流体系
//
// 用 enum 替代 C++ 的继承体系，4种控制流变体：
// - Start: 回到 Timeline 开头
// - End: 跳到 Timeline 末尾（终止）
// - Skip: 跳过 N 个 stage
// - Goto: 跳转到指定 stage

/// 控制流变体
#[derive(Debug, Clone)]
pub enum ControlFlow {
    /// 回到 Timeline 开头
    Start { stage_name: String },
    /// 跳到 Timeline 末尾（终止）
    End { stage_name: String },
    /// 跳过 N 个 stage
    Skip { stage_name: String, skip_count: i32 },
    /// 跳转到指定 stage
    Goto { stage_name: String, target_name: String },
}

impl ControlFlow {
    /// 从命令字符串创建控制流
    /// 支持格式：":start"、":end"、":skip:N"、":goto:name"
    pub fn create_from_string(command: &str) -> Option<ControlFlow> {
        if command == ":start" {
            Some(ControlFlow::Start { stage_name: String::new() })
        } else if command == ":end" {
            Some(ControlFlow::End { stage_name: String::new() })
        } else if let Some(rest) = command.strip_prefix(":skip:") {
            let count = rest.parse::<i32>().ok()?;
            Some(ControlFlow::Skip { stage_name: String::new(), skip_count: count })
        } else if let Some(target) = command.strip_prefix(":goto:") {
            Some(ControlFlow::Goto {
                stage_name: String::new(),
                target_name: target.to_string(),
            })
        } else {
            None
        }
    }

    pub fn set_stage_name(&mut self, name: &str) {
        match self {
            ControlFlow::Start { stage_name } => *stage_name = name.to_string(),
            ControlFlow::End { stage_name } => *stage_name = name.to_string(),
            ControlFlow::Skip { stage_name, .. } => *stage_name = name.to_string(),
            ControlFlow::Goto { stage_name, .. } => *stage_name = name.to_string(),
        }
    }

    pub fn get_stage_name(&self) -> &str {
        match self {
            ControlFlow::Start { stage_name } => stage_name,
            ControlFlow::End { stage_name } => stage_name,
            ControlFlow::Skip { stage_name, .. } => stage_name,
            ControlFlow::Goto { stage_name, .. } => stage_name,
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            ControlFlow::Start { .. } => "start",
            ControlFlow::End { .. } => "end",
            ControlFlow::Skip { .. } => "skip",
            ControlFlow::Goto { .. } => "goto",
        }
    }
}
