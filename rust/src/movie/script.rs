// 电影剧本（.movie）解析器
//
// 行式 DSL，`#` 注释，`:label` 标签，指令 + 空格分隔参数：
//
//   scene <alias> [(x,y)]              加载地图（经 MapManager.open_map）；
//                                      spawn 坐标可选（兼容保留）——玩家演出
//                                      站位统一用 player (x,y) 声明；跨图演出
//                                      结束自动回迁原地图（见 movie_player.rs）
//   actor <id> enter [(x,y)] [from (x,y)]
//                                      角色出场（按 config movie.actors 实例化）；
//                                      from 指定屏幕视野外的起点格——先落在
//                                      from，再走直线网格路径到 (x,y) 后继续
//   actor <id> exit [to (x,y)|left|right|up|down]
//                                      角色退场：to 指定格/方向词——先走出
//                                      N 格（EXIT_WALK_CELLS）到画面外再消失；
//                                      缺省则原地消失（实例化角色会被释放）
//   actor <id> teleport (x,y)          直接落格（不寻路）
//   player (x,y)                       玩家瞬移落格——显式声明玩家演出站位
//                                      （玩家位置随档持久化，scene 的 spawn
//                                      出生点可能被恢复位置覆盖；对话演出
//                                      开场用它保证两人站位靠近）
//   move <id> (x,y)                    网格移动（等走完）：地图有 find_path
//                                      （GdQuickMap）则寻路，否则直线逐格
//                                      （GdDigLayer 等协议无寻路能力）
//   camera follow <target>             跟随（target=actor id 或 player）
//   camera unfollow                    停止跟随
//   camera pan (x,y) [dur]             平移到世界坐标（等 dur，默认 1.0s）
//   camera focus <target> [dur]        聚焦目标（默认 1.0s）
//   camera unfocus                     取消聚焦
//   camera shake <amount>              震屏（fire&forget + 默认等 0.4s）
//   camera snap (x,y)                  瞬移相机
//   camera zoom <step>                 缩放档位（经 start_zoom）
//   camera reset_zoom                  复位缩放
//   camera blur <val>                  径向模糊（0 关闭）
//   say <id> [秒] <台词...>             对话框台词（等点击推进；带秒则定时继续）
//   bubble <id> <秒> <台词...>          气泡台词（等秒数）
//   wait <秒>                          等待
//   flag <名称>                        置剧情 flag（经 dialog control set_flag）
//   jump <label>                       无条件跳转
//   if <flag> jump <label>             flag 为真时跳转（经 control has_flag）
//   end                                结束演出（缺省时播完自动结束）
//
// target（camera/actor 的目标）统一是 actor id；特殊值 `player` 指向
// player_path 注入的玩家节点。
//
// 台词文本是行内剩余原文（保留内部空格/标点）；台词指令的时长写法统一为
// 裸数字（say/bubble 第一参数），`dur:秒` 前缀作为兼容别名保留。

use std::collections::HashMap;

use godot::builtin::Vector2;
use godot::builtin::Vector2i;

/// 相机动作
#[derive(Debug, Clone)]
pub enum CamAction {
    Follow { target: String },
    Unfollow,
    Pan { pos: Vector2, dur: f64 },
    Focus { target: String, dur: f64 },
    Unfocus,
    Shake { amount: f64 },
    Snap { pos: Vector2 },
    Zoom { step: i64 },
    ResetZoom,
    Blur { val: f32 },
}

/// 退场方向（走出画面用）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// 方向退场默认走出的格数（足以离开小地图取景框）
pub const EXIT_WALK_CELLS: i32 = 6;

impl Direction {
    /// 方向对应的位移格数
    pub fn offset(self) -> Vector2i {
        match self {
            Direction::Left => Vector2i::new(-EXIT_WALK_CELLS, 0),
            Direction::Right => Vector2i::new(EXIT_WALK_CELLS, 0),
            Direction::Up => Vector2i::new(0, -EXIT_WALK_CELLS),
            Direction::Down => Vector2i::new(0, EXIT_WALK_CELLS),
        }
    }

    /// 解析方向词（left/right/up/down）
    pub fn parse(s: &str) -> Option<Direction> {
        match s.trim() {
            "left" => Some(Direction::Left),
            "right" => Some(Direction::Right),
            "up" => Some(Direction::Up),
            "down" => Some(Direction::Down),
            _ => None,
        }
    }
}

/// 角色动作
#[derive(Debug, Clone)]
pub enum ActorAction {
    /// 出场：实例化角色并落位到指定格（缺省 (0,0) 由宿主自吸附）；
    /// from 有值时先落在 from 格（屏幕视野外），再走直线网格路径到
    /// cell（走出画面的进场），走完才执行后续指令
    Enter { cell: Option<Vector2i>, from: Option<Vector2i> },
    /// 退场：to 指定格则走出到 to 再消失；dir 方向词则沿方向走出
    /// EXIT_WALK_CELLS 格再消失；都缺省则原地消失（实例化角色会被释放）
    Exit { to: Option<Vector2i>, dir: Option<Direction> },
    Teleport { cell: Vector2i },
}

/// 电影指令
#[derive(Debug, Clone)]
pub enum MovieCmd {
    /// 加载地图；spawn 可选（兼容保留，玩家站位统一由 PlayerPos 声明）
    Scene { alias: String, spawn: Option<Vector2i> },
    PlayerPos { cell: Vector2i },
    Actor { id: String, action: ActorAction },
    Move { id: String, cell: Vector2i },
    Camera(CamAction),
    Say { id: String, text: String, dur: Option<f64> },
    Bubble { id: String, secs: f64, text: String },
    Wait { secs: f64 },
    SetFlag { name: String },
    Jump { label: String },
    JumpIf { flag: String, label: String },
    End,
}

/// 解析后的剧本：指令表 + 标签索引
#[derive(Debug, Clone, Default)]
pub struct MovieScript {
    pub cmds: Vec<MovieCmd>,
    pub labels: HashMap<String, usize>,
}

/// 解析剧本文本；语法错误返回 Err(行号+描述)
pub fn parse(text: &str) -> Result<MovieScript, String> {
    let mut script = MovieScript::default();
    for (idx, raw) in text.lines().enumerate() {
        let lineno = idx + 1;
        // 去注释（# 不要求行首，但台词里允许 #？台词含 # 的场景极少，
        // 统一按"行内首个 # 起为注释"处理，写台词时避开 # 即可）
        let line = match raw.find('#') {
            Some(p) => &raw[..p],
            None => raw,
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // 标签行
        if let Some(label) = line.strip_prefix(':') {
            let label = label.trim();
            if label.is_empty() {
                return Err(format!("line {}: empty label", lineno));
            }
            if script.labels.insert(label.to_string(), script.cmds.len()).is_some() {
                return Err(format!("line {}: duplicate label '{}'", lineno, label));
            }
            continue;
        }

        let cmd = parse_line(line).map_err(|e| format!("line {}: {}", lineno, e))?;
        script.cmds.push(cmd);
    }
    Ok(script)
}

/// 从串首提取 (x,y) 坐标（括号匹配，允许坐标内含空格），返回 (坐标串, 剩余)
fn take_coord(s: &str) -> Option<(String, &str)> {
    let start = s.find('(')?;
    let close = s[start..].find(')')? + start;
    Some((s[start..=close].to_string(), s[close + 1..].trim_start()))
}

/// 解析 (x,y) 形式的坐标（允许空格："( 8, 7 )"）
fn parse_vec2i(s: &str) -> Result<Vector2i, String> {
    let inner = s
        .strip_prefix('(')
        .and_then(|r| r.strip_suffix(')'))
        .ok_or_else(|| format!("bad coord '{}', expect (x,y)", s))?;
    let mut it = inner.split(',');
    let x: i32 = it
        .next()
        .ok_or_else(|| format!("missing x in '{}'", s))?
        .trim()
        .parse()
        .map_err(|_| format!("bad x in '{}'", s))?;
    let y: i32 = it
        .next()
        .ok_or_else(|| format!("missing y in '{}'", s))?
        .trim()
        .parse()
        .map_err(|_| format!("bad y in '{}'", s))?;
    Ok(Vector2i::new(x, y))
}

fn parse_vec2(s: &str) -> Result<Vector2, String> {
    let v = parse_vec2i(s)?;
    Ok(Vector2::new(v.x as f32, v.y as f32))
}

fn parse_f64(s: &str) -> Result<f64, String> {
    s.trim()
        .parse::<f64>()
        .map_err(|_| format!("bad number '{}'", s))
}

/// 解析单行指令（已去注释/trim）
fn parse_line(line: &str) -> Result<MovieCmd, String> {
    let mut tokens = line.splitn(2, char::is_whitespace);
    let head = tokens.next().unwrap_or("");
    let rest = tokens.next().unwrap_or("").trim();

    match head {
        "scene" => {
            // scene <alias> [(x,y)]：spawn 可选（兼容保留，站位统一 player 声明）
            let (alias, tail) =
                split_head(rest).ok_or_else(|| "scene: missing map alias".to_string())?;
            let spawn = match tail {
                "" => None,
                t => Some(parse_vec2i(
                    &take_coord(t).ok_or_else(|| "scene: bad spawn coord".to_string())?.0,
                )?),
            };
            Ok(MovieCmd::Scene { alias: alias.to_string(), spawn })
        }
        "actor" => {
            // actor <id> enter [(x,y)] | exit | teleport (x,y)
            let mut it = rest.splitn(2, char::is_whitespace);
            let id = it
                .next()
                .ok_or_else(|| "actor: missing actor id".to_string())?
                .to_string();
            let action_str = it.next().unwrap_or("").trim();
            let (verb, arg) = match split_head(action_str) {
                Some((v, a)) => (v, Some(a)),
                None => ("", None),
            };
            // 无参形式（`exit`）split_head 会给 Some("")，归一为 None
            let arg = arg.filter(|a| !a.trim().is_empty());
            match verb {
                "enter" => {
                    // enter [(x,y)] [from (x,y)]
                    let mut cell = None;
                    let mut from = None;
                    if let Some(a) = arg {
                        let mut remaining = a;
                        if let Some((coord, rest)) = take_coord(a) {
                            cell = Some(parse_vec2i(&coord)?);
                            remaining = rest.trim();
                        }
                        if !remaining.is_empty() {
                            let (kw, tail) = split_head(remaining)
                                .ok_or_else(|| "actor enter: bad 'from' clause".to_string())?;
                            if kw != "from" {
                                return Err(format!(
                                    "actor enter: unknown clause '{}' (expect 'from (x,y)')",
                                    kw
                                ));
                            }
                            let (coord, _) = take_coord(tail)
                                .ok_or_else(|| "actor enter: missing from (x,y)".to_string())?;
                            from = Some(parse_vec2i(&coord)?);
                        }
                    }
                    Ok(MovieCmd::Actor { id, action: ActorAction::Enter { cell, from } })
                }
                "exit" => {
                    // exit [to (x,y) | left|right|up|down]
                    let action = match arg {
                        None => ActorAction::Exit { to: None, dir: None },
                        Some(a) => {
                            if let Some((coord, _)) = take_coord(a) {
                                // 兼容裸坐标 `exit (x,y)`（等价 to (x,y)）
                                ActorAction::Exit { to: Some(parse_vec2i(&coord)?), dir: None }
                            } else {
                                let (kw, tail) = split_head(a)
                                    .ok_or_else(|| "actor exit: empty clause".to_string())?;
                                if kw == "to" {
                                    let (coord, _) = take_coord(tail)
                                        .ok_or_else(|| "actor exit: missing to (x,y)".to_string())?;
                                    ActorAction::Exit { to: Some(parse_vec2i(&coord)?), dir: None }
                                } else {
                                    let dir = Direction::parse(kw).ok_or_else(|| {
                                        format!(
                                            "actor exit: unknown clause '{}' (expect 'to (x,y)' or left/right/up/down)",
                                            kw
                                        )
                                    })?;
                                    if !tail.is_empty() {
                                        return Err(format!("actor exit: trailing '{}'", tail));
                                    }
                                    ActorAction::Exit { to: None, dir: Some(dir) }
                                }
                            }
                        }
                    };
                    Ok(MovieCmd::Actor { id, action })
                }
                "teleport" => {
                    let cell = arg.ok_or_else(|| "actor teleport: missing (x,y)".to_string())?;
                    Ok(MovieCmd::Actor { id, action: ActorAction::Teleport { cell: parse_vec2i(&take_coord(cell).ok_or_else(|| "actor teleport: bad coord".to_string())?.0)? } })
                }
                _ => Err(format!("actor: unknown action '{}' (enter/exit/teleport)", verb)),
            }
        }
        "move" => {
            // move <id> (x,y)
            let (id, tail) =
                split_head(rest).ok_or_else(|| "move: missing actor id".to_string())?;
            let (coord, _) =
                take_coord(tail).ok_or_else(|| "move: missing (x,y)".to_string())?;
            Ok(MovieCmd::Move { id: id.to_string(), cell: parse_vec2i(&coord)? })
        }
        "player" => {
            // player (x,y)：玩家瞬移落格（显式演出站位声明）
            let (coord, tail) =
                take_coord(rest).ok_or_else(|| "player: missing (x,y)".to_string())?;
            if !tail.is_empty() {
                return Err(format!("player: trailing '{}'", tail));
            }
            Ok(MovieCmd::PlayerPos { cell: parse_vec2i(&coord)? })
        }
        "camera" => {
            let (verb, arg) = match split_head(rest) {
                Some((v, a)) => (v, Some(a)),
                None => ("", None),
            };
            let cam = match verb {
                "follow" => {
                    let target = arg.ok_or_else(|| "camera follow: missing target".to_string())?;
                    CamAction::Follow { target: target.to_string() }
                }
                "unfollow" => CamAction::Unfollow,
                "pan" => {
                    let (pos, dur) = parse_pos_dur(arg, 1.0)?;
                    CamAction::Pan { pos, dur }
                }
                "focus" => {
                    let (target, dur) = parse_target_dur(arg, 1.0)?;
                    CamAction::Focus { target: target.to_string(), dur }
                }
                "unfocus" => CamAction::Unfocus,
                "shake" => {
                    let amount = parse_f64(arg.ok_or_else(|| "camera shake: missing amount")?)?;
                    CamAction::Shake { amount }
                }
                "snap" => {
                    let pos = parse_vec2(arg.ok_or_else(|| "camera snap: missing (x,y)")?)?;
                    CamAction::Snap { pos }
                }
                "zoom" => {
                    let step: i64 = parse_f64(arg.ok_or_else(|| "camera zoom: missing step")?)? as i64;
                    CamAction::Zoom { step }
                }
                "reset_zoom" => CamAction::ResetZoom,
                "blur" => {
                    let val = parse_f64(arg.ok_or_else(|| "camera blur: missing val")?)? as f32;
                    CamAction::Blur { val }
                }
                _ => return Err(format!("camera: unknown action '{}'", verb)),
            };
            Ok(MovieCmd::Camera(cam))
        }
        "say" => {
            // say <id> [秒] <台词...>（`dur:秒` 为兼容别名）
            let (id, tail) = split_head(rest)
                .ok_or_else(|| "say: missing actor id".to_string())?;
            let (dur, text) = match split_head(tail) {
                Some((first, remain)) if first.starts_with("dur:") => {
                    (Some(parse_f64(first.trim_start_matches("dur:"))?), remain.trim().to_string())
                }
                Some((first, remain)) if first.parse::<f64>().is_ok() => {
                    (Some(parse_f64(first)?), remain.trim().to_string())
                }
                _ => (None, tail.to_string()),
            };
            if text.is_empty() {
                return Err("say: missing text".to_string());
            }
            Ok(MovieCmd::Say { id: id.to_string(), text, dur })
        }
        "bubble" => {
            // bubble <id> <秒> <台词...>（`dur:秒` 为兼容别名）
            let (id, tail) = split_head(rest)
                .ok_or_else(|| "bubble: missing actor id".to_string())?;
            let (secs_str, text) = split_head(tail)
                .ok_or_else(|| "bubble: missing seconds".to_string())?;
            let secs = parse_f64(secs_str.trim_start_matches("dur:"))?;
            if text.is_empty() {
                return Err("bubble: missing text".to_string());
            }
            Ok(MovieCmd::Bubble { id: id.to_string(), secs, text: text.to_string() })
        }
        "wait" => {
            let secs = parse_f64(rest)?;
            if secs < 0.0 {
                return Err("wait: negative seconds".to_string());
            }
            Ok(MovieCmd::Wait { secs })
        }
        "flag" => {
            let name = rest
                .split_whitespace()
                .next()
                .ok_or_else(|| "flag: missing name".to_string())?;
            Ok(MovieCmd::SetFlag { name: name.to_string() })
        }
        "jump" => Ok(MovieCmd::Jump { label: rest.to_string() }),
        "if" => {
            // if <flag> jump <label>
            let mut it = rest.split_whitespace();
            let flag = it
                .next()
                .ok_or_else(|| "if: missing flag".to_string())?;
            let jump = it.next().unwrap_or("");
            if jump != "jump" {
                return Err("if: expect 'if <flag> jump <label>'".to_string());
            }
            let label = it
                .next()
                .ok_or_else(|| "if: missing jump label".to_string())?;
            Ok(MovieCmd::JumpIf { flag: flag.to_string(), label: label.to_string() })
        }
        "end" => Ok(MovieCmd::End),
        _ => Err(format!("unknown command '{}'", head)),
    }
}

/// 拆出头部 token 与剩余
fn split_head(s: &str) -> Option<(&str, &str)> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let mut it = s.splitn(2, char::is_whitespace);
    let head = it.next().unwrap_or("");
    let rest = it.next().unwrap_or("").trim();
    Some((head, rest))
}

/// "target [dur:秒|秒]"：兼容 `focus xiao_master 1.2` 与 `pan (960,540) 0.8`
/// 的可选时长后缀；dur 也接受 `dur:1.2` 前缀形式
fn parse_pos_dur(arg: Option<&str>, default_dur: f64) -> Result<(Vector2, f64), String> {
    let arg = arg.ok_or_else(|| "missing (x,y) [dur]".to_string())?;
    let (coord, rest) = take_coord(arg).ok_or_else(|| "missing (x,y)".to_string())?;
    let pos = parse_vec2(&coord)?;
    let dur = match rest {
        "" => default_dur,
        s if s.starts_with("dur:") => parse_f64(s.trim_start_matches("dur:"))?,
        s => parse_f64(s)?,
    };
    Ok((pos, dur))
}

/// "target [dur]"：target 是 token，其余是可选时长
fn parse_target_dur(arg: Option<&str>, default_dur: f64) -> Result<(&str, f64), String> {
    let arg = arg.ok_or_else(|| "missing target".to_string())?;
    let (target, rest) = split_head(arg).ok_or_else(|| "missing target".to_string())?;
    let dur = match rest {
        "" => default_dur,
        s if s.starts_with("dur:") => parse_f64(s.trim_start_matches("dur:"))?,
        s => parse_f64(s)?,
    };
    Ok((target, dur))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let src = r#"
# 开场演出
:begin
scene xiuxian_xiaozhai (9,6)
player (9,6)
actor xiao_master enter (5,7)
actor xiao_maid enter (11,7) from (22,7)
camera follow player
move xiao_master (8,7)
say xiao_master dur:2.0 你来了。# 行内注释
bubble xiao_maid 2.5 少爷早！
say xiao_maid 1.5 裸数字时长写法。
say xiao_master 点击推进不写秒数。
wait 0.5
if said_hello jump finale
flag said_hello
jump begin
camera pan (960, 540) 0.8
camera focus xiao_master 1.5
camera unfocus
camera shake 8.0
camera zoom 2
camera reset_zoom
actor xiao_master teleport (6,7)
actor xiao_master exit
actor xiao_maid exit to (22,4)
actor xiao_maid exit right
end
"#;
        let s = parse(src).expect("parse ok");
        assert_eq!(s.cmds.len(), 25);
        assert_eq!(s.labels.get("begin"), Some(&0));
        assert!(matches!(&s.cmds[0], MovieCmd::Scene { alias, spawn: Some(sp) } if alias == "xiuxian_xiaozhai" && *sp == Vector2i::new(9, 6)));
        // scene 无坐标：spawn 缺省（站位统一 player 声明）
        assert!(matches!(&parse("scene foo").unwrap().cmds[0], MovieCmd::Scene { alias, spawn: None } if alias == "foo"));
        assert!(matches!(&s.cmds[1], MovieCmd::PlayerPos { cell } if *cell == Vector2i::new(9, 6)));
        assert!(matches!(&s.cmds[2], MovieCmd::Actor { id, action: ActorAction::Enter { cell: Some(c), from: None } } if id == "xiao_master" && *c == Vector2i::new(5, 7)));
        assert!(matches!(&s.cmds[3], MovieCmd::Actor { id, action: ActorAction::Enter { cell: Some(c), from: Some(f) } } if id == "xiao_maid" && *c == Vector2i::new(11, 7) && *f == Vector2i::new(22, 7)));
        assert!(matches!(&s.cmds[6], MovieCmd::Say { id, text, dur: Some(d) } if id == "xiao_master" && text == "你来了。" && (*d - 2.0).abs() < 1e-9));
        assert!(matches!(&s.cmds[7], MovieCmd::Bubble { id, secs, text } if id == "xiao_maid" && (*secs - 2.5).abs() < 1e-9 && text == "少爷早！"));
        // say 裸数字时长（与 bubble 写法统一）+ 无时长等点击
        assert!(matches!(&s.cmds[8], MovieCmd::Say { id, text, dur: Some(d) } if id == "xiao_maid" && text == "裸数字时长写法。" && (*d - 1.5).abs() < 1e-9));
        assert!(matches!(&s.cmds[9], MovieCmd::Say { id: _, text, dur: None } if text == "点击推进不写秒数。"));
        assert!(matches!(&s.cmds[11], MovieCmd::JumpIf { flag, label } if flag == "said_hello" && label == "finale"));
        assert!(matches!(&s.cmds[14], MovieCmd::Camera(CamAction::Pan { pos, dur }) if *pos == Vector2::new(960.0, 540.0) && (*dur - 0.8).abs() < 1e-9));
        assert!(matches!(&s.cmds[21], MovieCmd::Actor { id: _, action: ActorAction::Exit { to: None, dir: None } }));
        assert!(matches!(&s.cmds[22], MovieCmd::Actor { id: _, action: ActorAction::Exit { to: Some(t), dir: None } } if *t == Vector2i::new(22, 4)));
        assert!(matches!(&s.cmds[23], MovieCmd::Actor { id: _, action: ActorAction::Exit { to: None, dir: Some(Direction::Right) } }));
        assert!(matches!(&s.cmds[24], MovieCmd::End));
    }

    #[test]
    fn test_parse_errors() {
        assert!(parse("fly me to the moon").is_err());
        assert!(parse("actor x walk").is_err());
        assert!(parse("actor x enter (1,2) at (3,4)").is_err());
        assert!(parse("actor x exit diagonal").is_err());
        assert!(parse("actor x exit to").is_err());
        assert!(parse("camera pan 3,4").is_err());
        assert!(parse(":dup\n:dup").is_err());
        assert!(parse("say xiao").is_err());
    }
}
