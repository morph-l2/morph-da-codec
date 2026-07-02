# Morph DA Codec Go Integration Guide

Go bindings for the Morph DA zstd codec (compress + decompress) via cgo over a Rust FFI static library.

## Package Paths

```text
module: github.com/morph-l2/morph-da-codec/bindings/codec
codec:  github.com/morph-l2/morph-da-codec/bindings/codec/zstd
```

## Core API

```go
func DecompressMorphDABatch(payload []byte) ([]byte, error)
func CompressMorphDABatch(batch []byte) ([]byte, error)
```

## Adding the Dependency

Import through Go modules:

```sh
go get github.com/morph-l2/morph-da-codec/bindings/codec@<version-or-commit>
```

For local development, add a `replace` directive to your application's `go.mod`:

```go
require github.com/morph-l2/morph-da-codec/bindings/codec v0.0.0

replace github.com/morph-l2/morph-da-codec/bindings/codec => /absolute/path/to/morph-da-codec/bindings/codec
```

Then import:

```go
import "github.com/morph-l2/morph-da-codec/bindings/codec/zstd"
```

## Decompression

```go
decoded, err := zstd.DecompressMorphDABatch(payload)
if err != nil {
	if errors.Is(err, zstd.ErrEmptyInput) {
		// handle empty payload
	}
	log.Fatalf("decompress morph da batch: %v", err)
}
```

Semantics:

- The input payload does not include the zstd magic bytes; the decoder prepends them internally (`28 b5 2f fd`).
- Only the first complete zstd frame is decoded; trailing bytes are ignored.
- The first frame must carry a content size, which is validated against the decompressed length.
- Empty input returns `ErrEmptyInput`. A `[0, 0, 0, ...]` payload returns an empty slice (an empty batch).

## Compression

```go
compressed, err := zstd.CompressMorphDABatch(batch)
if err != nil {
	log.Fatalf("compress morph da batch: %v", err)
}
```

Semantics:

- Produces a single magic-less zstd frame, matching what `DecompressMorphDABatch` expects (no `28 b5 2f fd` prefix).
- Empty input returns `ErrEmptyInput`.
- Round-trips: `DecompressMorphDABatch(CompressMorphDABatch(batch))` returns the original batch.

## Errors

- `zstd.ErrEmptyInput` — the input is empty.
- `zstd.ErrInvalidDecompressedLength` / `zstd.ErrInvalidCompressedLength` — the length returned by the FFI is invalid or exceeds the representable range of `int` on the current platform.
- Other errors come from the underlying Rust FFI layer, prefixed with `morph da zstd decode:` (decompress) or `morph da zstd encode:` (compress).

## Native Static Library

These bindings use cgo to call a Rust FFI static library. The platform-specific archive must exist in the `zstd` package directory:

```text
libmorph_da_zstd_${GOOS}_${GOARCH}.a
```

Declared cgo link configurations cover:

- `darwin/arm64`
- `linux/amd64`

Build and install the library for the current host, then test:

```sh
make install-codec-bindings   # generate + install the static library
make go-test                  # run Go tests
make test                     # full test suite
```

## cgo Notes

cgo must be enabled:

```sh
CGO_ENABLED=1 go build ./...
CGO_ENABLED=1 go test ./...
```

Cross-compilation additionally requires: matching `GOOS`/`GOARCH`, a C toolchain for the target platform, and the target's `libmorph_da_zstd_${GOOS}_${GOARCH}.a` built and present under `zstd`.

## Security and Resource Considerations

- Output buffers are sized from the frame content size (decompress) or the compress bound. Avoid calling these APIs on untrusted, unbounded payloads; validate payload size, source, and batch boundaries first.
- Trailing bytes are ignored on decompress. If your semantics require exactly one frame, validate that in the upper layer.
- Deployment images and build artifacts must include the platform-specific `.a` file and a cgo build environment.
