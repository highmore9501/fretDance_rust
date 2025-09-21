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
