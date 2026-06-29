package zstd

import (
	"bytes"
	"os"
	"strings"
	"testing"
)

func loadHelloPayload(t *testing.T) []byte {
	t.Helper()

	payload, err := os.ReadFile("../../../testdata/any/text-1.bin")
	if err != nil {
		t.Fatal(err)
	}
	return payload
}

func TestDecompressMorphDABatch(t *testing.T) {
	payload := loadHelloPayload(t)

	decoded, err := DecompressMorphDABatch(payload)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(decoded, []byte("hello world")) {
		t.Fatalf("decoded mismatch: %q", decoded)
	}
}

func TestDecompressMorphDABatchIgnoresTrailingBytes(t *testing.T) {
	payload := append(loadHelloPayload(t), 0x05)

	decoded, err := DecompressMorphDABatch(payload)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(decoded, []byte("hello world")) {
		t.Fatalf("decoded mismatch: %q", decoded)
	}
}

func TestDecompressMorphDABatchIgnoresTrailingPayload(t *testing.T) {
	payload := loadHelloPayload(t)
	payload = append(payload, payload...)

	decoded, err := DecompressMorphDABatch(payload)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(decoded, []byte("hello world")) {
		t.Fatalf("decoded mismatch: %q", decoded)
	}
}

func TestDecompressMorphDABatchComplexStr(t *testing.T) {
	payload, err := os.ReadFile("../../../testdata/any/text-2.bin")
	if err != nil {
		t.Fatal(err)
	}

	decoded, err := DecompressMorphDABatch(payload)
	if err != nil {
		t.Fatal(err)
	}
	if want := []byte("TEXT: Jun 23 2026 22:26:07 PM (+08:00 UTC)"); !bytes.Equal(decoded, want) {
		t.Fatalf("decoded mismatch: %q", decoded)
	}
}

func TestDecompressMorphDABatchBlobs(t *testing.T) {
	payload, err := os.ReadFile("../../../testdata/any/20-blobs.bin")
	if err != nil {
		t.Fatal(err)
	}

	decoded, err := DecompressMorphDABatch(payload)
	if err != nil {
		t.Fatal(err)
	}
	if want := []byte("76f91869161dc4348230d5f60883dd17462035f4"); !bytes.Equal(decoded[:len(want)], want) {
		t.Fatalf("decoded prefix mismatch: %q", decoded[:len(want)])
	}
	if want := 2621440; len(decoded) != want {
		t.Fatalf("decoded length mismatch: got %d, want %d", len(decoded), want)
	}
}

func TestDecompressMorphDABatchRejectsEmptyInput(t *testing.T) {
	_, err := DecompressMorphDABatch(nil)
	if !errorsIs(err, ErrEmptyInput) {
		t.Fatalf("expected ErrEmptyInput, got %v", err)
	}
}

func TestDecompressMorphDABatchRejectsEmptyFramePayload(t *testing.T) {
	payload := []byte{0x20, 0x00, 0x01, 0x00, 0x00}

	_, err := DecompressMorphDABatch(payload)
	if err == nil || !strings.Contains(err.Error(), "invalid frame") {
		t.Fatalf("expected invalid frame error, got %v", err)
	}
}

func TestDecompressMorphDABatchRejectsCorruptedFrame(t *testing.T) {
	payload := loadHelloPayload(t)
	payload = payload[:len(payload)-1]

	_, err := DecompressMorphDABatch(payload)
	if err == nil || !strings.Contains(err.Error(), "invalid frame") {
		t.Fatalf("expected invalid frame error, got %v", err)
	}
}

func errorsIs(err, target error) bool {
	return err != nil && target != nil && err.Error() == target.Error()
}
