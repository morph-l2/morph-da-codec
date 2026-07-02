package zstd

/*
#include <stdint.h>

const char* morph_da_zstd_compress_bound(
    const uint8_t* src,
    uint64_t src_size,
    uint64_t* output_size
);

const char* morph_da_zstd_compress(
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

var ErrInvalidCompressedLength = errors.New("invalid compressed length")

const outputBufferTooSmall = "output buffer too small"

// CompressMorphDABatch compresses a Morph DA batch into a single magic-less
// zstd frame, matching the parameters DecompressMorphDABatch expects.
func CompressMorphDABatch(batch []byte) ([]byte, error) {
	if len(batch) == 0 {
		return nil, ErrEmptyInput
	}

	src := (*C.uint8_t)(unsafe.Pointer(unsafe.SliceData(batch)))
	srcSize := C.uint64_t(len(batch))

	var bound C.uint64_t
	if err := C.morph_da_zstd_compress_bound(src, srcSize, &bound); err != nil {
		return nil, encodeError(err)
	}

	boundLen := uint64(bound)
	if uint64(int(boundLen)) != boundLen {
		return nil, ErrInvalidCompressedLength
	}

	output, outputSize, err := compressInto(src, srcSize, bound)
	if err != nil {
		neededLen := uint64(outputSize)
		if C.GoString(err) != outputBufferTooSmall || neededLen <= boundLen {
			return nil, encodeError(err)
		}
		if uint64(int(neededLen)) != neededLen {
			return nil, ErrInvalidCompressedLength
		}

		output, outputSize, err = compressInto(src, srcSize, C.uint64_t(neededLen))
	}
	if err != nil {
		return nil, encodeError(err)
	}

	outputLen := uint64(outputSize)
	if outputLen > uint64(len(output)) {
		return nil, ErrInvalidCompressedLength
	}
	if uint64(int(outputLen)) != outputLen {
		return nil, ErrInvalidCompressedLength
	}

	return output[:int(outputLen)], nil
}

func compressInto(src *C.uint8_t, srcSize C.uint64_t, capacity C.uint64_t) ([]byte, C.uint64_t, *C.char) {
	output := make([]byte, int(uint64(capacity)))
	var outputSize = capacity
	var outputPtr *C.uint8_t
	if len(output) != 0 {
		outputPtr = (*C.uint8_t)(unsafe.Pointer(unsafe.SliceData(output)))
	}

	err := C.morph_da_zstd_compress(src, srcSize, outputPtr, &outputSize)
	return output, outputSize, err
}

// compressIntoBuf is a cgo-free wrapper around compressInto for tests, which
// cannot import "C". It returns the raw FFI error text ("" on success) so the
// retry preconditions in CompressMorphDABatch can be asserted directly.
func compressIntoBuf(batch []byte, capacity uint64) ([]byte, uint64, string) {
	src := (*C.uint8_t)(unsafe.Pointer(unsafe.SliceData(batch)))
	output, outputSize, err := compressInto(src, C.uint64_t(len(batch)), C.uint64_t(capacity))
	if err == nil {
		return output, uint64(outputSize), ""
	}
	return output, uint64(outputSize), C.GoString(err)
}

func encodeError(err *C.char) error {
	return fmt.Errorf("morph da zstd encode: %s", C.GoString(err))
}
