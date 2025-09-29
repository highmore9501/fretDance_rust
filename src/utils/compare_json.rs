use serde_json::Value;
use std::fs::File;
use std::io::BufReader;

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
