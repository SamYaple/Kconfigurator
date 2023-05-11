# Kconfigurator

Kconfigurator is a tool for parsing and analyzing Kconfig options. It provides a simple and efficient way to extract Kconfig information from the Linux kernel source code.
All `Kconfig` and `Kconfig.*` files are matched, kconfig files with macros and kconfig files which are for tests are not being processed.
Config strings are not evaluated nor replaced, they are &str matched.

## Example Output

The following is yaml output of the Kconfigurator tool:

```plaintext
~/repos/kconfigurator$ cargo run --release -- /path/to/linux
kconfigs:
  "/path/to/linux/fs/ceph/Kconfig":
    - name: "CEPH_FS"
      type: "tristate"
      description: "\"Ceph distributed file system\""
      depends:
        - "INET"
      defaults:
        - "n"
      selects:
        - expression: "CEPH_LIB"
        - expression: "LIBCRC32C"
        - expression: "CRYPTO_AES"
        - expression: "CRYPTO"
        - expression: "NETFS_SUPPORT"
      help: |
        Choose Y or M here to include support for mounting the
        experimental Ceph distributed file system.  Ceph is an extremely
        scalable file system designed to provide high performance,
        reliable access to petabytes of storage.

        More information at https://ceph.io/.

        If unsure, say N.

    - name: "CEPH_FS_POSIX_ACL"
      type: "bool"
      description: "\"Ceph POSIX Access Control Lists\""
      depends:
        - "CEPH_FS"
      selects:
        - expression: "FS_POSIX_ACL"
      help: |
        POSIX Access Control Lists (ACLs) support permissions for users and
        groups beyond the owner/group/world scheme.

        If you don't know what Access Control Lists are, say N

    - name: "CEPH_FS_SECURITY_LABEL"
      type: "bool"
      description: "\"CephFS Security Labels\""
      depends:
        - "CEPH_FS && SECURITY"
      help: |
        Security labels support alternative access control models
        implemented by security modules like SELinux. This option
        enables an extended attribute handler for file security
        labels in the Ceph filesystem.

        If you are not using a security module that requires using
        extended attributes for file security labels, say N.
...snip...
```

## Usage

1. Clone the repository.
2. Build the project using Cargo.
3. Run the compiled binary, passing the path to the Linux kernel source code as an argument.
