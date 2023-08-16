use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Deserialize,
    serde::Serialize,
    sqlx::Type,
)]
pub struct Permissions(i64);

impl Permissions {
    pub const EMPTY: Self = Self(0b0);
    pub const ADMINISTRATOR: Self = Self(0b1 << 63);
    pub const VERIFY_RUNS: Self = Self(0b1 << 0);
    pub const BLOCK_USERS: Self = Self(0b1 << 1);
    pub const MANAGE_CATEGORIES: Self = Self(0b1 << 2);
    pub fn new(input: i64) -> Self {
        Self(input)
    }
    pub fn get(&self) -> i64 {
        self.0
    }
    pub fn is_empty(&self) -> bool {
        *self == Self::EMPTY
    }
    pub fn contains(&self, check: Self) -> bool {
        // administrators occupy the sign bit
        if self.0.is_negative() {
            return true;
        }
        (*self & check) == check
    }
}

impl BitOr for Permissions {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Permissions {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitAnd for Permissions {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Permissions {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}
