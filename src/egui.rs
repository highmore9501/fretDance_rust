use crate::midi::midi_to_note::MidiProcessor;
use eframe::egui;
use std::fs;
use std::path::Path;

// 标签页枚举
#[derive(Clone, Copy, PartialEq)]
enum Tab {
    ParameterSetting,
    MidiInfoScan,
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
    avatar_options: Vec<String>,
    midi_options: Vec<String>,

    // 控制台输出
    console_output: String,

    // 主题设置
    dark_mode: bool,

    // MIDI信息扫描
    midi_info_result: String,
    scanning_midi: bool,

    // 当前活动的标签页
    current_tab: Tab,
}

impl FretDanceApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 加载系统中文字体
        Self::configure_fonts(&cc.egui_ctx);

        let mut app = Self {
            avatar: "户山香澄_E".to_string(),
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
            console_output: String::new(),
            dark_mode: true,
            midi_info_result: String::new(),
            scanning_midi: false,
            current_tab: Tab::ParameterSetting,
        };

        app.load_avatar_options();
        app.load_midi_options();

        app
    }

    // 配置中文字体
    fn configure_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        // 从系统加载微软雅黑字体
        // 需要先将 msyh.ttc 字体文件复制到 asset/fonts/ 目录下
        fonts.font_data.insert(
            "Microsoft YaHei".to_owned(),
            egui::FontData::from_static(include_bytes!("../asset/fonts/msyh.ttc")),
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

    // 应用主题
    fn apply_theme(&self, ctx: &egui::Context) {
        if self.dark_mode {
            self.set_dark_theme(ctx);
        } else {
            self.set_light_theme(ctx);
        }
    }

    // 设置黑色主题
    fn set_dark_theme(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        // 设置暗色主题的基本颜色
        style.visuals = egui::Visuals::dark();

        // 自定义颜色 - 黑色背景和白色文字
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(0, 0, 0); // 窗口背景 - 纯黑
        style.visuals.panel_fill = egui::Color32::from_rgb(20, 20, 20); // 面板背景 - 深灰黑
        style.visuals.window_fill = egui::Color32::from_rgb(15, 15, 15); // 窗口填充 - 更深的灰黑
        style.visuals.window_stroke =
            egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)); // 窗口边框

        // 文字颜色设置为白色
        style.visuals.override_text_color = Some(egui::Color32::from_rgb(255, 255, 255)); // 主要文字 - 纯白
        style.visuals.widgets.noninteractive.fg_stroke.color =
            egui::Color32::from_rgb(255, 255, 255); // 非交互部件文字
        style.visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255); // 非活动部件文字
        style.visuals.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255); // 悬停部件文字
        style.visuals.widgets.active.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255); // 活动部件文字
        style.visuals.widgets.open.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255); // 打开部件文字

        // 背景颜色保持深色
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 30, 30); // 非交互部件背景
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(40, 40, 40); // 非活动部件背景
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(60, 60, 60); // 悬停部件背景
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(50, 50, 50); // 活动部件背景
        style.visuals.widgets.open.bg_fill = egui::Color32::from_rgb(50, 50, 50); // 打开部件背景

        // 滚动条颜色
        style.visuals.widgets.noninteractive.bg_stroke.color = egui::Color32::from_rgb(80, 80, 80);

        // 选中项颜色
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(70, 70, 70);
        style.visuals.selection.stroke.color = egui::Color32::WHITE;

        // 分隔线颜色
        style.visuals.widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100));

        // 应用样式
        ctx.set_style(style);
    }

    // 设置浅色主题
    fn set_light_theme(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        // 使用默认的浅色主题
        style.visuals = egui::Visuals::light();

        // 应用样式
        ctx.set_style(style);
    }
    fn load_avatar_options(&mut self) {
        let controller_info_path = "asset/controller_infos";
        if let Ok(entries) = fs::read_dir(controller_info_path) {
            self.avatar_options = entries
                .filter_map(|entry| {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("json") {
                            if let Some(file_name) = path.file_stem() {
                                return Some(file_name.to_string_lossy().to_string());
                            }
                        }
                    }
                    None
                })
                .collect();

            if !self.avatar_options.is_empty() && self.avatar.is_empty() {
                self.avatar = self.avatar_options[0].clone();
            }
        }
    }

    fn load_midi_options(&mut self) {
        let midi_path = "asset/midi";
        if let Ok(entries) = fs::read_dir(midi_path) {
            self.midi_options = entries
                .filter_map(|entry| {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("mid") {
                            if let Some(file_name) = path.file_name() {
                                return Some(file_name.to_string_lossy().to_string());
                            }
                        }
                    }
                    None
                })
                .collect();

            if !self.midi_options.is_empty() && self.midi_file_path.is_empty() {
                self.midi_file_path = format!("asset/midi/{}", self.midi_options[0]);
            }
        }
    }

    fn parse_track_numbers(&self) -> Vec<i32> {
        self.track_numbers_str
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect()
    }

    fn scan_midi_info(&mut self) {
        // 设置扫描状态
        self.scanning_midi = true;
        self.midi_info_result = String::new();

        // 获取MIDI文件名（不含路径和扩展名）
        let midi_file_name = Path::new(&self.midi_file_path)
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // 在后台线程中执行扫描操作，避免阻塞UI
        let midi_file_name_clone = midi_file_name.clone();
        let result = std::thread::spawn(move || {
            // 创建MidiProcessor实例
            let midi_processor = MidiProcessor::new();

            // 调用export_midi_info方法
            match midi_processor.export_midi_info(&midi_file_name_clone) {
                Ok(result) => Ok(result),
                Err(e) => Err(format!("扫描MIDI信息时出错: {}", e)),
            }
        });

        // 获取结果
        match result.join() {
            Ok(Ok(info)) => {
                self.midi_info_result = info;
            }
            Ok(Err(e)) => {
                self.midi_info_result = e;
            }
            Err(_) => {
                self.midi_info_result = "扫描过程中发生未知错误".to_string();
            }
        }

        // 重置扫描状态
        self.scanning_midi = false;
    }

    fn show_parameter_setting(&mut self, ui: &mut egui::Ui) {
        // 创建一个滚动区域以容纳所有控件
        egui::ScrollArea::vertical().show(ui, |ui| {
            // 第一部分：用户输入参数
            ui.group(|ui| {
                ui.heading("1. 参数设置");
                ui.separator();

                // Avatar选择
                ui.horizontal(|ui| {
                    ui.label("Avatar:");
                    egui::ComboBox::from_id_source("avatar_select")
                        .selected_text(&self.avatar)
                        .show_ui(ui, |ui| {
                            for option in &self.avatar_options {
                                ui.selectable_value(&mut self.avatar, option.clone(), option);
                            }
                        });
                });

                // MIDI文件选择
                ui.horizontal(|ui| {
                    ui.label("MIDI文件:");
                    egui::ComboBox::from_id_source("midi_select")
                        .selected_text({
                            // 显示文件名而不是完整路径
                            Path::new(&self.midi_file_path)
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or(&self.midi_file_path)
                                .to_string()
                        })
                        .show_ui(ui, |ui| {
                            for option in &self.midi_options {
                                let full_path = format!("asset/midi/{}", option);
                                ui.selectable_value(&mut self.midi_file_path, full_path, option);
                            }
                        });
                });

                // Track Numbers输入
                ui.horizontal(|ui| {
                    ui.label("轨道号 (逗号分隔):");
                    ui.text_edit_singleline(&mut self.track_numbers_str);
                });

                // Channel Number输入
                ui.horizontal(|ui| {
                    ui.label("通道号:");
                    ui.add(egui::DragValue::new(&mut self.channel_number));
                });

                // FPS输入
                ui.horizontal(|ui| {
                    ui.label("FPS:");
                    ui.add(egui::DragValue::new(&mut self.fps).speed(1.0));
                });

                // 吉他弦音高设置
                ui.label("吉他弦音高 (从最细的弦到最粗的弦):");
                for (i, note) in self.guitar_string_notes.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        let string_names = ["1弦", "2弦", "3弦", "4弦", "5弦", "6弦"];
                        ui.label(string_names[i]);
                        ui.text_edit_singleline(note);
                    });
                }

                // Octave down checkbox
                ui.checkbox(&mut self.octave_down_checkbox, "降低八度");

                // Capo number输入
                ui.horizontal(|ui| {
                    ui.label("变调夹位置:");
                    ui.add(egui::DragValue::new(&mut self.capo_number).range(0..=12));
                });

                // Use harm notes checkbox
                ui.checkbox(&mut self.use_harm_notes, "使用泛音");
            });

            ui.add_space(10.0);

            // 控制台输出部分
            ui.group(|ui| {
                ui.heading("2. 控制台输出");
                ui.separator();
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        ui.monospace(&self.console_output);
                    });
            });
        });
    }

    fn show_midi_info_scan(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.group(|ui| {
                ui.heading("MIDI信息扫描");
                ui.separator();

                // MIDI文件选择
                ui.horizontal(|ui| {
                    ui.label("MIDI文件:");
                    egui::ComboBox::from_id_source("midi_scan_select")
                        .selected_text({
                            // 显示文件名而不是完整路径
                            Path::new(&self.midi_file_path)
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or(&self.midi_file_path)
                                .to_string()
                        })
                        .show_ui(ui, |ui| {
                            for option in &self.midi_options {
                                let full_path = format!("asset/midi/{}", option);
                                ui.selectable_value(&mut self.midi_file_path, full_path, option);
                            }
                        });
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("扫描MIDI信息").clicked() {
                        self.scan_midi_info();
                    }

                    if self.scanning_midi {
                        ui.spinner();
                        ui.label("扫描中...");
                    }
                });

                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        ui.monospace(&self.midi_info_result);
                    });
            });
        });
    }
}

impl eframe::App for FretDanceApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 应用当前主题
        self.apply_theme(ctx);

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
                        self.apply_theme(ctx);
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.current_tab {
            Tab::ParameterSetting => self.show_parameter_setting(ui),
            Tab::MidiInfoScan => self.show_midi_info_scan(ui),
        });
    }
}
