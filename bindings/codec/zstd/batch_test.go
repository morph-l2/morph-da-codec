package zstd

import (
	"encoding/hex"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"
)

const blobWidth = 4096

func TestDecompressesBatch(t *testing.T) {
	batchDirs, err := filepath.Glob("../../../testdata/batches/batch-*")
	if err != nil {
		t.Fatal(err)
	}
	sort.Strings(batchDirs)
	if len(batchDirs) == 0 {
		t.Fatal("no batch directories found")
	}

	for _, batchDir := range batchDirs {
		t.Run(filepath.Base(batchDir), func(t *testing.T) {
			blobPaths := blobPathsSorted(t, batchDir)
			if len(blobPaths) == 0 {
				t.Fatalf("no blob hex files found in %s", batchDir)
			}

			var batchData []byte
			for _, path := range blobPaths {
				batchData = append(batchData, decodeBlob(t, readHex(t, path))...)
			}

			decoded, err := DecompressMorphDABatch(batchData)
			if err != nil {
				t.Fatal(err)
			}
			t.Logf("decoded len %d", len(decoded))
		})
	}
}

func blobPathsSorted(t *testing.T, dir string) []string {
	t.Helper()

	paths, err := filepath.Glob(filepath.Join(dir, "blob-*.hex"))
	if err != nil {
		t.Fatal(err)
	}
	sort.Slice(paths, func(i, j int) bool {
		return blobIndex(paths[i]) < blobIndex(paths[j])
	})
	return paths
}

func blobIndex(path string) int {
	name := filepath.Base(path)
	name = strings.TrimPrefix(name, "blob-")
	name = strings.TrimSuffix(name, ".hex")

	var index int
	for _, r := range name {
		index = index*10 + int(r-'0')
	}
	return index
}

func readHex(t *testing.T, path string) []byte {
	t.Helper()

	raw, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}

	s := strings.TrimSpace(string(raw))
	s = strings.TrimPrefix(s, "0x")

	data, err := hex.DecodeString(s)
	if err != nil {
		t.Fatal(err)
	}
	return data
}

func decodeBlob(t *testing.T, blob []byte) []byte {
	t.Helper()

	if len(blob) != blobWidth*32 {
		t.Fatalf("invalid blob size: %d", len(blob))
	}

	out := make([]byte, blobWidth*31)
	for i := 0; i < blobWidth; i++ {
		chunk := blob[i*32 : (i+1)*32]
		if chunk[0] != 0 {
			t.Fatalf("non-zero high byte in field element %d", i)
		}
		copy(out[i*31:(i+1)*31], chunk[1:])
	}
	return out
}
