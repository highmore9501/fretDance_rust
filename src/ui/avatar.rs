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

    pub fn delete_avatar(&mut self, avatar_name: &str) -> Result<(), String> {
        // 防止删除最后一个avatar
        if self.avatar_infos.len() <= 1 {
            return Err("不能删除最后一个Avatar".to_string());
        }

        // 查找要删除的avatar
        let index = self
            .avatar_infos
            .iter()
            .position(|info| info.name == avatar_name);
        if index.is_none() {
            return Err("未找到指定的Avatar".to_string());
        }
        let index = index.unwrap();

        // 从内存中删除avatar
        self.avatar_infos.remove(index);

        // 更新avatar选项列表
        self.avatar_options = self
            .avatar_infos
            .iter()
            .map(|info| info.name.clone())
            .collect();

        // 如果删除的是当前选中的avatar，则选择下一个（如果存在）或上一个
        if self.avatar == avatar_name {
            if index < self.avatar_infos.len() {
                self.avatar = self.avatar_infos[index].name.clone();
            } else {
                self.avatar = self.avatar_infos[index - 1].name.clone();
            }
        }

        // 更新当前avatar信息
        self.update_current_avatar_info();

        // 保存到文件
        let avatar_infos_path = "asset/controller_infos/avatar_infos.json";
        let json_content = serde_json::to_string_pretty(&self.avatar_infos)
            .map_err(|e| format!("序列化Avatar信息失败: {}", e))?;

        std::fs::write(avatar_infos_path, json_content)
            .map_err(|e| format!("写入文件失败: {}", e))?;

        Ok(())
    }
}
