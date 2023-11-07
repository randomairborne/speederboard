use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, sqlx::Type)]
pub struct Permissions(i64);

impl Permissions {
    // TODO: Stupid rustfmt should stop reordering my constants
    pub const ADMINISTRATOR: Self = Self(0b1 << 63);
    pub const BLOCK_USERS: Self = Self(0b1 << 1);
    pub const EMPTY: Self = Self(0b0);
    pub const MANAGE_CATEGORIES: Self = Self(0b1 << 2);
    pub const VERIFY_RUNS: Self = Self(0b1 << 0);

    pub fn new(input: i64) -> Self {
        Self(input)
    }

    pub fn new_opt(input: Option<i64>) -> Self {
        if let Some(input) = input {
            Self::new(input)
        } else {
            Self::EMPTY
        }
    }

    #[inline]
    pub fn get(self) -> i64 {
        self.0
    }

    #[inline]
    #[allow(dead_code)]
    pub fn is_empty(self) -> bool {
        self == Self::EMPTY
    }

    #[inline]
    pub fn contains(self, check: Self) -> bool {
        // administrators occupy the sign bit
        if (self & Self::ADMINISTRATOR) == Self::ADMINISTRATOR {
            return true;
        }
        (self & check) == check
    }

    #[inline]
    pub fn check(self, check: Self) -> Result<(), crate::Error> {
        if self.contains(check) {
            Ok(())
        } else {
            Err(crate::Error::InsufficientPermissions)
        }
    }

    #[inline]
    fn expand(self) -> PermissionsSerde {
        PermissionsSerde {
            administrator: self.contains(Self::ADMINISTRATOR),
            verify_runs: self.contains(Self::VERIFY_RUNS),
            block_users: self.contains(Self::BLOCK_USERS),
            manage_categories: self.contains(Self::MANAGE_CATEGORIES),
        }
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

impl From<i64> for Permissions {
    fn from(value: i64) -> Self {
        Self::new(value)
    }
}

impl From<Option<i64>> for Permissions {
    fn from(value: Option<i64>) -> Self {
        Self::new_opt(value)
    }
}

impl From<PermissionsSerde> for Permissions {
    fn from(value: PermissionsSerde) -> Self {
        value.compress()
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, Debug, PartialEq, Eq)]
struct PermissionsSerde {
    administrator: bool,
    block_users: bool,
    manage_categories: bool,
    verify_runs: bool,
}

impl PermissionsSerde {
    fn compress(self) -> Permissions {
        let mut smol = Permissions::EMPTY;
        if self.administrator {
            smol |= Permissions::ADMINISTRATOR;
        }
        if self.block_users {
            smol |= Permissions::BLOCK_USERS;
        }
        if self.manage_categories {
            smol |= Permissions::MANAGE_CATEGORIES;
        }
        if self.verify_runs {
            smol |= Permissions::VERIFY_RUNS;
        }
        smol
    }
}

impl From<Permissions> for PermissionsSerde {
    fn from(value: Permissions) -> Self {
        value.expand()
    }
}

impl serde::Serialize for Permissions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.expand().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Permissions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(PermissionsSerde::deserialize(deserializer)?.into())
    }
}
