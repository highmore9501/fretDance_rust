use crate::ui::app::{AvatarInfo, EditAvatarMode, FretDanceApp};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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

        // 根据乐器类型自动设置调弦预设
        if let Some(ref avatar_info) = self.current_avatar_info {
            let instrument_type = avatar_info.instrument.as_str();
            match instrument_type {
                "bass" => {
                    // 如果是bass，设置bass调弦
                    if let Some(bass_preset) =
                        self.tuning_presets.iter().find(|p| p.name.contains("Bass"))
                    {
                        self.guitar_string_notes = bass_preset.notes.clone();
                    }
                }
                "finger_style_guitar" | "electric_guitar" => {
                    // 如果是吉他，设置标准吉他调弦
                    if let Some(guitar_preset) = self
                        .tuning_presets
                        .iter()
                        .find(|p| p.name.contains("标准调弦"))
                    {
                        self.guitar_string_notes = guitar_preset.notes.clone();
                    }
                }
                _ => {
                    // 默认使用标准吉他调弦
                    if let Some(guitar_preset) = self
                        .tuning_presets
                        .iter()
                        .find(|p| p.name.contains("标准调弦"))
                    {
                        self.guitar_string_notes = guitar_preset.notes.clone();
                    }
                }
            }
        }
    }

    pub fn save_avatar(&mut self) -> Result<(), String> {
        // 检查名字是否为空
        if self.edit_avatar_name.is_empty() {
            return Err("Avatar名字不能为空".to_string());
        }

        // 检查是否重名
        let is_duplicate = match self.edit_avatar_mode {
            EditAvatarMode::New => {
                // 新增模式下检查是否与现有Avatar重名
                self.avatar_infos
                    .iter()
                    .any(|info| info.name == self.edit_avatar_name)
            }
            EditAvatarMode::Edit => {
                // 修改模式下检查是否与其他Avatar重名（排除自身）
                if let Some(ref current) = self.current_avatar_info {
                    self.avatar_infos
                        .iter()
                        .any(|info| info.name == self.edit_avatar_name && info.name != current.name)
                } else {
                    false
                }
            }
        };

        if is_duplicate {
            return Err("Avatar名字已存在".to_string());
        }

        // 处理图片文件路径
        let image_filename = if !self.edit_avatar_image.is_empty() {
            // 获取文件名
            let path = Path::new(&self.edit_avatar_image);
            let filename = path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| "default.png".to_string());

            // 如果图片不在asset/img目录中，需要复制过去
            let selected_image_path = if !self.edit_avatar_selected_image_path.is_empty() {
                Path::new(&self.edit_avatar_selected_image_path).to_path_buf()
            } else {
                Path::new(&self.edit_avatar_image).to_path_buf()
            };

            // 检查文件是否存在且不在目标目录中
            if selected_image_path.exists() && !selected_image_path.starts_with("asset/img/") {
                let target_path = Path::new("asset/img").join(&filename);
                // 将路径都转换为绝对路径后再比较
                if let (Ok(selected_abs_path), Ok(target_abs_path)) = (
                    std::fs::canonicalize(&selected_image_path),
                    std::fs::canonicalize(&target_path),
                ) {
                    if selected_abs_path != target_abs_path {
                        if let Err(e) = fs::copy(&selected_image_path, &target_path) {
                            eprintln!("复制图片文件失败: {}", e);
                        }
                    }
                } else {
                    // 如果无法获取绝对路径，回退到原始比较方法
                    if selected_image_path != target_path {
                        if let Err(e) = fs::copy(&selected_image_path, &target_path) {
                            eprintln!("复制图片文件失败: {}", e);
                        }
                    }
                }
            }

            filename
        } else {
            "default.png".to_string()
        };

        // 处理JSON文件路径
        let json_filename = if !self.edit_avatar_json.is_empty() {
            // 获取文件名
            let path = Path::new(&self.edit_avatar_json);
            let filename = path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("{}.json", self.edit_avatar_name));

            // 如果JSON文件不在asset/controller_infos目录中，需要复制过去
            let selected_json_path = if !self.edit_avatar_selected_json_path.is_empty() {
                Path::new(&self.edit_avatar_selected_json_path).to_path_buf()
            } else {
                Path::new(&self.edit_avatar_json).to_path_buf()
            };

            // 检查文件是否存在且不在目标目录中
            if selected_json_path.exists()
                && !selected_json_path.starts_with("asset/controller_infos/")
            {
                let target_path = Path::new("asset/controller_infos").join(&filename);
                // 将路径都转换为绝对路径后再比较
                if let (Ok(selected_abs_path), Ok(target_abs_path)) = (
                    std::fs::canonicalize(&selected_json_path),
                    std::fs::canonicalize(&target_path),
                ) {
                    if selected_abs_path != target_abs_path {
                        if let Err(e) = fs::copy(&selected_json_path, &target_path) {
                            eprintln!("复制JSON文件失败: {}", e);
                        }
                    }
                } else {
                    // 如果无法获取绝对路径，回退到原始比较方法
                    if selected_json_path != target_path {
                        if let Err(e) = fs::copy(&selected_json_path, &target_path) {
                            eprintln!("复制JSON文件失败: {}", e);
                        }
                    }
                }
            }

            filename
        } else {
            format!("{}.json", self.edit_avatar_name)
        };

        // 创建AvatarInfo对象
        let avatar_info = AvatarInfo {
            name: self.edit_avatar_name.clone(),
            file: json_filename,
            image: image_filename,
            instrument: self.edit_avatar_instrument.as_str().to_string(),
        };

        match self.edit_avatar_mode {
            EditAvatarMode::New => {
                // 新增Avatar
                self.avatar_infos.push(avatar_info);
            }
            EditAvatarMode::Edit => {
                // 修改Avatar
                if let Some(ref current) = self.current_avatar_info {
                    // 查找要修改的Avatar索引
                    if let Some(index) = self
                        .avatar_infos
                        .iter()
                        .position(|info| info.name == current.name)
                    {
                        // 更新Avatar信息
                        self.avatar_infos[index] = avatar_info;

                        // 如果修改的是当前选中的Avatar，更新当前选中项
                        if self.avatar == current.name {
                            self.avatar = self.edit_avatar_name.clone();
                        }
                    } else {
                        return Err("未找到要修改的Avatar".to_string());
                    }
                }
            }
        }

        // 更新avatar选项列表
        self.avatar_options = self
            .avatar_infos
            .iter()
            .map(|info| info.name.clone())
            .collect();

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
