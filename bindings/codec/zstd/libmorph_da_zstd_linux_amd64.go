//go:build linux && amd64 && cgo && !musl

package zstd

/*
#cgo LDFLAGS: ${SRCDIR}/libmorph_da_zstd_linux_amd64.a -ldl -lpthread -lm
*/
import "C"
