//! Pure math helpers from `xaero/common/misc/OptimizedMath` of the original
//! mod. Rotation helpers operate on Minecraft's `PoseStack`/JOML and remain
//! engine glue; `myFloor` is the data-side helper used by coordinate maths.

/// Java `OptimizedMath.myFloor(double)` — truncating cast with a correction
/// for negative non-integral values.
pub fn my_floor(value: f64) -> i32 {
    let mut truncated = value as i32;
    if truncated as f64 != value && value < 0.0 {
        truncated -= 1;
    }
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floors_positive_negative_and_integral_values() {
        assert_eq!(my_floor(3.9), 3);
        assert_eq!(my_floor(3.0), 3);
        assert_eq!(my_floor(-3.1), -4);
        assert_eq!(my_floor(-3.0), -3);
        assert_eq!(my_floor(-0.5), -1);
        assert_eq!(my_floor(0.5), 0);
    }
}
