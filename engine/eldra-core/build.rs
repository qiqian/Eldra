use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn main() {
    let file_path = "../../submodule/wgpu-native/Cargo.toml"; // 目标文件

    // 检查文件是否存在
    if !Path::new(file_path).exists() {
        panic!("File not found: {}", file_path);
    }

    // 打开文件进行读取
    let file = File::open(file_path).expect("Unable to open file");
    let reader = BufReader::new(file);

    // 创建一个新的 String 来保存修改后的内容
    let mut new_content = String::new();

    // 逐行读取文件
    for line in reader.lines() {
        let line = line.expect("Unable to read line");
        if line.contains("crate-type = [\"cdylib\", \"staticlib\"]") {
            // 如果找到要替换的行，替换成新的内容
            new_content.push_str("crate-type = [\"cdylib\", \"staticlib\", \"lib\"]");
            new_content.push('\n');
        } else {
            // 否则保持原始行内容
            new_content.push_str(&line);
            new_content.push('\n');
        }
    }

    // 将新的内容写回文件
    fs::write(file_path, new_content).expect("Unable to write to file");
}