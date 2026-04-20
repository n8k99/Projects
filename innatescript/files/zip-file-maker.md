# G090 — Zip File Maker

> The Rosetta Stone's sixth **Files-category** project. Introduces **byte-level round-trip equivalence** — compress → decompress returns identical bytes, always. Uses **RLE** (run-length encoding) as the exemplar: simple enough to fit in five lines of decoder, visibly effective on repetitive data. Per-entry method selection: store vs compress, picked to minimise output size. This establishes the **archive as manifest + payload** split that every real format (ZIP, tar, OCI images) follows.

```yaml
id: G090
title: Zip File Maker
category: files
requires: [G029-hex-viewer, G085-quiz-maker, G088-sort-file-records]
provides: [byte-round-trip, rle-compression, per-entry-method-selection, archive-manifest]
```

## Insight: Byte-Level Round-Trip Is the Compression Contract

G085 proved that a quiz text file parses and re-serialises to the same text. G090 raises the bar: **every byte** of the input must be recoverable from the compressed output. No lossiness, no approximation. If the archive loses a single bit of a file, the file is corrupt.

That's a stricter contract than textual round-trip, because binary data can include bytes that mean different things in different encodings. The compressor must be indifferent to content — it sees bytes, not characters. The decoder reverses the transformation exactly.

First Rosetta Stone project where the **data is bytes, not text**. G085's quiz parser could normalise whitespace because the semantic content was unchanged; G090's RLE cannot normalise anything, because every byte is semantically load-bearing. The vault's planned binary-asset handling (images, PDFs, archives) will need this contract.

## Insight: RLE Is the Minimal Interesting Compression

G090 uses run-length encoding because it's the simplest compression that:
1. **Decodes trivially** (read count byte, emit that many copies — five lines).
2. **Visibly reduces** repetitive data (`xxxxxxx` → `7 x`, 7 bytes to 2).
3. **Visibly fails** on random data (every byte becomes two bytes).

That last point is important. RLE is terrible for general-purpose compression — it only helps when the input has actual runs. The Rosetta Stone picks up this trade-off explicitly: `add_file` runs RLE and compares the result to the raw size, picking whichever is smaller. **The compressor must know when not to compress.**

First Rosetta Stone project where **the algorithm makes a choice based on its own output size**. G053 simulated Fizzbuzz without choice; G071 parsed HTML without choice; G090's `add_file` explicitly branches on "did compression help?". This meta-level decision — the algorithm evaluating itself — recurs in every adaptive system (LSM compaction, JPEG quality tuning, etc).

## Insight: Per-Entry Method Is the Archive Design Pattern

Real archive formats (ZIP, 7z, tar.gz) don't choose one compression method for the whole archive. They let each entry pick the best method — or no method at all. A JPEG inside a ZIP is `STORED` because re-compressing a JPEG does nothing; a text file inside the same ZIP is `DEFLATED`.

G090 adopts the same pattern with two methods (Store, RLE). The archive header declares methods per entry; the decoder dispatches on the method tag. Adding LZ77, Huffman, Deflate later is a matter of:
1. Adding the method variant.
2. Implementing encode/decode.
3. Updating `add_file` to consider the new method.

The **structure** of the archive doesn't change — entries always have `(name, method, size_original, data)`. The pattern is stable even as the compression menu grows.

First Rosetta Stone project where **the archive is an extensible dispatch table**. Every real file format uses this structure (PNG chunks, PDF objects, OCI image layers). G090 introduces it.

## Insight: Archive = Manifest + Payload

An archive is two conceptual parts: a **manifest** (what's in it, how it's stored, how big it is) and the **payload** (the actual compressed bytes). In-memory, they live together; on disk, they could be split. The vault's future binary handling may separate them: manifest in Postgres, payload bytes in object storage.

`compressed_size` and `compression_ratio` read only the manifest. `extract` reads the manifest to find the right method, then reads the payload to decode. This separation is load-bearing: it lets lightweight operations (list contents, check if file exists) run without touching payload, and it means the payload's representation (bytes, file handles, URL references) can change without breaking the manifest API.

First Rosetta Stone project with an explicit **manifest-vs-payload split**. G082's CMS had an implicit manifest (revisions as a list); G090 makes it the primary abstraction. Every indexed storage system (databases, S3, Git object store) layers this split over the raw bytes.

## Choreographic Case: Vault Bundle

```innate
(@vault-bundle){
  @bundle <- @archive/new{}
  @notes <- @vault/find{path: "*.md"}
  @for note in @notes {
    @bytes <- @file/read-bytes{path: @note.path}
    @archive/add-file{archive: @bundle, name: @note.path, data: @bytes}
  }

  @manifest <- @archive/manifest{archive: @bundle}
  ;; -> [{name, method, size_original, size_stored} ...]
  @ratio <- @archive/compression-ratio{archive: @bundle}

  @vault/save-bytes{path: "backups/${@now}.zb",
                     content: @archive/serialise{archive: @bundle}}
}
```

A bundle of vault notes, each compressed individually, packed into a single `.zb` file. Manifest-first reading means a UI can display what's in the bundle without decompressing everything.

## Structures

```innate
(defenum method STORE | RLE)

(defstruct entry
  name          : String
  method        : Method
  size-original : Int
  data          : Bytes)

(defstruct archive
  entries : [Entry])
```

## Resolver Natives

```innate
@rle/encode{data}                    -> Bytes
@rle/decode{encoded}                 -> Bytes
@archive/new{}                       -> Archive
@archive/add-file{archive, name, data}    -> Unit
@archive/add-stored{archive, name, data}  -> Unit
@archive/extract{archive, name}       -> Bytes | null
@archive/names{archive}               -> [String]
@archive/compressed-size{archive}     -> Int
@archive/compression-ratio{archive}   -> Float
```

## Demo

```innate
(@demo){
  @a <- @archive/new{}
  @archive/add-file{archive: @a, name: "repetitive.txt", data: "aaaaaaaaaa"}
  ;; -> method=RLE (10 bytes -> 2 bytes)

  @archive/add-file{archive: @a, name: "random.bin", data: [1,2,3,4,5]}
  ;; -> method=STORE (RLE would double to 10; keep original 5)

  @archive/add-file{archive: @a, name: "big.txt", data: "x" repeat 1000}
  ;; -> method=RLE (1000 bytes -> 8 bytes: 4 runs of 255+235)

  @archive/extract{archive: @a, name: "repetitive.txt"}  ;; -> "aaaaaaaaaa"
  @archive/compression-ratio{archive: @a}                 ;; -> ~0.04
}
```

## Where

Every byte MUST round-trip exactly — binary fidelity is the compression contract; losing a single bit corrupts the file. `add_file` MUST pick the smaller of Store vs RLE per entry — a compressor that always compresses doubles the size of random data, and an archive cannot afford that. The method tag MUST be per entry, NOT per archive — real formats (ZIP, 7z) choose per entry because payloads are heterogeneous (already-compressed JPEGs beside plain text). The manifest (names, methods, sizes) MUST be readable without decompressing any payload — UIs need to list contents fast, and separation of manifest from payload is the primitive that enables it. RLE runs MUST cap at 255 (one-byte count field) — a two-byte count would trade simpler encoding for headers that waste space on short runs. Decoder MUST tolerate an odd-length encoded buffer by stopping before the incomplete pair — robustness across truncated inputs.
