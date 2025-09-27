use crate::fret_dancer::FretDancerState;
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::sync::mpsc;

// 标签页枚举
#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    ParameterSetting,
    MidiInfoScan,
    ExecuteOperation,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AvatarInfo {
    pub name: String,
    pub file: String,
    pub image: String,
    pub instrument: String,
}
// 乐器调弦预设
#[derive(Clone, Debug)]
pub struct TuningPreset {
    pub name: String,
    pub notes: Vec<String>,
}
// 乐器类型枚举
#[derive(Clone, PartialEq)]
pub enum InstrumentType {
    FingerStyleGuitar,
    Bass,
    ElectricGuitar,
}

impl InstrumentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            InstrumentType::FingerStyleGuitar => "finger_style_guitar",
            InstrumentType::Bass => "bass",
            InstrumentType::ElectricGuitar => "electric_guitar",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "bass" => InstrumentType::Bass,
            "electric_guitar" => InstrumentType::ElectricGuitar,
            _ => InstrumentType::FingerStyleGuitar,
        }
    }
}

// 编辑Avatar模式枚举
#[derive(Clone, PartialEq)]
pub enum EditAvatarMode {
    New,
    Edit,
}

pub struct FretDanceApp {
    // 用户选择的参数
    pub avatar: String,
    pub midi_file_path: String,
    pub track_numbers_str: String,
    pub selected_track: i32,
    pub channel_number: i32,
    pub fps: f64,
    pub guitar_string_notes: Vec<String>,
    pub octave_down_checkbox: bool,
    pub capo_number: i32,
    pub use_harm_notes: bool,
    pub disable_barre: bool,

    // 预设调弦
    pub tuning_presets: Vec<TuningPreset>,

    // 下拉菜单选项
    pub(crate) avatar_options: Vec<String>,
    pub(crate) midi_options: Vec<String>,

    // 控制台输出
    pub(crate) console_output: String,

    // Avatar信息
    pub(crate) avatar_infos: Vec<AvatarInfo>,
    pub(crate) current_avatar_info: Option<AvatarInfo>,

    // 删除确认对话框
    pub(crate) show_delete_confirmation: bool,

    // 新增/修改Avatar对话框
    pub(crate) show_edit_avatar_dialog: bool,
    pub(crate) edit_avatar_name: String,
    pub(crate) edit_avatar_image: String,
    pub(crate) edit_avatar_selected_image_path: String,
    pub(crate) edit_avatar_json: String,
    pub(crate) edit_avatar_selected_json_path: String,
    pub(crate) edit_avatar_instrument: InstrumentType,
    pub(crate) edit_avatar_mode: EditAvatarMode,

    // 主题设置
    pub(crate) dark_mode: bool,

    // MIDI信息扫描
    pub(crate) midi_info_result: String,
    pub(crate) scanning_midi: bool,

    // 当前活动的标签页
    pub(crate) current_tab: Tab,

    // FretDancer状态，用于在操作间共享
    pub fret_dancer_state: Option<FretDancerState>,

    // 用于多线程通信的接收端
    pub output_receiver: Option<mpsc::Receiver<String>>,

    // 标记是否正在执行长时间操作
    pub is_processing: bool,
}

impl FretDanceApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 加载系统中文字体
        Self::configure_fonts(&cc.egui_ctx);

        // 安装图像加载器
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut app = Self {
            avatar: "户山香澄".to_string(),
            midi_file_path: "asset/midi/Sunburst.mid".to_string(),
            track_numbers_str: "1".to_string(),
            selected_track: 1,
            channel_number: -1,
            fps: 30.0,
            guitar_string_notes: vec![
                "e".to_string(),
                "b".to_string(),
                "G".to_string(),
                "D".to_string(),
                "A".to_string(),
                "E1".to_string(),
            ],
            octave_down_checkbox: false,
            capo_number: 0,
            use_harm_notes: false,
            disable_barre: false,
            tuning_presets: vec![
                TuningPreset {
                    name: "标准调弦 (E A D G B E)".to_string(),
                    notes: vec![
                        "e".to_string(),
                        "b".to_string(),
                        "G".to_string(),
                        "D".to_string(),
                        "A".to_string(),
                        "E1".to_string(),
                    ],
                },
                TuningPreset {
                    name: "Drop D (D A D G B E)".to_string(),
                    notes: vec![
                        "e".to_string(),
                        "b".to_string(),
                        "G".to_string(),
                        "D".to_string(),
                        "A".to_string(),
                        "D1".to_string(),
                    ],
                },
                TuningPreset {
                    name: "科庸巴巴特殊调弦 (D A D G B F)".to_string(),
                    notes: vec![
                        "f".to_string(),
                        "d".to_string(),
                        "a".to_string(),
                        "D".to_string(),
                        "A".to_string(),
                        "D1".to_string(),
                    ],
                },
                TuningPreset {
                    name: "Open D (D A D F# A D)".to_string(),
                    notes: vec![
                        "d".to_string(),
                        "a".to_string(),
                        "F#".to_string(),
                        "D".to_string(),
                        "A".to_string(),
                        "D1".to_string(),
                    ],
                },
                TuningPreset {
                    name: "Bass调弦 (E A D G)".to_string(),
                    notes: vec![
                        "G".to_string(),
                        "D".to_string(),
                        "A".to_string(),
                        "E1".to_string(),
                    ],
                },
            ],
            avatar_options: Vec::new(),
            midi_options: Vec::new(),
            avatar_infos: Vec::new(),
            current_avatar_info: None,
            show_delete_confirmation: false,
            show_edit_avatar_dialog: false,
            edit_avatar_name: String::new(),
            edit_avatar_image: String::new(),
            edit_avatar_selected_image_path: String::new(),
            edit_avatar_json: String::new(),
            edit_avatar_selected_json_path: String::new(),
            edit_avatar_instrument: InstrumentType::FingerStyleGuitar,
            edit_avatar_mode: EditAvatarMode::New,
            console_output: String::new(),
            dark_mode: true,
            midi_info_result: String::new(),
            scanning_midi: false,
            current_tab: Tab::ParameterSetting,
            fret_dancer_state: None,
            output_receiver: None,
            is_processing: false,
        };

        app.load_avatar_options();
        app.load_midi_options();
        app.load_avatar_infos();

        app
    }

    // 配置中文字体
    fn configure_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        // 从系统加载微软雅黑字体
        // 需要先将 msyh.ttc 字体文件复制到 asset/fonts/ 目录下
        fonts.font_data.insert(
            "Microsoft YaHei".to_owned(),
            egui::FontData::from_static(include_bytes!("../../asset/fonts/msyh.ttc")),
        );

        // 将微软雅黑字体设置为默认字体
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "Microsoft YaHei".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "Microsoft YaHei".to_owned());

        // 应用字体设置
        ctx.set_fonts(fonts.clone());

        // 同时设置字体大小层次
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (
                egui::TextStyle::Heading,
                egui::FontId::new(22.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::new(14.0, egui::FontFamily::Monospace),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Small,
                egui::FontId::new(12.0, egui::FontFamily::Proportional),
            ),
        ]
        .into();
        ctx.set_style(style);
    }
    /// 更新控制台输出并添加时间戳
    pub fn append_console_output(&mut self, message: &str) {
        self.console_output.push_str(message);
        self.console_output.push('\n');
    }

    /// 更新控制台输出并立即请求UI刷新
    pub fn append_console_output_with_refresh(&mut self, ctx: &egui::Context, message: &str) {
        self.append_console_output(message);
        ctx.request_repaint();
    }
}

impl eframe::App for FretDanceApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 应用当前主题
        crate::ui::theme::apply_theme(self.dark_mode, ctx);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Fret Dance");

                // 标签页导航
                ui.separator();
                if ui
                    .selectable_label(self.current_tab == Tab::ParameterSetting, "角色参数设置")
                    .clicked()
                {
                    self.current_tab = Tab::ParameterSetting;
                }
                if ui
                    .selectable_label(self.current_tab == Tab::MidiInfoScan, "MIDI参数设置")
                    .clicked()
                {
                    self.current_tab = Tab::MidiInfoScan;
                }
                if ui
                    .selectable_label(self.current_tab == Tab::ExecuteOperation, "执行操作")
                    .clicked()
                {
                    self.current_tab = Tab::ExecuteOperation;
                }

                // 将主题切换开关放在右上角
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // 添加主题切换开关
                    if ui.checkbox(&mut self.dark_mode, "暗色主题").changed() {
                        crate::ui::theme::apply_theme(self.dark_mode, ctx);
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.current_tab {
            Tab::ParameterSetting => crate::ui::parameter_setting::show_parameter_setting(self, ui),
            Tab::MidiInfoScan => crate::ui::midi_info_scan::show_midi_info_scan(self, ui),
            Tab::ExecuteOperation => crate::ui::execute_operation::show_execute_operation(self, ui),
        });
    }
}
