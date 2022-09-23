# flb-plugin - Fluent Bit plugin binding for Rust

## Example: hello

[`hello`](./hello) is an example plugin. It prints out data passed from Fluent Bit.

Fluent Bit plugins are built as shared objects. Therefore, you get `libhello.so` after the build.
```console
$ cargo build --package hello
$ file target/debug/libhello.so
target/debug/libhello.so: ELF 64-bit LSB shared object, x86-64, version 1 (SYSV), dynamically linked, BuildID[sha1]=c07bab6d70349cedc0634989b526f19a7dfe9857, with debug_info, not stripped
```

Now, you can start Fluent Bit with hello plugin.
```console
$ /path/to/fluent-bit -e target/debug/libhello.so -i cpu -o hello -p param=world
Fluent Bit v1.9.8
* Copyright (C) 2015-2022 The Fluent Bit Authors
* Fluent Bit is a CNCF sub-project under the umbrella of Fluentd
* https://fluentbit.io

[2022/09/24 02:40:28] [ info] [fluent bit] version=1.9.8, commit=, pid=2109764
[2022/09/24 02:40:28] [ info] [storage] version=1.2.0, type=memory-only, sync=normal, checksum=disabled, max_chunks_up=128
[2022/09/24 02:40:28] [ info] [cmetrics] version=0.3.6
[new] param: Some("world")
[2022/09/24 02:40:28] [ info] [sp] stream processor started
[flush] tag: cpu.0, data: Array([Ext(0, [99, 45, 239, 141, 48, 221, 237, 26]), Map([(String(Utf8StringRef { s: Ok("cpu_p") }), F64(0.3333333333333333)), ...TRUNCATED... (String(Utf8StringRef { s: Ok("cpu23.p_system") }), F64(0.0))])])
^C[2022/09/24 02:40:31] [engine] caught signal (SIGINT)
[2022/09/24 02:40:31] [ info] [input] pausing cpu.0
[2022/09/24 02:40:31] [ warn] [engine] service will shutdown in max 5 seconds
[exit]
[2022/09/24 02:40:31] [ info] [engine] service has stopped (0 pending tasks)
```
