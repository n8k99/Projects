-- MP3-to-Wav Converter — audio buffer + resample + normalize + RIFF.

namespace Graphics.Audio

structure AudioBuffer where
  sampleRate : Nat
  channels : Nat
  samples : List Int := []
  deriving Repr

def AudioBuffer.frameCount (b : AudioBuffer) : Nat :=
  if b.channels == 0 then 0 else b.samples.length / b.channels

def AudioBuffer.durationMs (b : AudioBuffer) : Nat :=
  if b.sampleRate == 0 then 0
  else (b.frameCount * 1000) / b.sampleRate

structure Mp3Frame where
  sampleRate : Nat
  channels : Nat
  bitrateKbps : Nat := 128
  samplesPerFrame : Nat
  pcm : List Int := []
  deriving Repr

inductive CodecError where
  | noFrames | mixedFormats | emptyBuffer
  | unsupportedChannels | sampleRateZero
  deriving Repr, BEq

def decodeMp3ToBuffer (frames : List Mp3Frame) : Except CodecError AudioBuffer :=
  match frames with
  | [] => .error .noFrames
  | first :: _ =>
    let sr := first.sampleRate
    let ch := first.channels
    if sr == 0 then .error .sampleRateZero
    else if ch != 1 ∧ ch != 2 then .error .unsupportedChannels
    else
      let mixed := frames.any (fun f => f.sampleRate != sr ∨ f.channels != ch)
      if mixed then .error .mixedFormats
      else
        let samples := frames.flatMap (·.pcm)
        .ok { sampleRate := sr, channels := ch, samples }

def clampI16 (v : Int) : Int :=
  if v > 32767 then 32767 else if v < -32768 then -32768 else v

def resample (buf : AudioBuffer) (dstRate : Nat) : Except CodecError AudioBuffer :=
  if buf.sampleRate == 0 ∨ dstRate == 0 then .error .sampleRateZero
  else if dstRate == buf.sampleRate then
    .ok { sampleRate := buf.sampleRate, channels := buf.channels, samples := buf.samples }
  else
    let inFrames := buf.frameCount
    if inFrames == 0 then
      .ok { sampleRate := dstRate, channels := buf.channels, samples := [] }
    else
      let outFrames := (inFrames * dstRate + buf.sampleRate / 2) / buf.sampleRate
      let samplesArr := buf.samples.toArray
      let channels := buf.channels
      let out := (List.range outFrames).foldl (init := #[]) fun acc i =>
        let srcPos := (i * buf.sampleRate * 1000) / dstRate
        let srcFrame := srcPos / 1000
        let frac : Int := (srcPos % 1000).toInt
        let nextFrame := min (srcFrame + 1) (inFrames - 1)
        (List.range channels).foldl (init := acc) fun acc2 c =>
          let a := samplesArr.get! (srcFrame * channels + c)
          let b := samplesArr.get! (nextFrame * channels + c)
          let v := a + (b - a) * frac / 1000
          acc2.push (clampI16 v)
      .ok { sampleRate := dstRate, channels := channels, samples := out.toList }

def normalizeToPeak (buf : AudioBuffer) (targetAmplitude : Int) : AudioBuffer :=
  if buf.samples.isEmpty then buf
  else
    let maxAbs := buf.samples.foldl (init := 0) fun acc s =>
      let a := s.natAbs
      if a > acc then a else acc
    if maxAbs == 0 then buf
    else
      let target : Int := Int.ofNat (min 32767 (max 0 targetAmplitude.toNat))
      let scaled := buf.samples.map fun s =>
        clampI16 (s * target / Int.ofNat maxAbs)
      { buf with samples := scaled }

def toMono (buf : AudioBuffer) : AudioBuffer :=
  if buf.channels == 1 then buf
  else
    let n := buf.frameCount
    let arr := buf.samples.toArray
    let out := (List.range n).foldl (init := #[]) fun acc i =>
      let l := arr.get! (i * 2)
      let r := arr.get! (i * 2 + 1)
      acc.push ((l + r) / 2)
    { buf with channels := 1, samples := out.toList }

def toStereo (buf : AudioBuffer) : AudioBuffer :=
  if buf.channels == 2 then buf
  else
    let out := buf.samples.foldl (init := #[]) fun acc s => acc.push s |>.push s
    { buf with channels := 2, samples := out.toList }

-- Byte serialization helpers
def byteLE (n : Nat) (nbytes : Nat) : List UInt8 :=
  (List.range nbytes).map fun i => ((n >>> (i * 8)) &&& 0xFF).toUInt8

def i16ToU16 (n : Int) : Nat :=
  if n < 0 then (n + 65536).toNat else n.toNat

def encodeWav (buf : AudioBuffer) : List UInt8 :=
  let bitsPerSample := 16
  let byteRate := buf.sampleRate * buf.channels * (bitsPerSample / 8)
  let blockAlign := buf.channels * (bitsPerSample / 8)
  let dataSize := buf.samples.length * 2
  let riffSize := 36 + dataSize
  let strBytes (s : String) : List UInt8 := s.toList.map (·.toNat.toUInt8)
  strBytes "RIFF" ++ byteLE riffSize 4
    ++ strBytes "WAVE"
    ++ strBytes "fmt " ++ byteLE 16 4
    ++ byteLE 1 2
    ++ byteLE buf.channels 2
    ++ byteLE buf.sampleRate 4
    ++ byteLE byteRate 4
    ++ byteLE blockAlign 2
    ++ byteLE bitsPerSample 2
    ++ strBytes "data" ++ byteLE dataSize 4
    ++ (buf.samples.flatMap (fun s => byteLE (i16ToU16 s) 2))

end Graphics.Audio
