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

	output := make([]byte, int(boundLen))
	var outputSize = bound
	var outputPtr *C.uint8_t
	if len(output) != 0 {
		outputPtr = (*C.uint8_t)(unsafe.Pointer(unsafe.SliceData(output)))
	}

	if err := C.morph_da_zstd_compress(src, srcSize, outputPtr, &outputSize); err != nil {
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

func encodeError(err *C.char) error {
	return fmt.Errorf("morph da zstd encode: %s", C.GoString(err))
}
