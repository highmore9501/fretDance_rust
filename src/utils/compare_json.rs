use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// 比较两个JSON文件的结构
///
/// # Arguments
/// * `file1_path` - 第一个JSON文件的路径
/// * `file2_path` - 第二个JSON文件的路径
///
/// # Returns
/// * `Result<bool, Box<dyn std::error::Error>>` - 如果结构相同返回true，否则返回false
pub fn compare_json_structure(
    file1_path: &str,
    file2_path: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    // 读取第一个文件
    let file1 = File::open(file1_path)?;
    let reader1 = BufReader::new(file1);
    let json1: Value = serde_json::from_reader(reader1)?;

    // 读取第二个文件
    let file2 = File::open(file2_path)?;
    let reader2 = BufReader::new(file2);
    let json2: Value = serde_json::from_reader(reader2)?;

    // 比较结构
    let same_structure = compare_values_structure(&json1, &json2);

    // 如果结构不同，打印两个文件的结构
    // if !same_structure {
    //     println!("JSON结构不匹配!");
    //     println!("\n第一个文件 {} 的结构:", file1_path);
    //     print_json_structure(&json1);

    //     println!("\n第二个文件 {} 的结构:", file2_path);
    //     print_json_structure(&json2);
    // }

    Ok(same_structure)
}

/// 递归比较两个JSON值的结构
fn compare_values_structure(val1: &Value, val2: &Value) -> bool {
    match (val1, val2) {
        // 两个都是对象
        (Value::Object(obj1), Value::Object(obj2)) => {
            // 检查键的数量是否相同
            if obj1.len() != obj2.len() {
                return false;
            }

            // 检查所有键是否相同
            for key in obj1.keys() {
                if !obj2.contains_key(key) {
                    return false;
                }

                // 递归比较子值的结构
                if !compare_values_structure(&obj1[key], &obj2[key]) {
                    return false;
                }
            }
            true
        }

        // 两个都是数组
        (Value::Array(arr1), Value::Array(arr2)) => {
            // 对于数组，我们只检查元素类型是否一致
            // 如果两个数组都为空，则结构相同
            if arr1.is_empty() && arr2.is_empty() {
                return true;
            }

            // 如果其中一个为空而另一个不为空，则结构不同
            if arr1.is_empty() || arr2.is_empty() {
                return false;
            }

            // 检查第一个元素的结构是否相同（简化处理）
            compare_values_structure(&arr1[0], &arr2[0])
        }

        // 两个都是null
        (Value::Null, Value::Null) => true,

        // 两个都是布尔值
        (Value::Bool(_), Value::Bool(_)) => true,

        // 两个都是数字
        (Value::Number(_), Value::Number(_)) => true,

        // 两个都是字符串
        (Value::String(_), Value::String(_)) => true,

        // 其他情况结构不同
        _ => {
            // print_value_structure(val1, 2);
            // print_value_structure(val2, 2);
            false
        }
    }
}

/// 打印JSON结构的摘要信息
pub fn print_json_structure(json: &Value) {
    println!("Structure :");
    print_value_structure(&json, 2);
}

/// 递归打印值的结构
fn print_value_structure(value: &Value, indent: usize) {
    let indent_str = "  ".repeat(indent);

    match value {
        Value::Object(obj) => {
            println!("{}Object ({} keys):", indent_str, obj.len());
            for (key, val) in obj {
                println!("{}  \"{}\":", indent_str, key);
                print_value_structure(val, indent + 2);
            }
        }

        Value::Array(arr) => {
            if arr.is_empty() {
                println!("{}Array (0 elements)", indent_str);
            } else {
                println!("{}Array ({} elements)", indent_str, arr.len());
                print_value_structure(&arr[0], indent + 1);
                if arr.len() > 1 {
                    println!(
                        "{}  ... ({} more elements with same structure)",
                        indent_str,
                        arr.len() - 1
                    );
                }
            }
        }

        Value::String(_) => {
            println!("{}String", indent_str);
        }

        Value::Number(_) => {
            println!("{}Number", indent_str);
        }

        Value::Bool(_) => {
            println!("{}Boolean", indent_str);
        }

        Value::Null => {
            println!("{}Null", indent_str);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_json_structure() {
        // 创建测试用的JSON数据
        let json1_str = r#"{"name": "test", "age": 30, "tags": ["rust", "json"]}"#;
        let json2_str = r#"{"name": "example", "age": 25, "tags": ["python", "yaml"]}"#;
        let json3_str = r#"{"name": "test", "age": 30}"#;

        let json1: Value = serde_json::from_str(json1_str).unwrap();
        let json2: Value = serde_json::from_str(json2_str).unwrap();
        let json3: Value = serde_json::from_str(json3_str).unwrap();

        // 测试相同结构
        let same_structure1 = compare_values_structure(&json1, &json2);
        assert!(same_structure1);

        // 测试不同结构并打印结构
        let same_structure2 = compare_values_structure(&json1, &json3);
        assert!(!same_structure2);

        // 手动调用打印函数查看结构
        println!("JSON1结构:");
        print_value_structure(&json1, 2);

        println!("JSON3结构:");
        print_value_structure(&json3, 2);
    }

    #[test]
    fn test_controller_info_structures() {
        let controller_info_path = "asset/controller_infos";
        let reference_files = ["神里绫华-花时来信.json", "Mavuika_E.json"];

        // 检查目录是否存在
        if !Path::new(controller_info_path).exists() {
            println!("目录 {} 不存在", controller_info_path);
            return;
        }

        // 遍历目录中的所有JSON文件
        if let Ok(entries) = fs::read_dir(controller_info_path) {
            let json_files: Vec<_> = entries
                .filter_map(|entry| {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("json") {
                            if let Some(file_name) = path.file_name() {
                                return Some(file_name.to_string_lossy().to_string());
                            }
                        }
                    }
                    None
                })
                .collect();

            println!("找到 {} 个JSON文件进行结构对比测试", json_files.len());

            // 对每个参考文件进行比较
            for reference_file in &reference_files {
                let reference_path = format!("{}/{}", controller_info_path, reference_file);

                if !Path::new(&reference_path).exists() {
                    println!("参考文件 {} 不存在，跳过", reference_path);
                    continue;
                }

                println!("\n=== 与参考文件 {} 进行结构对比 ===", reference_file);

                for json_file in &json_files {
                    let file_path = format!("{}/{}", controller_info_path, json_file);

                    match compare_json_structure(&reference_path, &file_path) {
                        Ok(same_structure) => {
                            if same_structure {
                                println!("✓ {} 与 {} 结构相同", json_file, reference_file);
                            } else {
                                println!("✗ {} 与 {} 结构不同", json_file, reference_file);
                            }
                        }
                        Err(e) => {
                            println!("✗ 比较 {} 与 {} 时出错: {}", json_file, reference_file, e);
                        }
                    }
                }
            }
        } else {
            println!("无法读取目录 {}", controller_info_path);
        }
    }
}
