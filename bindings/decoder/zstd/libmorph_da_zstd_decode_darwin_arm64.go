//go:build darwin && arm64 && cgo

package zstd

/*
#cgo LDFLAGS: ${SRCDIR}/libmorph_da_zstd_decode_darwin_arm64.a -framework Security -framework SystemConfiguration
*/
import "C"
