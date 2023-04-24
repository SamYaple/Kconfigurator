# Kconfigurator

Kconfigurator is a tool for parsing and analyzing Kconfig options. It provides a simple and efficient way to extract Kconfig information from the Linux kernel source code.

## Example Output

The following is an example output of the Kconfigurator tool:

```plaintext
~/repos/kconfigurator $ ./target/release/kconfigurator /tmp/linux/ | head -n80
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
...


## Usage

1. Clone the repository.
2. Build the project using Cargo.
3. Run the compiled binary, passing the path to the Linux kernel source code as an argument.
