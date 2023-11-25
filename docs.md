### Filesystem structure

- `/dev/` - Device files.
- `/init` - Init.
- `/proc/` - Process and kernel information.
- `/sys/` - Information about devices, drivers, and kernel features.
- `/system/{package_name}/` - Mounted package images.
- `/userdata/packages/{package_name}.mocha` - Package images.
- `/userdata/{user_name}/cache/{package_name}/` - Per-package cache.
- `/userdata/{user_name}/data/{package_name}/` - Per-package data.
- `/userdata/{user_name}/settings.toml` - User configuration.

#### Additional information

- RootFS (`/`) is immutable.
- `{package_name}.mocha` is in the mocha package format, it contains metadata about the package, then EROFS data.
- `/userdata/images` is a volume (BTRFS subvolume?).
- Each `{user_name}` in `/userdata/` is a (BTRFS subvolume?).
