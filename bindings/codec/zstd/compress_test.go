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
