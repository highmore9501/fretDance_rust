@echo off
echo 构建发布版本...
cargo build --release

echo 创建分发目录...
mkdir fret_dance_dist 2>nul
mkdir fret_dance_dist\asset 2>nul
mkdir fret_dance_dist\output 2>nul

echo 复制可执行文件...
copy target\release\fret_dance_rust.exe fret_dance_dist\

echo 创建输出目录...
mkdir fret_dance_dist\output\midi_info 2>nul
mkdir fret_dance_dist\output\hand_recorder 2>nul
mkdir fret_dance_dist\output\hand_animation 2>nul
mkdir fret_dance_dist\output\string_recorder 2>nul
mkdir fret_dance_dist\output\final_result 2>nul

echo 打包完成!