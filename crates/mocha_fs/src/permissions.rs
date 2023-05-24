use std::{fs::DirBuilder, os::unix::fs::DirBuilderExt};

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct Permission: u8 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXECUTE = 1 << 2;
        const STICKY = 1 << 3;
    }
}

const DEFAULT_PERMISSION: Permission = Permission::READ
    .union(Permission::WRITE)
    .union(Permission::EXECUTE);

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct Permissions {
    user: Permission,
    group: Permission,
    other: Permission,
}

impl Permissions {
    #[inline]
    pub const fn new() -> Self {
        Self {
            user: DEFAULT_PERMISSION,
            group: DEFAULT_PERMISSION,
            other: DEFAULT_PERMISSION,
        }
    }

    #[inline]
    pub const fn user(mut self, permission: Permission) -> Self {
        self.user = permission;
        self
    }

    #[inline]
    pub const fn group(mut self, permission: Permission) -> Self {
        self.group = permission;
        self
    }

    #[inline]
    pub const fn other(mut self, permission: Permission) -> Self {
        self.other = permission;
        self
    }

    #[inline]
    pub(crate) fn mode(self) -> u32 {
        let Self { user, group, other } = self;

        let user = (user.bits() as u32) << 8;
        let group = (group.bits() as u32) << 4;
        let other = other.bits() as u32;

        user | group | other
    }

    #[inline]
    pub(crate) fn dir_builder(self) -> DirBuilder {
        let mut builder = DirBuilder::new();

        builder.mode(self.mode());
        builder
    }
}
