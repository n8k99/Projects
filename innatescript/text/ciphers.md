# Vigenere / Vernam / Caesar Ciphers

> Vigenere / Vernam / Caesar Ciphers -- classical text encryption.

## Key Insights

Encryption is **reversible transformation with a secret** -- extending G017's reversible Pig Latin. Pig Latin's rule is public. Caesar's shift is a secret (but trivially breakable). Vigenere's key is a shared secret. Vernam's pad is a perfect secret. The spectrum from no-key to perfect-secrecy maps to InnateScript's trust model: how much do you trust the channel between agents?

The key is **shared state between encrypt and decrypt** -- both sides must have it. In InnateScript, this is the shared-secret pattern for agent communication. Two agents who share a key can communicate privately even through a public choreography. The key negotiation itself could be a choreography.

Caesar cipher's weakness (only 26 possible keys) demonstrates that **small key spaces are broken by enumeration** -- brute force. This connects to G013's credit card validation: structural checks catch simple errors, but deeper analysis catches fraud. The Luhn check is Caesar-level security -- it stops accidents, not attacks.

Vernam's requirement (key as long as the message, never reused) shows the **cost of perfect security**. In the noosphere, perfect privacy between agents requires a unique pad for every message. That's expensive. Agents typically settle for good-enough encryption (Vigenere level) because perfect security is impractical at scale.

**Choreographic case**: secure agent-to-agent communication channel. Kathryn sends financial data to Nathan encrypted with a shared key. The choreography negotiates the key via a side channel, then uses it for the main communication. This is Diffie-Hellman at the choreographic level.

## Domain Model

### Caesar Cipher

Caesar is `@map` over characters with a fixed shift -- the simplest form of keyed transformation. The key space is tiny (26 values), making it the "Hello World" of encryption.

```yaml
@define caesar-shift-char:
  @given: char, shift (integer)
  @derive:
    @match char:
      @when @alpha?:
        base: @if (@uppercase? char) 65 97
        offset: @mod (+ (- (@char-code char) base) shift) 26
        @return: @code-char (+ offset base)
      @otherwise:
        @return: char

@define caesar-encrypt:
  @given: text (string), shift (integer)
  @return: @map (@partial caesar-shift-char shift) (@chars text)

@define caesar-decrypt:
  @given: text (string), shift (integer)
  @return: caesar-encrypt text (- shift)
```

### Vigenere Cipher

Vigenere is Caesar with a **rotating key** -- each letter position uses a different shift derived from the keyword. The key index advances only on alphabetic characters, skipping punctuation and spaces. This is `@map-with-state`: the transformation depends on both the current element and accumulated state (the key index).

```yaml
@define vigenere-encrypt:
  @given: text (string), key (string)
  @precondition: @and (> (length key) 0) (@every? @alpha? key)
  @derive:
    key-upper: @uppercase key
    key-shifts: @map (lambda (c) (- (@char-code c) 65)) (@chars key-upper)
  @return:
    @map-with-state
      @initial-state: 0  # key index
      @step: (char, ki) =>
        @if (@alpha? char):
          shift: @nth key-shifts (@mod ki (length key-shifts))
          encrypted: caesar-shift-char char shift
          @emit: encrypted
          @next-state: (+ ki 1)
        @else:
          @emit: char
          @next-state: ki
      @over: (@chars text)

@define vigenere-decrypt:
  @given: text (string), key (string)
  @derive:
    inverted-key:
      @map (lambda (c) (@code-char (+ (@mod (- 26 (- (@char-code c) 65)) 26) 65)))
           (@chars (@uppercase key))
  @return: vigenere-encrypt text inverted-key
```

### Vernam Cipher (One-Time Pad)

Vernam is the **theoretically perfect** cipher: XOR with a key as long as the message, used exactly once. The key consumes itself -- each byte of key is used for exactly one byte of message, then discarded.

```yaml
@define vernam-encrypt:
  @given: data (bytes), key (bytes)
  @precondition: (>= (length key) (length data))
  @return: @zip-with @xor data key

@define vernam-decrypt:
  @given: data (bytes), key (bytes)
  # XOR is its own inverse -- encrypt and decrypt are the same operation
  @return: vernam-encrypt data key
```

## The Security Spectrum

These three ciphers form a spectrum from weakest to strongest:

```yaml
@define security-spectrum:
  caesar:
    key-space: 26
    breakable-by: "enumeration (brute force)"
    analogy: "a lock with 26 combinations"
    trust-level: "obscurity only"

  vigenere:
    key-space: "26^n where n = key length"
    breakable-by: "frequency analysis (Kasiski examination)"
    analogy: "a combination lock with multiple dials"
    trust-level: "shared secret"

  vernam:
    key-space: "256^n where n = message length"
    breakable-by: "nothing (information-theoretically secure)"
    analogy: "a unique lock for every message, destroyed after use"
    trust-level: "perfect secrecy"
    cost: "key distribution as expensive as message delivery"
```

This spectrum mirrors trust levels in agent communication. Agents on a public channel use Caesar-level obfuscation (enough to prevent casual eavesdropping). Agents with a shared secret use Vigenere-level encryption. Agents that need provable privacy use Vernam -- but pay the cost of key distribution.

## The @map-with-state Primitive

Vigenere introduces `@map-with-state` -- a transformation where each step depends on accumulated state. Caesar is pure `@map` (no state). Vigenere needs a key index that advances only on certain elements. This is a `fold` that also emits output -- sometimes called `mapAccum` or `scan-with-state`.

```yaml
# Pure map: each element transforms independently
@map caesar-shift-char text               # Caesar

# Map with state: transformation depends on position in key
@map-with-state step initial-state text    # Vigenere

# Zip: pair-wise combination of two sequences
@zip-with @xor data key                   # Vernam
```

These three patterns -- `@map`, `@map-with-state`, `@zip-with` -- cover most element-wise transformations. The resolver unifies them under a common abstraction: transformations over sequences with varying degrees of context.

## XOR as Self-Inverse

Vernam's elegance comes from XOR being its own inverse: `(a XOR k) XOR k = a`. The encrypt and decrypt functions are identical. This is the simplest form of a mathematical involution -- a function that is its own inverse.

```yaml
@property vernam: @involution
  # encrypt = decrypt
  # This is provable from XOR's algebraic properties
```

In InnateScript, any `@involution` automatically gets its inverse for free. The resolver knows that calling the function twice returns the original. This is the simplest possible `@inverse` -- the function *is* its own inverse.

## Choreography: Secure Agent-to-Agent Message

```innate
(define-choreography secure-message
  "Kathryn sends Nathan a message only he can read."
  (participants Kathryn Nathan)

  ;; Pre-shared key — established out-of-band
  (let ((shared-key "DRAGONPUNK"))

    ;; Kathryn encrypts
    (at Kathryn
      (let ((plaintext "Q3 projections show 40% growth in the noosphere sector")
            (ciphertext (vigenere-encrypt plaintext shared-key)))
        (send Nathan ciphertext)))

    ;; Nathan decrypts
    (at Nathan
      (let ((received (recv Kathryn))
            (plaintext (vigenere-decrypt received shared-key)))
        (display plaintext)))))
```

## Choreography: Vernam for Maximum Security

```innate
(define-choreography vernam-secure
  "When the secret justifies the cost of a full-length key."
  (participants Kathryn Nathan)

  (let ((message (encode "Launch code: ALPHA-7"))
        (otp-key (random-bytes (length message))))

    ;; Key distribution is the hard problem —
    ;; the key must travel a secure channel too.
    ;; The cost of the key equals the cost of the message.
    (at Kathryn
      (send Nathan (vernam-encrypt message otp-key)))

    (at Nathan
      (let ((plaintext (decode (vernam-decrypt (recv Kathryn) otp-key))))
        (display plaintext)))))
```

## Test Cases

```yaml
@verify caesar-encrypt:
  ("Hello, World!", 13) => "Uryyb, Jbeyq!"    # ROT13
  ("abc", 1)            => "bcd"               # simple shift
  ("xyz", 3)            => "abc"               # wrap around
  ("ABC", 0)            => "ABC"               # identity shift
  ("Hello 123!", 5)     => "Mjqqt 123!"        # non-alpha preserved

@verify caesar-roundtrip:
  @forall text shift:
    caesar-decrypt (caesar-encrypt text shift) shift == text

@verify vigenere-encrypt:
  ("Hello, World!", "SECRET") => "Zincs, Nfkpq!"
  ("A", "S")                  => "S"            # single char

@verify vigenere-roundtrip:
  @forall text key:
    @precondition: @and (> (length key) 0) (@every? @alpha? key)
    vigenere-decrypt (vigenere-encrypt text key) key == text

@verify vernam-roundtrip:
  @forall data key:
    @precondition: (>= (length key) (length data))
    vernam-decrypt (vernam-encrypt data key) key == data

@verify vernam-self-inverse:
  @forall data key:
    vernam-encrypt data key == vernam-decrypt data key
```

## Design Notes

- **Ciphers are the first InnateScript pattern where the KEY is more important than the
  ALGORITHM.** Caesar, Vigenere, and Vernam all use the same basic operation (modular
  addition or XOR), but the key's properties determine security.
- **The choreography controls who can decrypt** -- only agents with the key participate
  in the decryption step. This is access control expressed as choreographic structure.
- **Key distribution is the unsolved problem.** Vernam is theoretically perfect, but
  the key must be shared securely -- which requires a secure channel -- which is the
  problem encryption was supposed to solve. This circularity is fundamental.
