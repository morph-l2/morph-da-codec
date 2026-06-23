UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

ifeq ($(UNAME_S),Darwin)
  GOOS := darwin
else ifeq ($(UNAME_S),Linux)
  GOOS := linux
else
  GOOS := unsupported
endif

ifeq ($(UNAME_M),arm64)
  GOARCH := arm64
else ifeq ($(UNAME_M),aarch64)
  GOARCH := arm64
else ifeq ($(UNAME_M),x86_64)
  GOARCH := amd64
else
  GOARCH := unsupported
endif

FFI_LIB := target/release/libmorph_da_zstd_decode.a
GO_LIB := bindings/decoder/zstd/libmorph_da_zstd_decode_$(GOOS)_$(GOARCH).a

.PHONY: build-zstd-ffi install-decoder-bindings test rust-test go-test clean

build-zstd-ffi:
	cargo build -p morph-da-decoder-ffi --release

install-decoder-bindings: build-zstd-ffi
	cp $(FFI_LIB) $(GO_LIB)

rust-test:
	cargo test --workspace

go-test: install-decoder-bindings
	cd bindings/decoder && go test ./...

test: rust-test go-test

clean:
	cargo clean
	rm -f bindings/decoder/zstd/*.a
