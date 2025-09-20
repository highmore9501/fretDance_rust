pub struct RightFingers;

impl RightFingers {
    pub const P: usize = 0;
    pub const I: usize = 1;
    pub const M: usize = 2;
    pub const A: usize = 3;

    // 将函数整合为关联函数
    pub fn get_finger_index(finger: &str) -> usize {
        match finger {
            "p" => RightFingers::P,
            "i" => RightFingers::I,
            "m" => RightFingers::M,
            "a" => RightFingers::A,
            _ => panic!("Unknown finger: {}", finger),
        }
    }
}
