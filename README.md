# Kconfigurator

Kconfigurator is a tool for parsing and analyzing Kconfig options. It provides a simple and efficient way to extract Kconfig information from the Linux kernel source code. All `Kconfig` and `Kconfig.*` files are matches and processed.

## Example Output

The following is an example output of the Kconfigurator tool:

```plaintext
~/repos/kconfigurator$ cargo run --release -- /path/to/linux/source
KOption {
  name:         CEPH_FS
  option_type:  tristate
  description:   "Ceph distributed file system"
  depends:      ["INET"]
  selects:      ["CEPH_LIB", "LIBCRC32C", "CRYPTO_AES", "CRYPTO", "NETFS_SUPPORT"]
  help:                   Choose Y or M here to include support for mounting the
          experimental Ceph distributed file system.  Ceph is an extremely
          scalable file system designed to provide high performance,
          reliable access to petabytes of storage.

          More information at https://ceph.io/.

          If unsure, say N.

  defaults:     ["n"]
}
KOption {
  name:         CEPH_FS_POSIX_ACL
  option_type:  bool
  description:   "Ceph POSIX Access Control Lists"
  depends:      ["CEPH_FS"]
  selects:      ["FS_POSIX_ACL"]
  help:                   POSIX Access Control Lists (ACLs) support permissions for users and
          groups beyond the owner/group/world scheme.

          If you don't know what Access Control Lists are, say N

}
KOption {
  name:         CEPH_FS_SECURITY_LABEL
  option_type:  bool
  description:   "CephFS Security Labels"
  depends:      ["CEPH_FS && SECURITY"]
  help:                   Security labels support alternative access control models
          implemented by security modules like SELinux. This option
          enables an extended attribute handler for file security
          labels in the Ceph filesystem.

          If you are not using a security module that requires using
          extended attributes for file security labels, say N.
}
KOption {
  name:         EXT3_FS
  option_type:  tristate
  description:   "The Extended 3 (ext3) filesystem"
  selects:      ["EXT4_FS"]
  help:                   This config option is here only for backward compatibility. ext3
          filesystem is now handled by the ext4 driver.

}
KOption {
  name:         EXT3_FS_POSIX_ACL
  option_type:  bool
  description:   "Ext3 POSIX Access Control Lists"
  depends:      ["EXT3_FS"]
  selects:      ["EXT4_FS_POSIX_ACL", "FS_POSIX_ACL"]
  help:                   This config option is here only for backward compatibility. ext3
          filesystem is now handled by the ext4 driver.

}
KOption {
  name:         EXT3_FS_SECURITY
  option_type:  bool
  description:   "Ext3 Security Labels"
  depends:      ["EXT3_FS"]
  selects:      ["EXT4_FS_SECURITY"]
  help:                   This config option is here only for backward compatibility. ext3
          filesystem is now handled by the ext4 driver.

}
...


## Usage

1. Clone the repository.
2. Build the project using Cargo.
3. Run the compiled binary, passing the path to the Linux kernel source code as an argument.
