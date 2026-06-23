# Morph DA Codec Go Integration Guide

This document explains how to integrate the Morph DA first-frame zstd decoder into Go projects.

## Package Paths

Go module:

```text
github.com/morph-l2/morph-da-codec/bindings/decoder
```

Decoder package path:

```text
github.com/morph-l2/morph-da-codec/bindings/decoder/zstd
```

Core API:

```go
func DecompressMorphDABatch(payload []byte) ([]byte, error)
```

## Adding the Dependency

### Option 1: Import directly through Go modules

Run the following command in your application project:

```sh
go get github.com/morph-l2/morph-da-codec/bindings/decoder@<version-or-commit>
```

Alternatively, add it manually to `go.mod`:

```go
require github.com/morph-l2/morph-da-codec/bindings/decoder v0.0.0-<pseudo-version>
```

Then import it in your code:

```go
import "github.com/morph-l2/morph-da-codec/bindings/decoder/zstd"
```

### Option 2: Local development integration

If your application project needs to integrate with local source code during development, add a `replace` directive to the application's `go.mod`:

```go
require github.com/morph-l2/morph-da-codec/bindings/decoder v0.0.0

replace github.com/morph-l2/morph-da-codec/bindings/decoder => /absolute/path/to/morph-da-codec/bindings/decoder
```

## Usage Example

```go
package main

import (
	"fmt"
	"log"

	"github.com/morph-l2/morph-da-codec/bindings/decoder/zstd"
)

func main() {
	var payload []byte

	decoded, err := zstd.DecompressMorphDABatch(payload)
	if err != nil {
		log.Fatalf("decompress morph da batch: %v", err)
	}

	fmt.Printf("decoded size: %d\n", len(decoded))
}
```

If `payload` is empty, an `empty input` error is returned.

If `payload` is `[0, 0, 0, ...]`, an empty byte slice is returned, which represents an empty batch.

## Native Static Library Requirements

This Go binding uses cgo to call a Rust FFI static library. Integrators must ensure that the static library for the target platform exists in the `bindings/decoder/zstd` package directory:

```text
libmorph_da_zstd_decode_${GOOS}_${GOARCH}.a
```

The current Go binding declares cgo link configurations for the following platforms:

- `darwin/amd64`
- `darwin/arm64`
- `linux/amd64`
- `linux/arm64`

Generate and install the static library for the current host platform in this repository:

```sh
make install-decoder-bindings
```

Run Go tests:

```sh
make go-test
```

Run the full test suite:

```sh
make test
```

## cgo Notes

Application projects must enable cgo:

```sh
CGO_ENABLED=1 go test ./...
CGO_ENABLED=1 go build ./...
```

For cross-compilation, all of the following requirements must be satisfied:

1. `GOOS` / `GOARCH` must match the target platform;
2. `CGO_ENABLED=1`;
3. A usable C toolchain for the target platform must be installed on the build host;
4. The target platform's `libmorph_da_zstd_decode_${GOOS}_${GOARCH}.a` must exist under `bindings/decoder/zstd`;
5. The Rust FFI static library must be correctly built for the target platform.

## Decoding Semantics

`DecompressMorphDABatch` takes a Morph DA batch compressed payload as input. Its semantics are as follows:

- The input payload does not include zstd magic bytes;
- The decoder automatically prepends the zstd magic bytes internally: `28 b5 2f fd`;
- Only the first complete zstd frame is decoded;
- Trailing bytes after the first frame are ignored;
- The first frame must contain a content size, and the decompressed length is validated.

## Error Handling

Typical errors that may be returned:

- `zstd.ErrEmptyInput`: the input is empty;
- `zstd.ErrInvalidDecompressedLength`: the decompressed length returned by the FFI is invalid or exceeds the representable range of `int` on the current Go platform;
- Other errors: decoding errors returned by the underlying zstd/Rust FFI layer. Their error messages are prefixed with `morph da zstd decode:`.

Example:

```go
decoded, err := zstd.DecompressMorphDABatch(payload)
if err != nil {
	if errors.Is(err, zstd.ErrEmptyInput) {
		// handle empty payload
	}
	return err
}

_ = decoded
```

## Security and Resource Considerations

- The decompressed output buffer is allocated at once based on the frame content size. Integrators should avoid calling this API without limits on untrusted oversized payloads;
- The upper-layer application is advised to validate the compressed payload size, source, and batch boundaries before calling this API;
- Trailing bytes are ignored. If the application semantics require the payload to contain exactly one frame, additional validation must be performed by the upper layer;
- This binding depends on a native static library. Deployment images or build artifacts must include the platform-specific `.a` file and a cgo build environment.