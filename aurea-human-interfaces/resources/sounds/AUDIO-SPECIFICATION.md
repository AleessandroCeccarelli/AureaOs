# AureaOS Sound Theme — Audio Specification

Initial synthesis parameters for the nine original AureaOS system sound effects. These values are starting points for listening tests and may be refined before approval.

## General parameters

* Format: Ogg Vorbis in .oga container
* Sample rate: 48 kHz
* Channels: Stereo, centered signal (no hard panning)
* Peak level: Target maximum peak of -3 dBFS
* Waveform: Smooth sine-based tones with gentle envelopes
* Silence: Trim unnecessary leading and trailing silence
* Transitions: Avoid clicks by applying short attack and release envelopes

## Event parameters

File	Duration	Frequency / notes	Character
message-new-instant.oga	350–500 ms	660 Hz → 880 Hz	Two soft, clear notes
dialog-error.oga	450–650 ms	220 Hz + 277 Hz	Low, restrained, slightly dissonant
dialog-warning.oga	400–600 ms	660 Hz → 740 Hz	Clear alert with a short tail
device-added.oga	250–400 ms	523 Hz → 784 Hz	Brief ascending tone
device-removed.oga	250–400 ms	784 Hz → 523 Hz	Brief descending tone
battery-low.oga	500–700 ms	440 Hz, two pulses	Gentle repeated warning
power-plug.oga	250–400 ms	523 Hz → 659 Hz	Brief, positive
power-unplug.oga	250–400 ms	659 Hz → 523 Hz	Brief, neutral
complete.oga	600–900 ms	523 Hz → 659 Hz → 784 Hz	Short, harmonious sequence

## Synthesis guidelines

* Use short attack and release ramps (approximately 5–15 ms) to prevent clicks.
* Use a brief gap between separate notes or pulses (approximately 40–80 ms).
* Keep the perceived loudness consistent; do not normalize solely by peak amplitude.
* Use gentle note transitions and avoid harsh high-frequency content.
* Export and audition every sound before marking it approved in SOUND-TESTING.md.