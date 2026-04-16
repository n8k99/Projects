/- Vigenere / Vernam / Caesar Ciphers — classical text encryption. -/

namespace Text

/-- Shift a character by `n` positions within its case range. Non-alpha pass through. -/
def shiftChar (ch : Char) (n : Nat) : Char :=
  if ch.isUpper then
    Char.ofNat ((ch.toNat - 'A'.toNat + n) % 26 + 'A'.toNat)
  else if ch.isLower then
    Char.ofNat ((ch.toNat - 'a'.toNat + n) % 26 + 'a'.toNat)
  else
    ch

/-- Encrypt text using Caesar cipher with given shift. -/
def caesarEncrypt (text : String) (shift : Nat) : String :=
  let s := shift % 26
  String.mk (text.toList.map fun ch => shiftChar ch s)

/-- Decrypt Caesar cipher by shifting in the opposite direction. -/
def caesarDecrypt (text : String) (shift : Nat) : String :=
  let s := (26 - shift % 26) % 26
  caesarEncrypt text s

/-- Encrypt text using Vigenere cipher with given keyword. -/
def vigenereEncrypt (text : String) (key : String) : String :=
  let keyUpper := key.toUpper.toList
  let (result, _) := text.toList.foldl (fun (acc, ki) ch =>
    if ch.isAlpha then
      let shift := (keyUpper.get! (ki % keyUpper.length)).toNat - 'A'.toNat
      (acc ++ [shiftChar ch shift], ki + 1)
    else
      (acc ++ [ch], ki)
  ) ([], 0)
  String.mk result

/-- Decrypt Vigenere cipher by inverting each key letter's shift. -/
def vigenereDecrypt (text : String) (key : String) : String :=
  let inverted := String.mk (key.toUpper.toList.map fun ch =>
    Char.ofNat ((26 - (ch.toNat - 'A'.toNat)) % 26 + 'A'.toNat))
  vigenereEncrypt text inverted

/-- Encrypt bytes using Vernam (one-time pad) XOR cipher. -/
def vernamEncrypt (data : List UInt8) (key : List UInt8) : List UInt8 :=
  (data.zip key).map fun (d, k) => d ^^^ k

/-- Decrypt Vernam cipher (XOR is its own inverse). -/
def vernamDecrypt (data : List UInt8) (key : List UInt8) : List UInt8 :=
  vernamEncrypt data key

--- Roundtrip verification

#eval do
  -- Caesar roundtrip
  let plain := "Hello, World!"
  let enc := caesarEncrypt plain 13
  let dec := caesarDecrypt enc 13
  if dec != plain then IO.println s!"FAIL: Caesar roundtrip: got {dec}"
  else IO.println s!"Caesar(13): {plain} -> {enc} -> {dec}"

#eval do
  -- Vigenere roundtrip
  let plain := "Hello, World!"
  let key := "SECRET"
  let enc := vigenereEncrypt plain key
  let dec := vigenereDecrypt enc key
  if dec != plain then IO.println s!"FAIL: Vigenere roundtrip: got {dec}"
  else IO.println s!"Vigenere({key}): {plain} -> {enc} -> {dec}"

#eval do
  -- Vernam roundtrip
  let data : List UInt8 := [72, 101, 108, 108, 111]  -- "Hello"
  let key  : List UInt8 := [0x4a, 0x1f, 0x7c, 0x33, 0x0b]
  let enc := vernamEncrypt data key
  let dec := vernamDecrypt enc key
  if dec != data then IO.println s!"FAIL: Vernam roundtrip"
  else IO.println s!"Vernam: {data} -> {enc} -> {dec}"

end Text
