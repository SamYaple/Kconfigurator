# Kconfigurator

Kconfigurator is a tool for parsing and analyzing Kconfig options. It provides a simple and efficient way to extract Kconfig information from the Linux kernel source code.
All `Kconfig` and `Kconfig.*` files are matched, kconfig files with macros and kconfig files which are for tests are not being processed.
Config strings are not evaluated nor replaced, they are &str matched.

## Example Output

Currently, we just spew out rendered versions of our structs. If you're looking for magic, you'll have to dig into the source code at present.

```plaintext
config CEPH_FS
        tristate
        prompt Ceph distributed file system
        defaults n
        depends on INET
        select CEPH_LIB
        select LIBCRC32C
        select CRYPTO_AES
        select CRYPTO
        select NETFS_SUPPORT
        help
          Choose Y or M here to include support for mounting the
          experimental Ceph distributed file system.  Ceph is an extremely
          scalable file system designed to provide high performance,
          reliable access to petabytes of storage.

          More information at https://ceph.io/.

          If unsure, say N.


config CEPH_FS_POSIX_ACL
        bool
        prompt Ceph POSIX Access Control Lists
        depends on CEPH_FS
        select FS_POSIX_ACL
        help
          POSIX Access Control Lists (ACLs) support permissions for users and
          groups beyond the owner/group/world scheme.

          If you don't know what Access Control Lists are, say N


config CEPH_FS_SECURITY_LABEL
        bool
        prompt CephFS Security Labels
        depends on CEPH_FS && SECURITY
        help
          Security labels support alternative access control models
          implemented by security modules like SELinux. This option
          enables an extended attribute handler for file security
          labels in the Ceph filesystem.

          If you are not using a security module that requires using
          extended attributes for file security labels, say N.

config CEPH_FSCACHE
        bool
        prompt Enable Ceph client caching support
        depends on CEPH_FS=m && FSCACHE || CEPH_FS=y && FSCACHE=y
        help
          Choose Y here to enable persistent, read-only local
          caching support for Ceph clients using FS-Cache
...snip...
```

## Usage

1. Clone the repository.
2. Build the project using Cargo.
3. Run the compiled binary, passing the path to the Linux kernel source code as an argument.
