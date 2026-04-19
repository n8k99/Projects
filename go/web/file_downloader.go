// File Downloader — resumable transfer with atomic file creation.
//
// Bytes flow to <dest>.partial during transfer. On success, os.Rename promotes
// the file in one syscall. On interruption, .partial persists; the next call
// resumes from its size. Optional hash verification refuses the rename on
// mismatch and keeps .partial for inspection.
package web

import (
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
)

// DLTransport is a source of bytes supporting ranged reads. ReadFrom returns
// io.EOF at the end; other errors abort the download but leave .partial intact.
type DLTransport interface {
	ReadFrom(offset int64, buf []byte) (int, error)
}

// DLOpts configures the downloader.
type DLOpts struct {
	Resume       bool
	ChunkSize    int
	ExpectedHash *uint64 // nil = no verification
}

// DefaultDLOpts returns sensible defaults.
func DefaultDLOpts() DLOpts { return DLOpts{Resume: true, ChunkSize: 8192} }

// DLOutcome is the result of a successful download.
type DLOutcome struct {
	BytesWritten int64
	FinalPath    string
	Verified     bool
}

// DLHashMismatch is returned when ExpectedHash is set and the final bytes don't match.
type DLHashMismatch struct{ Expected, Actual uint64 }

func (e *DLHashMismatch) Error() string {
	return fmt.Sprintf("hash mismatch: expected %x, got %x", e.Expected, e.Actual)
}

// DLByteSumHash is a streamable integrity hash (not cryptographic).
func DLByteSumHash(b []byte) uint64 {
	var h uint64
	for _, byte_ := range b {
		h = h*31 + uint64(byte_)
	}
	return h + uint64(len(b))
}

// Download streams bytes from t into dest via dest.partial with atomic rename.
func Download(t DLTransport, dest string, opts DLOpts,
	onProgress func(int64)) (*DLOutcome, error) {
	if opts.ChunkSize <= 0 {
		opts.ChunkSize = 8192
	}
	if onProgress == nil {
		onProgress = func(int64) {}
	}
	partial := partialPath(dest)

	var startOffset int64
	var hashState uint64
	var bytesHashed int64

	if opts.Resume {
		if info, err := os.Stat(partial); err == nil {
			existing, err := os.ReadFile(partial)
			if err != nil {
				return nil, err
			}
			for _, b := range existing {
				hashState = hashState*31 + uint64(b)
			}
			startOffset = info.Size()
			bytesHashed = startOffset
		}
	}

	flag := os.O_WRONLY | os.O_CREATE
	if startOffset > 0 {
		flag |= os.O_APPEND
	} else {
		flag |= os.O_TRUNC
	}
	f, err := os.OpenFile(partial, flag, 0644)
	if err != nil {
		return nil, err
	}

	buf := make([]byte, opts.ChunkSize)
	total := startOffset
	for {
		n, readErr := t.ReadFrom(total, buf)
		if n > 0 {
			if _, err := f.Write(buf[:n]); err != nil {
				f.Close()
				return nil, err
			}
			for _, b := range buf[:n] {
				hashState = hashState*31 + uint64(b)
			}
			bytesHashed += int64(n)
			total += int64(n)
			onProgress(total)
		}
		if readErr == io.EOF {
			break
		}
		if readErr != nil {
			f.Close()
			return nil, readErr
		}
		if n == 0 {
			break
		}
	}
	if err := f.Sync(); err != nil {
		f.Close()
		return nil, err
	}
	f.Close()

	actual := hashState + uint64(bytesHashed)
	if opts.ExpectedHash != nil && *opts.ExpectedHash != actual {
		return nil, &DLHashMismatch{Expected: *opts.ExpectedHash, Actual: actual}
	}
	if err := os.Rename(partial, dest); err != nil {
		return nil, err
	}
	return &DLOutcome{
		BytesWritten: total, FinalPath: dest,
		Verified: opts.ExpectedHash != nil,
	}, nil
}

func partialPath(dest string) string {
	return filepath.Clean(dest) + ".partial"
}

// Ensure error types assert correctly.
var _ error = (*DLHashMismatch)(nil)
var _ = errors.New
