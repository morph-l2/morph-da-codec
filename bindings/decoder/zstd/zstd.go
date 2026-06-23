package zstd

/*
#include <stdint.h>

const char* morph_da_zstd_decompress_bound(
    const uint8_t* src,
    uint64_t src_size,
    uint64_t* output_size
);

const char* morph_da_zstd_decompress(
    const uint8_t* src,
    uint64_t src_size,
    uint8_t* output_buf,
    uint64_t* output_buf_size
);
*/
import "C"

import (
	"errors"
	"fmt"
	"unsafe"
)

var (
	ErrEmptyInput                = errors.New("empty input")
	ErrInvalidDecompressedLength = errors.New("invalid decompressed length")
)

func DecompressMorphDABatch(payload []byte) ([]byte, error) {
	if len(payload) == 0 {
		return nil, ErrEmptyInput
	}

	src := (*C.uint8_t)(unsafe.Pointer(unsafe.SliceData(payload)))
	srcSize := C.uint64_t(len(payload))

	var bound C.uint64_t
	if err := C.morph_da_zstd_decompress_bound(src, srcSize, &bound); err != nil {
		return nil, ffiError(err)
	}

	boundLen := uint64(bound)
	if boundLen == 0 {
		return []byte{}, nil
	}
	if uint64(int(boundLen)) != boundLen {
		return nil, ErrInvalidDecompressedLength
	}

	output := make([]byte, int(boundLen))
	var outputSize = bound
	var outputPtr *C.uint8_t
	if len(output) != 0 {
		outputPtr = (*C.uint8_t)(unsafe.Pointer(unsafe.SliceData(output)))
	}

	if err := C.morph_da_zstd_decompress(src, srcSize, outputPtr, &outputSize); err != nil {
		return nil, ffiError(err)
	}

	outputLen := uint64(outputSize)
	if outputLen > uint64(len(output)) {
		return nil, ErrInvalidDecompressedLength
	}
	if uint64(int(outputLen)) != outputLen {
		return nil, ErrInvalidDecompressedLength
	}

	return output[:int(outputLen)], nil
}

func ffiError(err *C.char) error {
	return fmt.Errorf("morph da zstd decode: %s", C.GoString(err))
}
