//go:build linux && arm64 && cgo && TODO

package zstd

/*
#cgo LDFLAGS: ${SRCDIR}/libmorph_da_zstd_decode_linux_arm64.a -ldl -lpthread -lm
*/
import "C"
