use {
    nix::unistd::{self, Gid, Uid},
    std::io,
};

/// Set the current list of group IDs.
pub fn set_group_ids<I>(ids: I) -> io::Result<()>
where
    I: IntoIterator<Item = u32>,
{
    let ids = ids
        .into_iter()
        .map(|id| Gid::from_raw(id as _))
        .collect::<Vec<_>>();

    unistd::setgroups(&ids)?;

    Ok(())
}

/// Set the current group ID.
pub fn set_group_id(id: u32) -> io::Result<()> {
    unistd::setgid(Gid::from_raw(id as _))?;

    Ok(())
}

/// Set the current user ID.
pub fn set_user_id(id: u32) -> io::Result<()> {
    unistd::setuid(Uid::from_raw(id as _))?;

    Ok(())
}

/// Set current IDs, in a specific order that (hopefully) won't result in permission denied.
pub fn set_ids<I>(user_id: Option<u32>, group_id: Option<u32>, group_ids: I) -> io::Result<()>
where
    I: IntoIterator<Item = u32>,
{
    let group_ids = group_ids.into_iter().collect::<Vec<_>>();

    if !group_ids.is_empty() {
        set_group_ids(group_ids)?;
    }

    if let Some(group_id) = group_id {
        set_group_id(group_id)?;
    }

    if let Some(user_id) = user_id {
        set_user_id(user_id)?;
    }

    Ok(())
}
