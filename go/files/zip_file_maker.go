// Zip File Maker — archive multiple named files with per-entry RLE compression.
package files

// ZipMethod is the storage method for a zip entry.
type ZipMethod int

const (
	ZMStore ZipMethod = 0
	ZMRle   ZipMethod = 1
)

// ZipEntry is one archived file.
type ZipEntry struct {
	Name         string
	Method       ZipMethod
	SizeOriginal int
	Data         []byte
}

// Archive is a list of zip entries.
type Archive struct {
	Entries []ZipEntry
}

// NewArchive returns an empty archive.
func NewArchive() *Archive {
	return &Archive{Entries: []ZipEntry{}}
}

// AddFile picks the smaller of Store or RLE per file.
func (a *Archive) AddFile(name string, data []byte) {
	rle := RleEncode(data)
	if len(rle) < len(data) {
		a.Entries = append(a.Entries, ZipEntry{
			Name: name, Method: ZMRle,
			SizeOriginal: len(data), Data: rle,
		})
	} else {
		a.Entries = append(a.Entries, ZipEntry{
			Name: name, Method: ZMStore,
			SizeOriginal: len(data), Data: append([]byte{}, data...),
		})
	}
}

// AddStored forces uncompressed storage.
func (a *Archive) AddStored(name string, data []byte) {
	a.Entries = append(a.Entries, ZipEntry{
		Name: name, Method: ZMStore,
		SizeOriginal: len(data), Data: append([]byte{}, data...),
	})
}

// Extract returns the original bytes for an entry, or nil if missing.
func (a *Archive) Extract(name string) []byte {
	for _, e := range a.Entries {
		if e.Name == name {
			if e.Method == ZMStore {
				out := make([]byte, len(e.Data))
				copy(out, e.Data)
				return out
			}
			return RleDecode(e.Data)
		}
	}
	return nil
}

// Names returns all entry names in order.
func (a *Archive) Names() []string {
	out := make([]string, len(a.Entries))
	for i, e := range a.Entries {
		out[i] = e.Name
	}
	return out
}

// CompressedSize is the sum of stored bytes across all entries.
func (a *Archive) CompressedSize() int {
	total := 0
	for _, e := range a.Entries {
		total += len(e.Data)
	}
	return total
}

// CompressionRatio is compressed/original, 1.0 if nothing original.
func (a *Archive) CompressionRatio() float64 {
	orig := 0
	for _, e := range a.Entries {
		orig += e.SizeOriginal
	}
	if orig == 0 {
		return 1.0
	}
	return float64(a.CompressedSize()) / float64(orig)
}

// RleEncode produces (count, byte) pairs. Max run = 255.
func RleEncode(data []byte) []byte {
	out := []byte{}
	i := 0
	for i < len(data) {
		b := data[i]
		run := 1
		for i+run < len(data) && data[i+run] == b && run < 255 {
			run++
		}
		out = append(out, byte(run), b)
		i += run
	}
	return out
}

// RleDecode reverses RleEncode.
func RleDecode(encoded []byte) []byte {
	out := []byte{}
	for i := 0; i+1 < len(encoded); i += 2 {
		count, b := int(encoded[i]), encoded[i+1]
		for j := 0; j < count; j++ {
			out = append(out, b)
		}
	}
	return out
}
