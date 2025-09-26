use crate::ui::app::{AvatarInfo, FretDanceApp};
use std::fs;

impl FretDanceApp {
    pub fn load_avatar_options(&mut self) {
        // 从avatar_infos.json加载avatar选项
        let avatar_infos_path = "asset/controller_infos/avatar_infos.json";
        if let Ok(content) = fs::read_to_string(avatar_infos_path) {
            if let Ok(avatar_infos) = serde_json::from_str::<Vec<AvatarInfo>>(&content) {
                self.avatar_infos = avatar_infos.clone();
                self.avatar_options = avatar_infos.iter().map(|info| info.name.clone()).collect();

                // 设置当前avatar信息
                self.update_current_avatar_info();

                if !self.avatar_options.is_empty() && self.avatar.is_empty() {
                    self.avatar = self.avatar_options[0].clone();
                }
            }
        }
    }

    pub fn load_avatar_infos(&mut self) {
        // 这里可以加载额外的avatar信息处理
    }

    pub fn update_current_avatar_info(&mut self) {
        // 根据当前选择的avatar更新avatar信息
        self.current_avatar_info = self
            .avatar_infos
            .iter()
            .find(|info| info.name == self.avatar)
            .cloned();
    }
}
