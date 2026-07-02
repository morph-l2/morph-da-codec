package zstd

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"testing"
)

func TestCompressMorphDABatchRoundTrip(t *testing.T) {
	original := bytes.Repeat([]byte("morph da batch payload \x00\x01\x02 round trip "), 64)

	compressed, err := CompressMorphDABatch(original)
	if err != nil {
		t.Fatal(err)
	}
	if len(compressed) == 0 {
		t.Fatal("compressed output is empty")
	}
	// Magic-less frame: must not start with the zstd magic bytes 0x28 0xB5 0x2F 0xFD.
	if bytes.HasPrefix(compressed, []byte{0x28, 0xB5, 0x2F, 0xFD}) {
		t.Fatal("compressed output unexpectedly carries the zstd magic bytes")
	}

	decoded, err := DecompressMorphDABatch(compressed)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(decoded, original) {
		t.Fatalf("round-trip mismatch: got %d bytes, want %d", len(decoded), len(original))
	}
}

func TestCompressMorphDABatchRejectsEmptyInput(t *testing.T) {
	_, err := CompressMorphDABatch(nil)
	if !errorsIs(err, ErrEmptyInput) {
		t.Fatalf("expected ErrEmptyInput, got %v", err)
	}
}

// TestCompressIntoTooSmallBufferReportsNeededSize validates the two
// preconditions the retry path in CompressMorphDABatch relies on: the FFI
// returns the exact "output buffer too small" error text, and writes the
// precise required size back through the output size pointer.
func TestCompressIntoTooSmallBufferReportsNeededSize(t *testing.T) {
	original := bytes.Repeat([]byte("morph da batch payload \x00\x01\x02 too small "), 64)

	compressed, err := CompressMorphDABatch(original)
	if err != nil {
		t.Fatal(err)
	}

	_, neededSize, errMsg := compressIntoBuf(original, 1)
	if errMsg == "" {
		t.Fatal("expected an error for a 1-byte output buffer")
	}
	if errMsg != outputBufferTooSmall {
		t.Fatalf("error text mismatch: got %q, want %q", errMsg, outputBufferTooSmall)
	}
	if want := uint64(len(compressed)); neededSize != want {
		t.Fatalf("needed size mismatch: got %d, want %d", neededSize, want)
	}

	// A retry with the reported size must succeed and fill the buffer exactly.
	output, writtenSize, errMsg := compressIntoBuf(original, neededSize)
	if errMsg != "" {
		t.Fatalf("retry with exact capacity failed: %s", errMsg)
	}
	if writtenSize != neededSize {
		t.Fatalf("written size mismatch: got %d, want %d", writtenSize, neededSize)
	}
	if !bytes.Equal(output[:writtenSize], compressed) {
		t.Fatal("retry output differs from single-shot compression output")
	}
}

func TestCompressMorphDABatchHelloWorldHash(t *testing.T) {
	compressed, err := CompressMorphDABatch([]byte("hello world"))
	if err != nil {
		t.Fatal(err)
	}
	if len(compressed) == 0 {
		t.Fatal("compressed output is empty")
	}

	hash := sha256.Sum256(compressed)
	if got, want := hex.EncodeToString(hash[:]), "5850cf750ce5ac2ccea3d4d0baeb85e3645297ca72df11c7c1c2b2dfbc1eb015"; got != want {
		t.Fatalf("compressed hash mismatch: got %s, want %s", got, want)
	}
}
