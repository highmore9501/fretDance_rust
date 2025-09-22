use std::error::Error;
use std::f64;

#[derive(Debug, Clone, Copy)]
pub struct Quaternion {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Quaternion {
    pub fn new(w: f64, x: f64, y: f64, z: f64) -> Self {
        Quaternion { w, x, y, z }
    }

    pub fn norm(&self) -> f64 {
        (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Quaternion {
        let norm = self.norm();
        Quaternion {
            w: self.w / norm,
            x: self.x / norm,
            y: self.y / norm,
            z: self.z / norm,
        }
    }

    pub fn dot(&self, other: &Quaternion) -> f64 {
        self.w * other.w + self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn scale(&self, scalar: f64) -> Quaternion {
        Quaternion {
            w: self.w * scalar,
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }

    pub fn add(&self, other: &Quaternion) -> Quaternion {
        Quaternion {
            w: self.w + other.w,
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn subtract(&self, other: &Quaternion) -> Quaternion {
        Quaternion {
            w: self.w - other.w,
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn negate(&self) -> Quaternion {
        Quaternion {
            w: -self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    pub fn to_vector64(&self) -> Vec<f64> {
        return vec![self.w, self.x, self.y, self.z];
    }

    pub fn from_vector64(vector: Vec<f64>) -> Quaternion {
        Quaternion {
            w: vector[0],
            x: vector[1],
            y: vector[2],
            z: vector[3],
        }
    }

    /// 四元数与3D向量相乘 (旋转向量)
    /// 实现 v' = q * v * conjugate(q)
    pub fn multiply_vector(&self, vector: &Vec<f64>) -> Vec<f64> {
        // 将向量转换为纯四元数 (0, vx, vy, vz)
        let v_quat = Quaternion::new(0.0, vector[0], vector[1], vector[2]);

        // 计算 q * v
        let qv = self.multiply(&v_quat);

        // 计算 conjugate(q)
        let q_conjugate = Quaternion::new(self.w, -self.x, -self.y, -self.z);

        // 计算 (q * v) * conjugate(q)
        let result_quat = qv.multiply(&q_conjugate);

        // 返回结果向量部分
        vec![result_quat.x, result_quat.y, result_quat.z]
    }

    /// 四元数乘法
    pub fn multiply(&self, other: &Quaternion) -> Quaternion {
        Quaternion {
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        }
    }
}

pub fn slerp(q1: &Quaternion, q2: &Quaternion, t_tan: f64) -> Quaternion {
    // 标准化四元数
    let q1_normalized = q1.normalize();
    let q2_normalized = q2.normalize();

    // 如果两个四元数相同，直接返回
    if all_close(q1_normalized, q2_normalized) {
        return q1_normalized;
    }

    // 计算两个四元数旋转值的夹角
    let dot = q1_normalized.dot(&q2_normalized);
    let angel_max = dot.acos() / q1_normalized.norm();
    let tan_angel_max = angel_max.tan();
    let current_angel = tan_angel_max * t_tan;
    let t = current_angel / angel_max;

    // 如果点积为负，取反一个四元数以选择较短的路径
    let (q2_used, dot_used) = if dot < 0.0 {
        (q2_normalized.negate(), -dot)
    } else {
        (q2_normalized, dot)
    };

    // 如果四元数非常接近，使用线性插值避免数值不稳定
    if dot_used > 0.9995 {
        let diff = q2_used.subtract(&q1_normalized);
        let result = q1_normalized.add(&diff.scale(t));
        return result.normalize();
    }

    // 计算角度和插值
    let theta_0 = dot_used.acos();
    let theta = theta_0 * t;
    let sin_theta = theta.sin();
    let sin_theta_0 = theta_0.sin();

    let s1 = theta.cos() - dot_used * sin_theta / sin_theta_0;
    let s2 = sin_theta / sin_theta_0;

    let part1 = q1_normalized.scale(s1);
    let part2 = q2_used.scale(s2);
    part1.add(&part2)
}

fn all_close(q1: Quaternion, q2: Quaternion) -> bool {
    const EPSILON: f64 = 1e-9;
    (q1.w - q2.w).abs() < EPSILON
        && (q1.x - q2.x).abs() < EPSILON
        && (q1.y - q2.y).abs() < EPSILON
        && (q1.z - q2.z).abs() < EPSILON
}

#[derive(Debug, Clone, Copy)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vector3 { x, y, z }
    }

    pub fn from_vector64(vector: Vec<f64>) -> Vector3 {
        Vector3 {
            x: vector[0],
            y: vector[1],
            z: vector[2],
        }
    }

    pub fn to_vector64(&self) -> Vec<f64> {
        vec![self.x, self.y, self.z]
    }

    pub fn subtract(&self, other: &Vector3) -> Vector3 {
        Vector3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn add(&self, other: &Vector3) -> Vector3 {
        Vector3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn scale(&self, scalar: f64) -> Vector3 {
        Vector3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

// 为 f64 标量实现插值
pub fn lerp_by_fret_scalar(fret: f64, value_1: f64, value_12: f64) -> f64 {
    // 处理边界情况
    if fret == 1.0 {
        return value_1;
    } else if fret == 12.0 {
        return value_12;
    }

    // 计算各种比率
    let ratio_fret = 2.0_f64.powf(-fret / 12.0);
    let ratio_1 = 2.0_f64.powf(-1.0 / 12.0);
    let ratio_12 = 2.0_f64.powf(-12.0 / 12.0); // 即 0.5

    // 计算插值参数
    let t = (ratio_fret - ratio_1) / (ratio_12 - ratio_1);

    // 标量情况：使用线性插值
    value_1 + (value_12 - value_1) * t
}

// 为 Vector3 实现插值
pub fn lerp_by_fret_vector3(fret: f64, value_1: &Vector3, value_12: &Vector3) -> Vector3 {
    // 处理边界情况
    if fret == 1.0 {
        return *value_1;
    } else if fret == 12.0 {
        return *value_12;
    }

    // 计算各种比率
    let ratio_fret = 2.0_f64.powf(-fret / 12.0);
    let ratio_1 = 2.0_f64.powf(-1.0 / 12.0);
    let ratio_12 = 2.0_f64.powf(-12.0 / 12.0); // 即 0.5

    // 计算插值参数
    let t = (ratio_fret - ratio_1) / (ratio_12 - ratio_1);

    // 向量情况：使用线性插值
    let diff = value_12.subtract(value_1);
    value_1.add(&diff.scale(t))
}

// 为 Quaternion 实现插值（使用你已有的 slerp 函数）
pub fn lerp_by_fret_quaternion(
    fret: f64,
    value_1: &Quaternion,
    value_12: &Quaternion,
) -> Quaternion {
    // 处理边界情况
    if fret == 1.0 {
        return *value_1;
    } else if fret == 12.0 {
        return *value_12;
    }

    // 计算各种比率
    let ratio_fret = 2.0_f64.powf(-fret / 12.0);
    let ratio_1 = 2.0_f64.powf(-1.0 / 12.0);
    let ratio_12 = 2.0_f64.powf(-12.0 / 12.0); // 即 0.5

    // 计算插值参数
    let t_tan = (ratio_fret - ratio_1) / (ratio_12 - ratio_1);

    // 四元数情况：使用球面线性插值
    slerp(value_1, value_12, t_tan)
}

/// 计算向量的模长
pub fn vector_norm(vector: &Vec<f64>) -> f64 {
    vector.iter().map(|x| x * x).sum::<f64>().sqrt()
}

/// 向量归一化
pub fn normalize_vector(vector: Vec<f64>) -> Vec<f64> {
    let norm = vector_norm(&vector);
    vector.iter().map(|x| x / norm).collect()
}

/// 向量相加
pub fn add_vectors(a: &Vec<f64>, b: &Vec<f64>) -> Vec<f64> {
    a.iter().zip(b.iter()).map(|(x, y)| x + y).collect()
}

/// 向量相减
pub fn subtract_vectors(a: &Vec<f64>, b: &Vec<f64>) -> Vec<f64> {
    a.iter().zip(b.iter()).map(|(x, y)| x - y).collect()
}

/// 向量数乘
pub fn scale_vector(vector: &Vec<f64>, scalar: f64) -> Vec<f64> {
    vector.iter().map(|x| x * scalar).collect()
}

/// 计算向量的叉积
pub fn cross_product(a: &Vec<f64>, b: &Vec<f64>) -> Vec<f64> {
    vec![
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// 计算向量的点积
pub fn dot_product(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// 计算吉他弦与手指运动平面的交点
pub fn get_string_touch_position(
    h: &Vec<f64>,              // 手掌位置
    f: &Vec<f64>,              // 手指位置
    n_quat: &Vec<f64>,         // 手背法线方向的旋转四元数或欧拉角
    p0: &Vec<f64>,             // 最低音弦上的一个端点位置
    p1: &Vec<f64>,             // 最低音弦上的另一个端点位置
    p2: &Vec<f64>,             // 最高音弦上的一个端点位置
    p3: &Vec<f64>,             // 最高音弦上的另一个端点位置
    current_string_index: i32, // 当前弦的索引
    max_string_index: f64,     // 最大弦索引
) -> Result<Vec<f64>, Box<dyn Error>> {
    // 基准向量为(0,0,1),也就是blender里的z轴方向
    let base_vector = vec![0.0, 0.0, 1.0];

    // 计算插值权重
    let weight = if max_string_index == 0.0 {
        0.0
    } else {
        current_string_index as f64 / max_string_index
    };

    // 通过插值计算当前弦的两个端点
    let p1_p0_diff = subtract_vectors(p1, p0);
    let scaled_diff = scale_vector(&p1_p0_diff, weight);
    let s = add_vectors(p0, &scaled_diff); // 当前弦的一个端点

    let p3_p2_diff = subtract_vectors(p3, p2);
    let scaled_diff = scale_vector(&p3_p2_diff, weight);
    let e = add_vectors(p2, &scaled_diff); // 当前弦的另一个端点

    // 计算弦的方向向量
    let d_dir = subtract_vectors(&e, &s);
    let d_dir_norm = vector_norm(&d_dir);
    let d_dir = scale_vector(&d_dir, 1.0 / d_dir_norm);

    let n_dir = if n_quat.len() == 4 {
        // 四元数情况，使用四元数乘法旋转基准向量
        let quat = Quaternion::from_vector64(n_quat.clone());
        let rotated_vector = quat.multiply_vector(&base_vector);
        let n_dir_norm = vector_norm(&rotated_vector);
        scale_vector(&rotated_vector, 1.0 / n_dir_norm)
    } else if n_quat.len() == 3 {
        n_quat.clone()
    } else {
        return Err("N_quat参数长度错误".into());
    };

    // 计算平面内的向量HF = F - H
    let hf = subtract_vectors(f, h);

    // 计算平面法向量 M = HF × N_dir
    let m = cross_product(&hf, &n_dir);

    // 检查平面法向量是否有效
    if vector_norm(&m) < 1e-5 {
        return Err("向量HF与N_dir平行，无法定义平面".into());
    }

    // 计算弦起点到手指的向量 FS = S - F
    let fs = subtract_vectors(&s, f);

    // 计算分母：M·D_dir
    let denominator = dot_product(&m, &d_dir);

    // 检查弦是否平行于平面
    if denominator.abs() < 1e-5 {
        return Err("弦方向与平面平行，无交点".into());
    }

    // 计算参数 t = - (M·HS) / (M·D_dir)
    let numerator = dot_product(&m, &fs);
    let t = -numerator / denominator;

    // 计算交点 P = S + t * D_dir
    let scaled_d_dir = scale_vector(&d_dir, t);
    Ok(add_vectors(&s, &scaled_d_dir))
}
