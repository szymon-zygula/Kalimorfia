#[derive(Clone, Copy, Debug)]
pub struct Number {
    is_negative: bool,
    integral_part: u32,
    fractional_part: u32,
}

impl Number {
    pub fn from_f32(x: f32) -> Self {
        Self {
            is_negative: x < 0.0,
            integral_part: x.trunc() as u32,
            fractional_part: (x.fract() * 1000.0) as u32,
        }
    }

    pub fn to_f32(&self) -> f32 {
        (if self.is_negative { -1.0 } else { 1.0 }) * self.integral_part as f32
            + self.fractional_part as f32 * 0.001
    }

    pub fn from_str_prefix(string: &str) -> Option<(Self, &str)> {
        let (before, after) = string.split_once('.')?;
        let after_bytes = after.as_bytes();

        if before.is_empty()
            || after_bytes.len() < 3
            || !after_bytes[0].is_ascii_digit()
            || !after_bytes[1].is_ascii_digit()
            || !after_bytes[2].is_ascii_digit()
        {
            return None;
        }

        let is_negative = before.starts_with('-');
        let start = if is_negative { 1 } else { 0 };

        Some((
            Number {
                integral_part: before[start..].parse().ok()?,
                fractional_part: after[0..3].parse().ok()?,
                is_negative,
            },
            &after[3..],
        ))
    }
}

impl std::str::FromStr for Number {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((number, left)) = Number::from_str_prefix(s) else {
            return Err(());
        };

        if left.is_empty() {
            Ok(number)
        } else {
            Err(())
        }
    }
}
