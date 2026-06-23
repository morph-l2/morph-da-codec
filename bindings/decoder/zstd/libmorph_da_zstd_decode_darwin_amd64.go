//go:build darwin && amd64 && cgo && TODO

package zstd

/*
#cgo LDFLAGS: ${SRCDIR}/libmorph_da_zstd_decode_darwin_amd64.a -framework Security -framework SystemConfiguration
*/
import "C"
