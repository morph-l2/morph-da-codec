# morph-da-codec

Morph DA zstd codec (encoder + decoder) shared by Rust zkVM guest code and Go node code.

The encoder and decoder are compiled into a single staticlib and exposed to Go through the `bindings/codec/zstd` package:

- `CompressMorphDABatch` — compress a batch into a magic-less zstd frame;
- `DecompressMorphDABatch` — decompress a frame produced by the encoder.

## Semantics

Encoder and decoder share the same frame format:

- the frame omits the zstd magic bytes (`28 b5 2f fd`); the decoder prepends them internally;
- a single frame is produced, with content size included so the decoder can validate it;
- literal compression is disabled (raw literal bytes) and the window log is capped at 17.

Decoder specifics:

- only the first complete frame is decoded; bytes after it are ignored;
- first frame content size is required and validated.

## Build

```sh
make build-zstd-ffi
make install-codec-bindings
make test
```
