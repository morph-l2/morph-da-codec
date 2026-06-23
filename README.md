# morph-da-codec

Morph DA zstd decoder shared by Rust zkVM guest code and Go node code.(There will be encoder in the future)

## Semantics

- input payload omits zstd magic bytes;
- decoder prepends `28 b5 2f fd` internally;
- only the first complete zstd frame is decoded;
- bytes after the first frame are ignored;
- first frame content size is required and validated.

## Build

```sh
make build-zstd-ffi
make install-decoder-bindings
make test
```
