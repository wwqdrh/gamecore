// GdViewAudio - 音频管理器
// 继承 Node，提供背景音乐播放、音效播放、语音播放、音频缓存、淡入淡出功能
// 移植自 C++ manager/audio.h/audio.cpp
// 通过 AudioServer 管理音频总线，通过 ResourceLoader 加载音频资源
// 使用 Tween 实现淡入淡出效果，使用 EaseMover 跟踪音乐时钟状态

use godot::prelude::*;
use godot::builtin::{GString, Variant, VarArray, VarDictionary, Vector2, StringName, NodePath, Array};
use godot::classes::{
    INode, Node, AudioServer, AudioStream, AudioStreamPlayer, AudioStreamPlayer2D,
    ResourceLoader, Engine, Tween, node::ProcessMode,
};
use godot::global::{linear_to_db, randf_range, randomize, is_zero_approx};
use godot::obj::NewGd;

use crate::anim::easy_move::EaseMover;

/// 音频类型
pub const AUDIO_TYPE_MUSIC: i64 = 0;
pub const AUDIO_TYPE_AUDIO: i64 = 1;
pub const AUDIO_TYPE_VOICE: i64 = 2;

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GdViewAudio {
    /// 背景音乐路径列表
    #[var(pub)]
    ost: Array<GString>,

    /// 背景音乐随机等待时间范围
    #[var(pub)]
    wait_range: Vector2,

    /// 背景音乐是否循环播放
    #[var(pub)]
    bgm_loop: bool,

    /// 背景音乐别名映射
    bgm_alias: VarDictionary,

    /// 默认音频总线名称 (AudioType -> GString)
    default_buses: VarDictionary,

    /// 音频播放器池 (AudioType -> Array<AudioStreamPlayer2D>)
    audio_players: VarDictionary,

    /// 节点音频缓存 (NodePath -> AudioStreamPlayer2D)
    node_audio_cache: VarDictionary,

    /// 音频资源缓存 (alias/path -> Dictionary{resource, path, type})
    audio_cacheds: VarDictionary,

    /// 背景音乐播放器（唯一）
    bgm_player: Option<Gd<AudioStreamPlayer>>,

    /// 音乐播放队列（存储 ost 索引）
    music_que: Array<i64>,

    /// 上一首播放的歌曲索引
    last_song: i64,

    /// 等待时钟
    wait_clock: f64,

    /// 是否正在等待下一首
    waiting_next: bool,

    /// 等待时间
    wait_time: f64,

    /// 是否停止背景音乐
    stopbgm: bool,

    /// 音乐时钟状态跟踪器
    music_ease: EaseMover,

    base: Base<Node>,
}

#[godot_api]
impl INode for GdViewAudio {
    fn init(base: Base<Node>) -> Self {
        let mut default_buses = VarDictionary::new();
        default_buses.set(&AUDIO_TYPE_MUSIC.to_variant(), &GString::from("Music").to_variant());
        default_buses.set(&AUDIO_TYPE_AUDIO.to_variant(), &GString::from("Audio").to_variant());
        default_buses.set(&AUDIO_TYPE_VOICE.to_variant(), &GString::from("Voice").to_variant());

        let mut audio_players = VarDictionary::new();
        audio_players.set(&AUDIO_TYPE_MUSIC.to_variant(), &VarArray::new().to_variant());
        audio_players.set(&AUDIO_TYPE_AUDIO.to_variant(), &VarArray::new().to_variant());
        audio_players.set(&AUDIO_TYPE_VOICE.to_variant(), &VarArray::new().to_variant());

        Self {
            ost: Array::new(),
            wait_range: Vector2::new(2.0, 5.0),
            bgm_loop: false,
            bgm_alias: VarDictionary::new(),
            default_buses,
            audio_players,
            node_audio_cache: VarDictionary::new(),
            audio_cacheds: VarDictionary::new(),
            bgm_player: None,
            music_que: Array::new(),
            last_song: -1,
            wait_clock: 4.0,
            waiting_next: false,
            wait_time: 10.0,
            stopbgm: true,
            music_ease: EaseMover::new(1.0, -1.0, Vector2::ZERO, Vector2::ZERO, None),
            base,
        }
    }

    fn ready(&mut self) {
        self.base_mut().set_process_mode(ProcessMode::ALWAYS);

        self.setup_audio_buses();
        self.load_node_audios();

        // 创建背景音乐播放器
        let mut bgm_player = AudioStreamPlayer::new_alloc();
        self.base_mut().add_child(&bgm_player);

        // 连接 finished 信号
        let callable = Callable::from_object_method(&*self.base_mut(), "on_bgm_finished");
        let _ = bgm_player.connect("finished", &callable);

        self.bgm_player = Some(bgm_player);

        randomize();
    }

    fn physics_process(&mut self, delta: f64) {
        if self.stopbgm {
            return;
        }

        // 处理等待时钟
        if self.waiting_next && self.wait_clock > 0.0 {
            self.wait_clock -= delta;
            if self.wait_clock <= 0.0 {
                self.play_bgm_random_public(1.0);
            }
        }

        // 处理音乐时钟状态
        let _s = self.music_ease.count(delta, true, true);
        if !self.music_ease.is_last() {
            if let Some(ref bgm_player) = self.bgm_player.clone() {
                if bgm_player.is_instance_valid() {
                    let mut bgm_player = bgm_player.clone();
                    if is_zero_approx(self.music_ease.clock) {
                        bgm_player.stop();
                    } else if is_zero_approx(self.music_ease.last) && !bgm_player.is_playing() {
                        let pos = bgm_player.get_playback_position();
                        bgm_player.play_ex().from_position(pos).done();
                    }
                }
            }
        }
    }
}

#[godot_api]
impl GdViewAudio {
    /// 音频类型常量 - 音乐
    #[constant]
    const AUDIO_TYPE_MUSIC: i64 = AUDIO_TYPE_MUSIC;

    /// 音频类型常量 - 音效
    #[constant]
    const AUDIO_TYPE_AUDIO: i64 = AUDIO_TYPE_AUDIO;

    /// 音频类型常量 - 语音
    #[constant]
    const AUDIO_TYPE_VOICE: i64 = AUDIO_TYPE_VOICE;

    /// 预加载音频资源
    /// alias: 音频别名
    /// path: 音频路径
    /// audio_type: 音频类型
    #[func]
    fn preload_audio(&mut self, alias: GString, path: GString, audio_type: i64) {
        let path_var = path.to_variant();
        if !self.audio_cacheds.contains_key(&path_var) {
            if let Some(resource) = ResourceLoader::singleton().load(&path) {
                if let Ok(audio_stream) = resource.try_cast::<AudioStream>() {
                    let mut conf = VarDictionary::new();
                    conf.set(&"resource".to_variant(), &audio_stream.to_variant());
                    conf.set(&"path".to_variant(), &path_var);
                    conf.set(&"type".to_variant(), &audio_type.to_variant());
                    self.audio_cacheds.set(&alias.to_variant(), &conf.to_variant());
                }
            }
        }
    }

    /// 播放背景音乐
    /// alias: 音频别名
    /// fade_duration: 淡入持续时间
    /// loop: 是否循环播放
    #[func]
    fn play_bgm(&mut self, alias: GString, fade_duration: f32, loop_: bool) {
        let bgm_player = self.bgm_player.clone();
        if let Some(ref bgm_player) = bgm_player {
            if bgm_player.is_instance_valid() && bgm_player.is_playing() {
                self.fade_out_music(fade_duration);
            }
        }

        let res = self.get_audio_resource(alias.clone());
        if res.is_none() {
            return;
        }
        let res = res.unwrap();

        self.stopbgm = false;
        let bus_name = self.get_bus_name(AUDIO_TYPE_MUSIC);
        if let Some(ref mut bgm_player) = self.bgm_player {
            if bgm_player.is_instance_valid() {
                // 使用 call 设置 stream（ByOption 类型不匹配）
                bgm_player.call("set_stream", &[res.to_variant()]);
                bgm_player.set_bus(&StringName::from(&bus_name));
                bgm_player.play();
            }
        }

        if fade_duration > 0.0 {
            self.fade_in_music(fade_duration);
        }
    }

    /// 随机播放背景音乐
    /// fade_duration: 淡入持续时间
    #[func]
    fn play_bgm_random(&mut self, fade_duration: f32) {
        self.waiting_next = false;

        // 如果音乐队列为空，重新填充
        if self.music_que.len() == 0 {
            self.music_que = Array::new();
            for i in 0..self.ost.len() {
                self.music_que.push(i as i64);
            }
            if self.ost.len() > 1 {
                // 移除上一首播放的歌曲
                if self.last_song >= 0 && self.last_song < self.ost.len() as i64 {
                    let mut new_que: Array<i64> = Array::new();
                    for i in 0..self.music_que.len() {
                        let idx = self.music_que.at(i);
                        if idx != self.last_song {
                            new_que.push(idx);
                        }
                    }
                    self.music_que = new_que;
                }
                // 洗牌
                self.music_que.shuffle();
            }
        }

        // 播放下一首
        if self.music_que.len() > 0 {
            self.last_song = self.music_que.at(0);
            // 移除第一个元素
            let mut new_que: Array<i64> = Array::new();
            for i in 1..self.music_que.len() {
                new_que.push(self.music_que.at(i));
            }
            self.music_que = new_que;

            if self.last_song >= 0 && self.last_song < self.ost.len() as i64 {
                let ost_path = self.ost.at(self.last_song as usize);
                let res = self.get_audio_resource(ost_path);
                if res.is_none() {
                    return;
                }
                let res = res.unwrap();

                let bgm_player = self.bgm_player.clone();
                let is_playing = bgm_player.as_ref().map_or(false, |p| {
                    p.is_instance_valid() && p.is_playing()
                });

                if is_playing {
                    self.fade_out_and_switch(fade_duration, res);
                } else {
                    let bus_name = self.get_bus_name(AUDIO_TYPE_MUSIC);
                    if let Some(ref mut bgm_player) = self.bgm_player {
                        if bgm_player.is_instance_valid() {
                            bgm_player.set_bus(&StringName::from(&bus_name));
                            bgm_player.call("set_stream", &[res.to_variant()]);
                            bgm_player.play();
                            self.stopbgm = false;
                            if fade_duration > 0.0 {
                                self.fade_in_music(fade_duration);
                            }
                        }
                    }
                }
            }
        }
    }

    /// 播放音效
    /// alias: 音频别名
    /// volume: 音量
    /// 返回音频播放器
    #[func]
    fn play_audio(&mut self, alias: GString, volume: f32) -> Option<Gd<AudioStreamPlayer2D>> {
        let audio_resource = self.get_audio_resource(alias.clone());
        if let Some(resource) = audio_resource {
            let mut player = self.get_audio_player(AUDIO_TYPE_AUDIO);
            player.call("set_stream", &[resource.to_variant()]);
            let bus_name = self.get_bus_name(AUDIO_TYPE_AUDIO);
            player.set_bus(&StringName::from(&bus_name));
            player.play();
            return Some(player);
        } else if self.node_audio_cache.contains_key(&alias.to_variant()) {
            let bus_name = self.get_bus_name(AUDIO_TYPE_AUDIO);
            return self.play_node_audio(alias.to_variant(), bus_name, 1.0, -1.0, 0.0);
        }
        None
    }

    /// 播放语音
    /// alias: 音频别名
    /// volume: 音量
    /// 返回音频播放器
    #[func]
    fn play_voice(&mut self, alias: GString, volume: f32) -> Option<Gd<AudioStreamPlayer2D>> {
        let audio_resource = self.get_audio_resource(alias.clone());
        if let Some(resource) = audio_resource {
            let mut player = self.get_audio_player(AUDIO_TYPE_VOICE);
            player.call("set_stream", &[resource.to_variant()]);
            let bus_name = self.get_bus_name(AUDIO_TYPE_VOICE);
            player.set_bus(&StringName::from(&bus_name));
            player.play();
            return Some(player);
        } else if self.node_audio_cache.contains_key(&alias.to_variant()) {
            let bus_name = self.get_bus_name(AUDIO_TYPE_VOICE);
            return self.play_node_audio(alias.to_variant(), bus_name, 1.0, -1.0, 0.0);
        }
        None
    }

    /// 停止背景音乐
    #[func]
    fn stop_bgm(&mut self) {
        self.stopbgm = true;
        if let Some(ref mut bgm_player) = self.bgm_player {
            if bgm_player.is_instance_valid() {
                bgm_player.stop();
            }
        }
    }

    /// 背景音乐播放结束回调
    #[func]
    fn on_bgm_finished(&mut self) {
        if !self.bgm_loop {
            return;
        }
        self.wait_clock = randf_range(self.wait_range.x as f64, self.wait_range.y as f64);
        self.waiting_next = true;
    }

    /// 停止所有音频
    #[func]
    fn stop_all(&mut self) {
        if let Some(ref mut bgm_player) = self.bgm_player {
            if bgm_player.is_instance_valid() {
                bgm_player.stop();
            }
        }

        // 停止所有播放器池中的播放器
        let audio_types = [AUDIO_TYPE_MUSIC, AUDIO_TYPE_AUDIO, AUDIO_TYPE_VOICE];
        for audio_type in audio_types {
            let players_var = self.audio_players.get_or_nil(&audio_type.to_variant());
            if players_var.is_nil() {
                continue;
            }
            let players: VarArray = players_var.to();
            for i in 0..players.len() {
                let player_var = players.at(i);
                if let Ok(mut player) = player_var.try_to::<Gd<AudioStreamPlayer2D>>() {
                    if player.is_instance_valid() {
                        player.stop();
                    }
                }
            }
        }
    }

    /// 播放节点音频
    /// arg: 节点路径或节点引用
    /// bus: 音频总线名称
    /// from: 音高起始值（或固定值当 to < 0）
    /// to: 音高结束值（当 to < 0 时使用 from 作为固定值）
    /// pos: 播放起始位置
    #[func]
    fn play_node_audio(
        &mut self,
        arg: Variant,
        bus: GString,
        from: f64,
        to: f64,
        pos: f64,
    ) -> Option<Gd<AudioStreamPlayer2D>> {
        let mut node_arg = arg.clone();

        // 如果参数是字符串，从缓存中查找对应的音频播放器
        if arg.get_type() == godot::builtin::VariantType::STRING {
            let arg_str: GString = arg.to();
            let cached = self.node_audio_cache.get_or_nil(&arg_str.to_variant());
            if cached.is_nil() {
                return None;
            }
            node_arg = cached;
        }

        // 尝试转换为 AudioStreamPlayer2D
        if let Ok(mut asp2d) = node_arg.try_to::<Gd<AudioStreamPlayer2D>>() {
            if asp2d.is_instance_valid() {
                // 设置音高
                if to < 0.0 {
                    asp2d.set_pitch_scale(from as f32);
                } else {
                    asp2d.set_pitch_scale(randf_range(from, to) as f32);
                }

                // 播放音频
                asp2d.set_bus(&StringName::from(&bus));
                asp2d.play_ex().from_position(pos as f32).done();
                return Some(asp2d);
            }
        }

        None
    }

    /// 切换到新歌曲（由 tween_callback 调用）
    #[func]
    fn switch_to_new_song(&mut self, new_stream: Gd<AudioStream>, fade_duration: f64) {
        let bus_name = self.get_bus_name(AUDIO_TYPE_MUSIC);
        if let Some(ref mut bgm_player) = self.bgm_player {
            if bgm_player.is_instance_valid() {
                // 停止当前音乐
                bgm_player.stop();

                // 设置并播放新音乐
                bgm_player.set_bus(&StringName::from(&bus_name));
                bgm_player.call("set_stream", &[new_stream.to_variant()]);
                bgm_player.set_volume_db(-80.0);
                bgm_player.play();
                self.stopbgm = false;

                // 开始淡入
                if fade_duration > 0.0 {
                    self.fade_in_music(fade_duration as f32);
                }
            }
        }
    }
}

/// 私有方法实现
impl GdViewAudio {
    /// 加载子节点中的音频播放器到缓存
    fn load_node_audios(&mut self) {
        self.node_audio_cache.clear();

        let children = self.get_all_children();
        for child in children {
            // 检查是否是 AudioStreamPlayer 或 AudioStreamPlayer2D
            let class_name = child.get_class();
            let class_str = class_name.to_string();
            if class_str == "AudioStreamPlayer" || class_str == "AudioStreamPlayer2D" {
                let path = self.base().get_path_to(&child);
                self.node_audio_cache.set(&path.to_variant(), &child.to_variant());
            }
        }
    }

    /// 递归获取所有子节点
    fn get_all_children(&self) -> Vec<Gd<Node>> {
        let mut result = Vec::new();
        let child_count = self.base().get_child_count();
        for i in 0..child_count {
            if let Some(child) = self.base().get_child(i) {
                result.push(child.clone());
                // 递归获取子节点的子节点
                let sub_children = Self::get_all_children_recursive(&child);
                result.extend(sub_children);
            }
        }
        result
    }

    /// 递归辅助函数
    fn get_all_children_recursive(node: &Gd<Node>) -> Vec<Gd<Node>> {
        let mut result = Vec::new();
        let child_count = node.get_child_count();
        for i in 0..child_count {
            if let Some(child) = node.get_child(i) {
                result.push(child.clone());
                result.extend(Self::get_all_children_recursive(&child));
            }
        }
        result
    }

    /// 获取音频资源
    /// path: 音频路径或别名
    fn get_audio_resource(&mut self, path: GString) -> Option<Gd<AudioStream>> {
        let path_var = path.to_variant();

        // 先从缓存中查找
        if self.audio_cacheds.contains_key(&path_var) {
            let conf_var = self.audio_cacheds.get_or_nil(&path_var);
            let conf: VarDictionary = conf_var.to();
            let resource_var = conf.get_or_nil(&"resource".to_variant());
            if let Ok(resource) = resource_var.try_to::<Gd<AudioStream>>() {
                return Some(resource);
            }
        }

        // 使用 ResourceLoader 加载
        let mut loader = ResourceLoader::singleton();
        if loader.exists(&path) {
            if let Some(resource) = loader.load(&path) {
                if let Ok(audio_stream) = resource.try_cast::<AudioStream>() {
                    let mut conf = VarDictionary::new();
                    conf.set(&"resource".to_variant(), &audio_stream.to_variant());
                    conf.set(&"type".to_variant(), &AUDIO_TYPE_AUDIO.to_variant());
                    conf.set(&"path".to_variant(), &path_var);
                    self.audio_cacheds.set(&path_var, &conf.to_variant());
                    return Some(audio_stream);
                }
            }
        }

        None
    }

    /// 获取音频播放器（从池中查找空闲的或创建新的）
    fn get_audio_player(&mut self, audio_type: i64) -> Gd<AudioStreamPlayer2D> {
        let players_var = self.audio_players.get_or_nil(&audio_type.to_variant());
        let players: VarArray = players_var.to();

        // 查找空闲的播放器
        for i in 0..players.len() {
            let player_var = players.at(i);
            if let Ok(player) = player_var.try_to::<Gd<AudioStreamPlayer2D>>() {
                if !player.is_playing() {
                    return player;
                }
            }
        }

        // 创建新的播放器
        let mut new_player = AudioStreamPlayer2D::new_alloc();
        self.base_mut().add_child(&new_player);

        // 添加到播放器池
        let mut new_players: VarArray = VarArray::new();
        for i in 0..players.len() {
            new_players.push(&players.at(i));
        }
        new_players.push(&new_player.to_variant());
        self.audio_players.set(&audio_type.to_variant(), &new_players.to_variant());

        new_player
    }

    /// 淡出音乐
    fn fade_out_music(&mut self, duration: f32) {
        let bgm_player = self.bgm_player.clone();
        if let Some(bgm_player) = bgm_player {
            if bgm_player.is_instance_valid() {
                let mut tween = self.base_mut().create_tween();
                tween.tween_property(
                    &bgm_player.upcast::<godot::classes::Object>(),
                    &NodePath::from("volume_db"),
                    &(-80.0f32).to_variant(),
                    duration as f64,
                );
            }
        }
    }

    /// 淡入音乐
    fn fade_in_music(&mut self, duration: f32) {
        let bgm_player = self.bgm_player.clone();
        if let Some(mut bgm_player) = bgm_player {
            if bgm_player.is_instance_valid() {
                // 获取当前配置的音量
                let target_volume_db = self.get_current_conf_music_db("Music");

                // 确保从静音开始淡入
                bgm_player.set_volume_db(-80.0);

                // 创建淡入 tween
                let mut tween = self.base_mut().create_tween();
                tween.tween_property(
                    &bgm_player.upcast::<godot::classes::Object>(),
                    &NodePath::from("volume_db"),
                    &(target_volume_db as f32).to_variant(),
                    duration as f64,
                );
            }
        }
    }

    /// 淡出并切换到新音乐
    fn fade_out_and_switch(&mut self, duration: f32, new_stream: Gd<AudioStream>) {
        let bgm_player = self.bgm_player.clone();
        if bgm_player.is_none() {
            return;
        }
        let bgm_player = bgm_player.unwrap();
        if !bgm_player.is_instance_valid() {
            return;
        }

        let mut tween = self.base_mut().create_tween();

        // 先淡出当前音乐
        tween.tween_property(
            &bgm_player.upcast::<godot::classes::Object>(),
            &NodePath::from("volume_db"),
            &(-80.0f32).to_variant(),
            duration as f64,
        );

        // 淡出完成后切换新音乐
        let callable = Callable::from_object_method(&*self.base_mut(), "switch_to_new_song")
            .bind(&[new_stream.to_variant(), (duration as f64).to_variant()]);
        tween.tween_callback(&callable);
    }

    /// 设置音频总线
    fn setup_audio_buses(&self) {
        let mut audio_server = AudioServer::singleton();
        let audio_types = [AUDIO_TYPE_MUSIC, AUDIO_TYPE_AUDIO, AUDIO_TYPE_VOICE];

        for audio_type in audio_types {
            let bus_name_var = self.default_buses.get_or_nil(&audio_type.to_variant());
            if bus_name_var.is_nil() {
                continue;
            }
            let bus_name: GString = bus_name_var.to();
            let bus_name_sn = StringName::from(&bus_name);

            if audio_server.get_bus_index(&bus_name_sn) == -1 {
                audio_server.add_bus();
                let bus_idx = audio_server.get_bus_count() - 1;
                audio_server.set_bus_name(bus_idx, &bus_name);
                audio_server.set_bus_volume_db(bus_idx, linear_to_db(1.0) as f32);
            }
        }
    }

    /// 获取当前配置的音乐音量（分贝）
    fn get_current_conf_music_db(&self, mode: &str) -> f64 {
        let audio_server = AudioServer::singleton();
        let mode_sn = StringName::from(mode);
        let bus_idx = audio_server.get_bus_index(&mode_sn);
        if bus_idx >= 0 {
            audio_server.get_bus_volume_db(bus_idx) as f64
        } else {
            0.0
        }
    }

    /// 获取总线名称
    fn get_bus_name(&self, audio_type: i64) -> GString {
        let bus_var = self.default_buses.get_or_nil(&audio_type.to_variant());
        if bus_var.is_nil() {
            GString::from("Master")
        } else {
            bus_var.to()
        }
    }

    /// play_bgm_random 的公开包装方法（供 physics_process 内部调用）
    fn play_bgm_random_public(&mut self, fade_duration: f32) {
        self.play_bgm_random(fade_duration);
    }
}
