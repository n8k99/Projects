"""MP3-to-Wav Converter — audio buffer + resample + normalize + RIFF encoding.

AudioBuffer: interleaved PCM int16 + sample_rate + channels.
Resample: linear interpolation, integer-only math for determinism.
Normalize: two-pass scan max_abs, then scale to target peak.
Channel mix: stereo/mono via average / duplicate.
RIFF encoding: canonical 44-byte WAV header + little-endian samples.
"""

from __future__ import annotations
import struct
from dataclasses import dataclass, field
from enum import Enum


@dataclass
class AudioBuffer:
    sample_rate: int
    channels: int
    samples: list[int] = field(default_factory=list)

    def frame_count(self) -> int:
        if self.channels == 0: return 0
        return len(self.samples) // self.channels

    def duration_ms(self) -> int:
        if self.sample_rate == 0: return 0
        return self.frame_count() * 1000 // self.sample_rate


@dataclass
class Mp3Frame:
    sample_rate: int
    channels: int
    bitrate_kbps: int
    samples_per_frame: int
    pcm: list[int]


class CodecError(Exception):
    pass


class NoFrames(CodecError):
    pass


class MixedFormats(CodecError):
    pass


class EmptyBuffer(CodecError):
    pass


class UnsupportedChannels(CodecError):
    def __init__(self, n: int):
        super().__init__(f"unsupported channels: {n}")
        self.n = n


class SampleRateZero(CodecError):
    pass


def decode_mp3_to_buffer(frames: list[Mp3Frame]) -> AudioBuffer:
    if not frames:
        raise NoFrames()
    sample_rate = frames[0].sample_rate
    channels = frames[0].channels
    if sample_rate == 0:
        raise SampleRateZero()
    if channels not in (1, 2):
        raise UnsupportedChannels(channels)
    for f in frames:
        if f.sample_rate != sample_rate or f.channels != channels:
            raise MixedFormats()
    samples: list[int] = []
    for f in frames:
        samples.extend(f.pcm)
    return AudioBuffer(sample_rate=sample_rate, channels=channels, samples=samples)


def _clamp_i16(v: int) -> int:
    if v > 32767: return 32767
    if v < -32768: return -32768
    return v


def resample(buf: AudioBuffer, dst_rate: int) -> AudioBuffer:
    if buf.sample_rate == 0 or dst_rate == 0:
        raise SampleRateZero()
    if dst_rate == buf.sample_rate:
        return AudioBuffer(buf.sample_rate, buf.channels, list(buf.samples))
    in_frames = buf.frame_count()
    if in_frames == 0:
        return AudioBuffer(dst_rate, buf.channels, [])
    out_frames = (in_frames * dst_rate + buf.sample_rate // 2) // buf.sample_rate
    channels = buf.channels
    out: list[int] = []
    for i in range(out_frames):
        src_pos = i * buf.sample_rate * 1000 // dst_rate
        src_frame = src_pos // 1000
        frac = src_pos % 1000
        next_frame = min(src_frame + 1, in_frames - 1)
        for c in range(channels):
            a = buf.samples[src_frame * channels + c]
            b = buf.samples[next_frame * channels + c]
            v = a + (b - a) * frac // 1000
            out.append(_clamp_i16(v))
    return AudioBuffer(dst_rate, channels, out)


def normalize_to_peak(buf: AudioBuffer, target_amplitude: int) -> AudioBuffer:
    if not buf.samples:
        return AudioBuffer(buf.sample_rate, buf.channels, list(buf.samples))
    max_abs = 0
    for s in buf.samples:
        a = abs(s)
        if a > max_abs:
            max_abs = a
    if max_abs == 0:
        return AudioBuffer(buf.sample_rate, buf.channels, list(buf.samples))
    target = max(0, min(32767, target_amplitude))
    scaled = [_clamp_i16(s * target // max_abs) for s in buf.samples]
    return AudioBuffer(buf.sample_rate, buf.channels, scaled)


def to_mono(buf: AudioBuffer) -> AudioBuffer:
    if buf.channels == 1:
        return AudioBuffer(buf.sample_rate, 1, list(buf.samples))
    n = buf.frame_count()
    out = []
    for i in range(n):
        l = buf.samples[i * 2]
        r = buf.samples[i * 2 + 1]
        out.append((l + r) // 2)
    return AudioBuffer(buf.sample_rate, 1, out)


def to_stereo(buf: AudioBuffer) -> AudioBuffer:
    if buf.channels == 2:
        return AudioBuffer(buf.sample_rate, 2, list(buf.samples))
    out = []
    for s in buf.samples:
        out.append(s)
        out.append(s)
    return AudioBuffer(buf.sample_rate, 2, out)


def encode_wav(buf: AudioBuffer) -> bytes:
    bits_per_sample = 16
    byte_rate = buf.sample_rate * buf.channels * (bits_per_sample // 8)
    block_align = buf.channels * (bits_per_sample // 8)
    data_size = len(buf.samples) * 2
    riff_size = 36 + data_size
    header = struct.pack(
        "<4sI4s4sIHHIIHH4sI",
        b"RIFF", riff_size,
        b"WAVE",
        b"fmt ", 16, 1, buf.channels, buf.sample_rate,
        byte_rate, block_align, bits_per_sample,
        b"data", data_size,
    )
    body = struct.pack(f"<{len(buf.samples)}h", *buf.samples) if buf.samples else b""
    return header + body


def decode_wav(data: bytes) -> AudioBuffer:
    if len(data) < 44:
        raise EmptyBuffer()
    if data[0:4] != b"RIFF" or data[8:12] != b"WAVE":
        raise EmptyBuffer()
    channels = struct.unpack("<H", data[22:24])[0]
    sample_rate = struct.unpack("<I", data[24:28])[0]
    data_size = struct.unpack("<I", data[40:44])[0]
    samples_bytes = data[44:44 + data_size]
    sample_count = len(samples_bytes) // 2
    samples = list(struct.unpack(f"<{sample_count}h", samples_bytes))
    return AudioBuffer(sample_rate, channels, samples)


def demo() -> None:
    frames = [
        Mp3Frame(22050, 2, 128, 4, [100, -100, 200, -200, 400, -400, 800, -800]),
        Mp3Frame(22050, 2, 128, 2, [-500, 500, -1000, 1000]),
    ]
    buf = decode_mp3_to_buffer(frames)
    print(f"decoded: rate={buf.sample_rate} ch={buf.channels} samples={len(buf.samples)}")

    resampled = resample(buf, 44100)
    print(f"resampled: rate={resampled.sample_rate} ch={resampled.channels} samples={len(resampled.samples)}")

    normalized = normalize_to_peak(resampled, 32000)
    max_abs = max(abs(s) for s in normalized.samples)
    print(f"normalized max_abs: {max_abs}")

    mono = to_mono(normalized)
    print(f"mono: ch={mono.channels} samples={len(mono.samples)}")

    wav = encode_wav(mono)
    print(f"wav bytes: {len(wav)}")
    print(f"header: {wav[0:4]!r} {wav[8:12]!r}")

    decoded = decode_wav(wav)
    print(f"decoded wav: rate={decoded.sample_rate} ch={decoded.channels} samples={len(decoded.samples)}")
    print(f"samples match: {decoded.samples == mono.samples}")


if __name__ == "__main__":
    demo()
