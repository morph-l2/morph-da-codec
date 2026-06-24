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
LINUX_AMD64_LIB := bindings/decoder/zstd/libmorph_da_zstd_decode_linux_amd64.a
CENTOS7_BUILD_OUT := build/out
CENTOS7_FFI_LIB := $(CENTOS7_BUILD_OUT)/libmorph_da_zstd_decode_linux_amd64.a
DARWIN_DEPLOYMENT_TARGET ?= 11.0

ifeq ($(GOOS),darwin)
  FFI_BUILD_ENV := MACOSX_DEPLOYMENT_TARGET=$(DARWIN_DEPLOYMENT_TARGET)
endif

.PHONY: build-zstd-ffi install-decoder-bindings build-zstd-ffi-linux-amd64 install-decoder-bindings-linux-amd64 linux-amd64-decoder-bindings test rust-test go-test clean

build-zstd-ffi:
	$(FFI_BUILD_ENV) cargo build -p morph-da-decoder-ffi --release

install-decoder-bindings: build-zstd-ffi
	cp $(FFI_LIB) $(GO_LIB)

build-zstd-ffi-linux-amd64:
	DOCKER_BUILDKIT=1 docker build \
		-f build/Dockerfile \
		--target artifact \
		--output type=local,dest=$(CENTOS7_BUILD_OUT) \
		.

install-decoder-bindings-linux-amd64: build-zstd-ffi-linux-amd64
	cp $(CENTOS7_FFI_LIB) $(LINUX_AMD64_LIB)


rust-test:
	cargo test --workspace

go-test: install-decoder-bindings
	cd bindings/decoder && go test ./...

test: rust-test go-test

clean:
	cargo clean
	rm -f bindings/decoder/zstd/*.a
	rm -rf $(CENTOS7_BUILD_OUT)
