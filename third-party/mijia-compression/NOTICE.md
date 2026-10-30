# Modern Mijia compression dependencies

The ADR-0094 response decoder uses the pure Rust backend of `flate2`, with
versions pinned in `Cargo.lock`. No dependency source is modified. The retained
license files were copied from the exact locally cached registry packages;
[digests](licenses.json) record their source-declared license expressions.

| Package | Version | Retained license |
|---|---|---|
| `flate2` | `1.1.9` | [License](flate2-1.1.9-LICENSE-MIT.txt) |
| `miniz_oxide` | `0.8.9` | [License](miniz_oxide-0.8.9-LICENSE-MIT.txt) |
| `adler2` | `2.0.1` | [License](adler2-2.0.1-LICENSE-MIT.txt) |
| `adler2` | `2.0.1` | [License](adler2-2.0.1-LICENSE-0BSD.txt) |
| `crc32fast` | `1.5.0` | [License](crc32fast-1.5.0-LICENSE-MIT.txt) |
| `simd-adler32` | `0.3.10` | [License](simd-adler32-0.3.10-LICENSE.txt) |

Retain these notices when distributing affected source/binaries. This incremental
record does not rewrite the historical October 17 dependency audit or claim a
complete binary-distribution notice package.
