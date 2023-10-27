use crate::cnc::number::Number;
use nalgebra::{vector, Vector3};

enum Coordinate {
    X,
    Y,
    Z,
}

struct CoordinateParseResult<'a> {
    coordinate: Coordinate,
    number: Number,
    left: &'a str,
}

#[derive(Default, Clone, Debug)]
pub struct Location {
    x: Option<Number>,
    y: Option<Number>,
    z: Option<Number>,
}

impl Location {
    pub fn to_f32(&self) -> Option<Vector3<f32>> {
        Some(Vector3::new(
            self.x.as_ref()?.to_f32(),
            self.y.as_ref()?.to_f32(),
            self.z.as_ref()?.to_f32(),
        ))
    }

    pub fn from_f32(location: &Vector3<f32>) -> Self {
        Self {
            x: Some(Number::from_f32(location.x)),
            y: Some(Number::from_f32(location.y)),
            z: Some(Number::from_f32(location.z)),
        }
    }

    pub fn relative_to(&self, other: &Vector3<f32>) -> Vector3<f32> {
        vector![
            self.x.map(|n| n.to_f32()).unwrap_or(other.x),
            self.y.map(|n| n.to_f32()).unwrap_or(other.y),
            self.z.map(|n| n.to_f32()).unwrap_or(other.z)
        ]
    }

    pub fn f32_dist(&self, other: &Vector3<f32>) -> f32 {
        let this = self.relative_to(other);

        Vector3::metric_distance(&this, other)
    }

    pub fn move_toward(&self, from: &Vector3<f32>, distance: f32) -> Vector3<f32> {
        let towards = self.relative_to(from);
        let Some(direction) = (towards - from).try_normalize(0.0) else {
            return *from;
        };
        from + distance * direction
    }

    fn parse_new_coordinate<'a>(&mut self, string: &'a str) -> Result<&'a str, ()> {
        let Some(CoordinateParseResult {
            coordinate,
            number,
            left,
        }) = Self::parse_coordinate(string)
        else {
            return Err(());
        };

        match coordinate {
            Coordinate::X => {
                if self.x.is_some() {
                    return Err(());
                }

                self.x = Some(number);
            }
            Coordinate::Y => {
                if self.y.is_some() {
                    return Err(());
                }

                self.y = Some(number);
            }
            Coordinate::Z => {
                if self.z.is_some() {
                    return Err(());
                }

                self.z = Some(number);
            }
        }

        Ok(left)
    }

    fn parse_coordinate(string: &str) -> Option<CoordinateParseResult> {
        let (coordinate, string) = Self::parse_coordinate_letter(string)?;
        let (number, left) = Number::from_str_prefix(string)?;
        Some(CoordinateParseResult {
            coordinate,
            number,
            left,
        })
    }

    fn parse_coordinate_letter(string: &str) -> Option<(Coordinate, &str)> {
        match string.as_bytes()[0] as char {
            'X' => Some((Coordinate::X, &string[1..])),
            'Y' => Some((Coordinate::Y, &string[1..])),
            'Z' => Some((Coordinate::Z, &string[1..])),
            _ => None,
        }
    }
}

impl std::str::FromStr for Location {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut location = Location::default();

        let left = location.parse_new_coordinate(s)?;
        if left.is_empty() {
            return Ok(location);
        }

        let left = location.parse_new_coordinate(left)?;
        if left.is_empty() {
            return Ok(location);
        }

        let left = location.parse_new_coordinate(left)?;
        if left.is_empty() {
            return Ok(location);
        }

        Err(())
    }
}
