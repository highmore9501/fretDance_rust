use eframe::egui;
use serde::{Deserialize, Serialize};

// 标签页枚举
#[derive(Clone, Copy, PartialEq)]
enum Tab {
    ParameterSetting,
    MidiInfoScan,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AvatarInfo {
    pub name: String,
    pub file: String,
    pub image: String,
    pub instrument: String,
}

pub struct FretDanceApp {
    // 用户选择的参数
    pub avatar: String,
    pub midi_file_path: String,
    pub track_numbers_str: String,
    pub channel_number: i32,
    pub fps: f64,
    pub guitar_string_notes: [String; 6],
    pub octave_down_checkbox: bool,
    pub capo_number: i32,
    pub use_harm_notes: bool,

    // 下拉菜单选项
    pub(crate) avatar_options: Vec<String>,
    pub(crate) midi_options: Vec<String>,

    // 控制台输出
    pub(crate) console_output: String,

    // Avatar信息
    pub(crate) avatar_infos: Vec<AvatarInfo>,
    pub(crate) current_avatar_info: Option<AvatarInfo>,

    // 主题设置
    pub(crate) dark_mode: bool,

    // MIDI信息扫描
    pub(crate) midi_info_result: String,
    pub(crate) scanning_midi: bool,

    // 当前活动的标签页
    pub(crate) current_tab: Tab,
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
            channel_number: -1,
            fps: 30.0,
            guitar_string_notes: [
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
            avatar_options: Vec::new(),
            midi_options: Vec::new(),
            avatar_infos: Vec::new(),
            current_avatar_info: None,
            console_output: String::new(),
            dark_mode: true,
            midi_info_result: String::new(),
            scanning_midi: false,
            current_tab: Tab::ParameterSetting,
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
        ctx.set_fonts(fonts);
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
                    .selectable_label(self.current_tab == Tab::ParameterSetting, "参数设置")
                    .clicked()
                {
                    self.current_tab = Tab::ParameterSetting;
                }
                if ui
                    .selectable_label(self.current_tab == Tab::MidiInfoScan, "MIDI信息扫描")
                    .clicked()
                {
                    self.current_tab = Tab::MidiInfoScan;
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
        });
    }
}
