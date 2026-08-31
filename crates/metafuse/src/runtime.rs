#![cfg(target_os = "linux")]

use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    fs::{self, OpenOptions},
    os::unix::{fs::FileExt, fs::MetadataExt, fs::OpenOptionsExt},
    path::{Component, Path, PathBuf},
    sync::RwLock,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use adad_core::Error;
use fuser::{
    FileAttr, FileHandle, FileType, Filesystem, FopenFlags, Generation, INodeNo, MountOption,
    OpenAccMode, OpenFlags, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, ReplyWrite,
    ReplyXattr, Request, WriteFlags,
};

use crate::{scrub_metadata, ScrubPolicy, VaultMetadata};

const ROOT_INO: u64 = 1;
const FIRST_CHILD_INO: u64 = 2;
const MAX_READ_BYTES: usize = 1024 * 1024;
const ATTRIBUTE_TTL: Duration = Duration::ZERO;

/// Mount a read-only FUSE view of a directory while preserving the existing
/// pure metadata policy as the single source of scrubbed attributes.
pub fn mount_read_only(
    source: impl AsRef<Path>,
    mountpoint: impl AsRef<Path>,
    policy: ScrubPolicy,
) -> Result<(), Error> {
    let source = validate_directory(source.as_ref())?;
    let mountpoint = validate_directory(mountpoint.as_ref())?;
    let mountpoint_canonical = fs::canonicalize(&mountpoint).map_err(|_| Error::Metafuse)?;
    if mountpoint_canonical == source || mountpoint_canonical.starts_with(&source) {
        return Err(Error::Metafuse);
    }

    let filesystem = MetadataFilesystem::new(source, policy);
    let mut config = fuser::Config::default();
    config.mount_options = vec![
        MountOption::RO,
        MountOption::DefaultPermissions,
        MountOption::NoDev,
        MountOption::NoSuid,
        MountOption::NoExec,
        MountOption::FSName("adad-metafuse".to_owned()),
    ];
    fuser::mount(filesystem, mountpoint, &config).map_err(|_| Error::Metafuse)
}

fn validate_directory(path: &Path) -> Result<PathBuf, Error> {
    let metadata = fs::symlink_metadata(path).map_err(|_| Error::Metafuse)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(Error::Metafuse);
    }
    fs::canonicalize(path).map_err(|_| Error::Metafuse)
}

struct MetadataFilesystem {
    root: PathBuf,
    policy: ScrubPolicy,
    nodes: RwLock<NodeTable>,
}

struct NodeTable {
    next_ino: u64,
    paths: HashMap<u64, PathBuf>,
    inodes: HashMap<PathBuf, u64>,
    parents: HashMap<u64, u64>,
}

impl MetadataFilesystem {
    fn new(root: PathBuf, policy: ScrubPolicy) -> Self {
        let mut paths = HashMap::new();
        let mut inodes = HashMap::new();
        let mut parents = HashMap::new();
        paths.insert(ROOT_INO, root.clone());
        inodes.insert(root.clone(), ROOT_INO);
        parents.insert(ROOT_INO, ROOT_INO);
        Self {
            root,
            policy,
            nodes: RwLock::new(NodeTable {
                next_ino: FIRST_CHILD_INO,
                paths,
                inodes,
                parents,
            }),
        }
    }

    fn path_for(&self, ino: INodeNo) -> Result<PathBuf, i32> {
        let table = self.nodes.read().map_err(|_| libc::EIO)?;
        let path = table.paths.get(&ino.0).cloned().ok_or(libc::ENOENT)?;
        self.validate_node_path(&path)
    }

    fn parent_for(&self, ino: u64) -> Result<u64, i32> {
        let table = self.nodes.read().map_err(|_| libc::EIO)?;
        table.parents.get(&ino).copied().ok_or(libc::ENOENT)
    }

    fn inode_for(&self, path: &Path, parent: u64) -> Result<u64, i32> {
        let mut table = self.nodes.write().map_err(|_| libc::EIO)?;
        if let Some(ino) = table.inodes.get(path) {
            return Ok(*ino);
        }
        let ino = table.next_ino;
        table.next_ino = table.next_ino.checked_add(1).ok_or(libc::EOVERFLOW)?;
        table.paths.insert(ino, path.to_owned());
        table.inodes.insert(path.to_owned(), ino);
        table.parents.insert(ino, parent);
        Ok(ino)
    }

    fn validate_node_path(&self, path: &Path) -> Result<PathBuf, i32> {
        let metadata = fs::symlink_metadata(path).map_err(|_| libc::ENOENT)?;
        if metadata.file_type().is_symlink() {
            return Err(libc::ELOOP);
        }
        let canonical = fs::canonicalize(path).map_err(|_| libc::ENOENT)?;
        if canonical != path || !canonical.starts_with(&self.root) {
            return Err(libc::EACCES);
        }
        if !metadata.is_dir() && !metadata.is_file() {
            return Err(libc::EOPNOTSUPP);
        }
        Ok(path.to_owned())
    }

    fn child_path(&self, parent: &Path, name: &OsStr) -> Result<PathBuf, i32> {
        let mut components = Path::new(name).components();
        if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
            return Err(libc::EINVAL);
        }
        let path = parent.join(name);
        self.validate_node_path(&path)
    }

    fn metadata(&self, ino: INodeNo) -> Result<(PathBuf, fs::Metadata), i32> {
        let path = self.path_for(ino)?;
        let metadata = fs::symlink_metadata(&path).map_err(|_| libc::ENOENT)?;
        Ok((path, metadata))
    }

    fn attr(&self, ino: INodeNo, path: &Path, metadata: &fs::Metadata) -> Result<FileAttr, i32> {
        let kind = if metadata.is_dir() {
            FileType::Directory
        } else if metadata.is_file() {
            FileType::RegularFile
        } else {
            return Err(libc::EOPNOTSUPP);
        };
        let created = unix_seconds(metadata.created());
        let modified = unix_seconds(metadata.modified());
        let accessed = unix_seconds(metadata.accessed());
        let scrubbed = scrub_metadata(
            &VaultMetadata::new(
                path.to_string_lossy(),
                metadata.uid(),
                metadata.gid(),
                created,
                modified,
                accessed,
                Vec::new(),
            ),
            &self.policy,
        )
        .map_err(|_| libc::EIO)?;

        Ok(FileAttr {
            ino,
            size: metadata.size(),
            blocks: metadata.blocks(),
            atime: scrubbed_time(scrubbed.timestamps.accessed_at),
            mtime: scrubbed_time(scrubbed.timestamps.modified_at),
            ctime: scrubbed_time(scrubbed.timestamps.modified_at),
            crtime: scrubbed_time(scrubbed.timestamps.created_at),
            kind,
            perm: (metadata.mode() & 0o7777) as u16,
            nlink: metadata.nlink().try_into().unwrap_or(u32::MAX),
            uid: scrubbed.uid,
            gid: scrubbed.gid,
            rdev: metadata.rdev().try_into().unwrap_or(u32::MAX),
            flags: 0,
            blksize: metadata.blksize().try_into().unwrap_or(u32::MAX),
        })
    }

    fn directory_entries(&self, ino: INodeNo, path: &Path) -> Result<Vec<DirectoryEntry>, i32> {
        let parent = self.parent_for(ino.0)?;
        let mut entries = vec![
            DirectoryEntry {
                ino: ino.0,
                kind: FileType::Directory,
                name: OsString::from("."),
            },
            DirectoryEntry {
                ino: parent,
                kind: FileType::Directory,
                name: OsString::from(".."),
            },
        ];
        let mut children = fs::read_dir(path)
            .map_err(|_| libc::EIO)?
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        children.sort_by_key(|entry| entry.file_name());
        for child in children {
            let name = child.file_name();
            let Ok(child_path) = self.child_path(path, &name) else {
                continue;
            };
            let Ok(metadata) = fs::symlink_metadata(&child_path) else {
                continue;
            };
            let kind = if metadata.is_dir() {
                FileType::Directory
            } else if metadata.is_file() {
                FileType::RegularFile
            } else {
                continue;
            };
            let child_ino = self.inode_for(&child_path, ino.0)?;
            entries.push(DirectoryEntry {
                ino: child_ino,
                kind,
                name,
            });
        }
        Ok(entries)
    }
}

struct DirectoryEntry {
    ino: u64,
    kind: FileType,
    name: OsString,
}

impl Filesystem for MetadataFilesystem {
    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {
        let result = (|| {
            let parent_path = self.path_for(parent)?;
            let parent_metadata = fs::symlink_metadata(&parent_path).map_err(|_| libc::ENOENT)?;
            if !parent_metadata.is_dir() {
                return Err(libc::ENOTDIR);
            }
            let path = self.child_path(&parent_path, name)?;
            let metadata = fs::symlink_metadata(&path).map_err(|_| libc::ENOENT)?;
            let ino = self.inode_for(&path, parent.0)?;
            let attr = self.attr(INodeNo(ino), &path, &metadata)?;
            Ok::<_, i32>(attr)
        })();
        match result {
            Ok(attr) => reply.entry(&ATTRIBUTE_TTL, &attr, Generation(0)),
            Err(errno) => reply.error(fuser::Errno::from_i32(errno)),
        }
    }

    fn getattr(&self, _req: &Request, ino: INodeNo, _fh: Option<FileHandle>, reply: ReplyAttr) {
        match self
            .metadata(ino)
            .and_then(|(path, metadata)| self.attr(ino, &path, &metadata))
        {
            Ok(attr) => reply.attr(&ATTRIBUTE_TTL, &attr),
            Err(errno) => reply.error(fuser::Errno::from_i32(errno)),
        }
    }

    fn readdir(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        mut reply: ReplyDirectory,
    ) {
        let result = (|| {
            let path = self.path_for(ino)?;
            let metadata = fs::symlink_metadata(&path).map_err(|_| libc::ENOENT)?;
            if !metadata.is_dir() {
                return Err(libc::ENOTDIR);
            }
            self.directory_entries(ino, &path)
        })();
        let entries = match result {
            Ok(entries) => entries,
            Err(errno) => {
                reply.error(fuser::Errno::from_i32(errno));
                return;
            }
        };
        for (index, entry) in entries.iter().enumerate().skip(offset as usize) {
            if reply.add(
                INodeNo(entry.ino),
                (index + 1) as u64,
                entry.kind,
                &entry.name,
            ) {
                break;
            }
        }
        reply.ok();
    }

    fn open(&self, _req: &Request, ino: INodeNo, flags: OpenFlags, reply: fuser::ReplyOpen) {
        let result = self.metadata(ino).and_then(|(_, metadata)| {
            if !metadata.is_file() {
                return Err(libc::EISDIR);
            }
            if flags.acc_mode() != OpenAccMode::O_RDONLY {
                return Err(libc::EROFS);
            }
            Ok(())
        });
        match result {
            Ok(()) => reply.opened(FileHandle(0), FopenFlags::empty()),
            Err(errno) => reply.error(fuser::Errno::from_i32(errno)),
        }
    }

    fn opendir(&self, _req: &Request, ino: INodeNo, _flags: OpenFlags, reply: fuser::ReplyOpen) {
        match self.metadata(ino) {
            Ok((_, metadata)) if metadata.is_dir() => {
                reply.opened(FileHandle(0), FopenFlags::empty())
            }
            Ok(_) => reply.error(fuser::Errno::ENOTDIR),
            Err(errno) => reply.error(fuser::Errno::from_i32(errno)),
        }
    }

    fn read(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        size: u32,
        _flags: OpenFlags,
        _lock_owner: Option<fuser::LockOwner>,
        reply: ReplyData,
    ) {
        let result = (|| {
            let path = self.path_for(ino)?;
            let metadata = fs::symlink_metadata(&path).map_err(|_| libc::ENOENT)?;
            if !metadata.is_file() {
                return Err(libc::EISDIR);
            }
            let size = usize::try_from(size).map_err(|_| libc::EOVERFLOW)?;
            if size > MAX_READ_BYTES {
                return Err(libc::EOVERFLOW);
            }
            let mut options = OpenOptions::new();
            options
                .read(true)
                .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
            let file = options.open(&path).map_err(|_| libc::EIO)?;
            let mut buffer = vec![0_u8; size];
            let read = file.read_at(&mut buffer, offset).map_err(|_| libc::EIO)?;
            Ok(buffer[..read].to_vec())
        })();
        match result {
            Ok(data) => reply.data(&data),
            Err(errno) => reply.error(fuser::Errno::from_i32(errno)),
        }
    }

    fn getxattr(
        &self,
        _req: &Request,
        _ino: INodeNo,
        _name: &OsStr,
        _size: u32,
        reply: ReplyXattr,
    ) {
        reply.error(fuser::Errno::ENODATA);
    }

    fn listxattr(&self, _req: &Request, _ino: INodeNo, size: u32, reply: ReplyXattr) {
        if size == 0 {
            reply.size(0);
        } else {
            reply.data(&[]);
        }
    }

    fn write(
        &self,
        _req: &Request,
        _ino: INodeNo,
        _fh: FileHandle,
        _offset: u64,
        _data: &[u8],
        _write_flags: WriteFlags,
        _flags: OpenFlags,
        _lock_owner: Option<fuser::LockOwner>,
        reply: ReplyWrite,
    ) {
        reply.error(fuser::Errno::EROFS);
    }
}

fn unix_seconds(time: std::io::Result<SystemTime>) -> i64 {
    time.ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .and_then(|value| i64::try_from(value.as_secs()).ok())
        .unwrap_or(0)
}

fn scrubbed_time(seconds: i64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(u64::try_from(seconds).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::{MetadataFilesystem, ROOT_INO};
    use crate::ScrubPolicy;
    use std::path::PathBuf;

    #[test]
    fn metadata_filesystem_presents_fake_owner_and_scrubbed_times() {
        let root = std::env::current_dir().expect("repository root");
        let filesystem = MetadataFilesystem::new(
            root.clone(),
            ScrubPolicy::new(65_534, 65_533, [7; 32]).expect("policy"),
        );
        let path = root.join("Cargo.toml");
        let metadata = std::fs::symlink_metadata(&path).expect("manifest");
        let attr = filesystem
            .attr(super::INodeNo(ROOT_INO), &path, &metadata)
            .expect("attributes");

        assert_eq!(attr.uid, 65_534);
        assert_eq!(attr.gid, 65_533);
        assert_ne!(attr.mtime, metadata.modified().expect("mtime"));
    }

    #[test]
    fn child_path_rejects_components_that_could_escape_the_source() {
        let root = std::env::current_dir().expect("repository root");
        let filesystem = MetadataFilesystem::new(
            root.clone(),
            ScrubPolicy::new(65_534, 65_533, [7; 32]).expect("policy"),
        );

        assert!(filesystem
            .child_path(&root, PathBuf::from("../outside").as_os_str())
            .is_err());
        assert!(filesystem
            .child_path(&root, PathBuf::from(".").as_os_str())
            .is_err());
    }
}
