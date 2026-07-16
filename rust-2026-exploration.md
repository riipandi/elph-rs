# 🦀 Riset Perkembangan Rust 2026

> **Tanggal riset:** 16 Juli 2026
> **Sumber:** blog resmi Rust, GitHub Releases, ZDNET, TIOBE, dan lainnya.
> **Query pencarian:** "Rust programming language 2026 latest news" & "Rust 2026 new features"

---

## 📦 Rilis Stabil Terbaru

### Rust 1.97.0 — 9 Juli 2026

Rilis stabil terkini dengan beberapa fitur utama:

| Fitur | Deskripsi |
|---|---|
| **v0 Symbol Mangling sebagai default** | Skema mangling simbol baru (8 tahun dalam pembuatan sejak RFC 2603 tahun 2018) menghasilkan stack trace yang lebih mudah dibaca untuk kode generic. Warisan skema Itanium ABI hanya bisa diaktifkan lewat nightly. |
| **Cargo `build.warnings = "deny"`** | Konfigurasi bawaan Cargo untuk mengontrol perilaku warning (allow/warn/deny) tanpa menginvalidasi cache build. Tidak perlu lagi `RUSTFLAGS=-Dwarnings`. |
| **Output linker tidak lagi disembunyikan** | `rustc` kini menampilkan pesan linker secara default sebagai warning lint (`linker_messages`). Berguna untuk mendeteksi masalah linker yang sebelumnya terlewat. |
| **API baru untuk integer bit manipulation** | `isolate_highest_one`, `isolate_lowest_one`, `highest_one`, `lowest_one`, `bit_width` — baik untuk tipe integer biasa maupun `NonZero`. |

### Rust 1.96.0 — 28 Mei 2026

| Fitur | Deskripsi |
|---|---|
| **Tipe `Range*` baru di `core::range`** | Range baru (`Range`, `RangeFrom`, `RangeInclusive`) bersifat `Copy` karena mengimplementasikan `IntoIterator`, bukan `Iterator`. Memudahkan penyimpanan slice accessor dalam tipe `Copy`. |
| **Makro `assert_matches!` dan `debug_assert_matches!`** | Versi resmi dari makro populer `assert_matches!`, dengan output Debug yang lebih informatif saat panic. Tidak masuk prelude karena bisa bentrok dengan crate pihak ketiga. |
| **Perubahan target WebAssembly** | `--allow-undefined` tidak lagi dilewatkan ke linker; simbol tak terdefinisi sekarang menjadi linker error. Mencegah bug build-time. |
| **Dua CVE fixed di Cargo** | CVE-2026-5223 (medium — symlink pada tarball crate) dan CVE-2026-5222 (low — autentikasi URL ternormalisasi). Tidak memengaruhi pengguna crates.io. |

---

## 🗺️ Project Goals 2026 — Highlight Terpilih

Berdasarkan [Rust Project Goals 2026](https://rust-lang.github.io/rust-project-goals/2026/highlights.html), ada **71 goal** yang direncanakan tahun ini. Berikut yang paling menarik:

### 1. Cargo Script (Stabilisasi)
Single file Rust + dependensi, bisa dijalankan langsung dengan `cargo my_script.rs` atau `./my_script.rs` (via shebang). Sangat memudahkan scripting dan prototyping.

```rust
#!/usr/bin/env cargo
---
edition: 2024
[dependencies]
reqwest = { version = "0.12", features = ["blocking"] }
---

fn main() {
    let body = reqwest::blocking::get("https://httpbin.org/ip")
        .unwrap()
        .text()
        .unwrap();
    println!("My IP info: {body}");
}
```

### 2. Polonius Alpha — Borrow Checker Generasi Berikutnya
Menyelesaikan **"Problem Case #3"** yang gagal diatasi NLL (Non-Lexical Lifetimes) sejak 2018. Contoh kasus: control flow kondisional lintas fungsi yang sebelumnya ditolak borrow checker akan diterima.

```rust
fn get_default<'r, K: Hash + Eq + Copy, V: Default>(
    map: &'r mut HashMap<K, V>,
    key: K,
) -> &'r mut V {
    match map.get_mut(&key) {
        //                    ─────── 'r hanya perlu valid di sini
        Some(value) => value,
        //          ◄─────────┘
        None => {
            map.insert(key, V::default()); // ← sebelumnya ERROR
            map.get_mut(&key).unwrap()
        }
    }
}
```

### 3. Const Traits & Reflection
- **Const traits MVP**: `const fn` bisa memanggil trait method.
- **ADT const params**: struct, tuple, dan array bisa jadi parameter const generic.
- **Compile-time reflection**: eksperimen `#[compile_time_only]` untuk serialisasi tanpa derive macro.

```rust
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

pub fn process<const D: Dimensions>(data: &[f32]) {
    // ...
}

fn main() {
    process::<{ Dimensions { width: 1920, height: 1080 } }>(&data);
}
```

### 4. Ergonomic Ref-counting & Async Traits
- **Trait `Share`**: membedakan clone yang membuat alias (`Arc`/`Rc`) vs deep copy.
- **Ekspresi `move(...)` di closure**: kontrol presisi atas closure capture.
- **Async fn dynamic dispatch via `dyn Trait`**: panggil async fn lewat trait object (awalnya dengan boxed futures).

```rust
// Sebelum: variable temporary yang ambigu
let tx_clone = tx.clone();
tokio::spawn(async move {
    send_data(tx_clone).await;
});

// Sesudah: inline dan jelas dengan Share + move expressions
tokio::spawn(async {
    send_data(move(tx.share())).await;
});
```

### 5. Never Type (`!`), Try Trait, Extern Types
- **`!` (never type)** — setelah **10 tahun** tidak stabil, akhirnya menuju stabilisasi.
- **Trait `Try`** — kustomisasi operator `?` untuk tipe kustom selain `Result`/`Option`.
- **Sized trait hierarchy** — membuka jalan untuk extern types dan Arm SVE (scalable vectors).

### 6. Arbitrary Self Types & Field Projections
- Smart pointer kustom bisa jadi method receiver (seperti `Box`, `Arc`, `&`).
- Eksperimen field projections — mengakses field *melalui* smart pointer (NonNull, Pin, dll).
- Bagian dari roadmap **"Beyond the `&`"** dan kebutuhan **Rust for Linux**.

### 7. build-std
Cargo dapat membangun ulang standard library dari source. Sangat penting untuk:
- Target tier 3 tanpa pre-compiled std.
- Embedded development dengan optimasi ukuran.
- Flag codegen kustom.

### 8. Next-Generation Trait Solver
Ground-up rewrite dari trait solver (sejak 2022). Sudah dipakai untuk coherence checking sejak Rust 1.84. Stabilisasi tahun ini membuka jalan untuk:
- Perbaikan semua soundness bug tipe system (**Project Zero**).
- Implied bounds, cyclic trait matching.
- Fitur async (**Just add async** roadmap).

---

## 🔥 Tren & Berita Industri

### 🏆 Rust Masuk TIOBE Top 10 (Juli 2026)
Untuk **pertama kalinya**, Rust masuk dalam **10 besar indeks TIOBE** — metrik popularitas bahasa pemrograman. Ini menandakan adopsi yang semakin meluas di industri.

### 🐧 Linux Kernel Makin Serius dengan Rust
**Greg Kroah-Hartman** (maintainer kernel Linux stable) dalam wawancara dengan ZDNET (15 Juli 2026):

> *"Rust makes coding fun again"*

Menyatakan bahwa meskipun C tidak akan hilang dalam waktu dekat, masa depan kernel Linux akan semakin banyak menggunakan Rust. Ini selaras dengan roadmap **Rust for Linux** yang menjadi salah satu project goals 2026.

### 🩺 Clippy: Panggilan untuk Kontributor
Tim Clippy mengeluarkan *health report* (6 Juli 2026) yang mengakui adanya **masalah kapasitas review**. Mereka mengajak komunitas untuk berkontribusi menjaga kesehatan proyek Clippy.

### 🔧 Infrastruktur: GitHub Rulesets & Q3 Plans
Tim Infrastruktur Rust melaporkan Q2 2026 accomplishments (15 Juli 2026), termasuk implementasi **GitHub Rulesets** untuk meningkatkan keamanan repository.

### 📦 crates.io: Pembaruan Pengembangan
Pembaruan dari tim crates.io (13 Juli 2026) merangkum berbagai perbaikan dan fitur baru selama 6 bulan terakhir.

---

## 📋 Ringkasan Timeline 2026

| Tanggal | Kejadian |
|---|---|
| 3 Feb | **First look**: 2026 Project Goals diumumkan |
| 28 Mei | **Rust 1.96.0** — Range baru, assert_matches!, WASM changes |
| 30 Juni | Rust 1.96.1 — perbaikan Cargo timeout/retry, CVE patches |
| 6 Juli | Clippy health report — ajakan kontribusi |
| 9 Juli | **Rust 1.97.0** — v0 symbol mangling default, Cargo warnings control |
| 13 Juli | crates.io development update |
| 15 Juli | Rust masuk **TIOBE Top 10**; wawancara GKH tentang Rust di Linux |
| 15 Juli | Infrastructure Team Q2 Recap & Q3 Plan |
| 15 Juli | 1.97.1 pre-release testing dimulai |
| **16 Juli** | **🟢 1.97.1 dirilis** (hari ini) — fix miscompilation LLVM optimization |

---

## 🔮 Apa yang Bisa Diharapkan ke Depan?

1. **Stabilisasi Polonius Alpha** — borrow checker yang lebih fleksibel.
2. **Cargo Script stabil** — Rust sebagai scripting language yang layak.
3. **Never type (`!`)** stabil setelah satu dekade.
4. **Const traits + ADT const params** — const generics makin powerful.
5. **Next-gen trait solver** — membuka banyak fitur baru yang tertunda.
6. **Async fn in traits via dyn dispatch** — ergonomi async Rust meningkat drastis.
7. **build-std** — tooling untuk embedded dan custom target makin matang.

---

## 📚 Sumber Referensi

| Sumber | URL |
|---|---|
| Rust Blog — 1.97.0 | <https://blog.rust-lang.org/2026/07/09/Rust-1.97.0/> |
| Rust Blog — 1.96.0 | <https://blog.rust-lang.org/2026/05/28/Rust-1.96.0/> |
| Rust Project Goals 2026 | <https://rust-lang.github.io/rust-project-goals/2026/highlights.html> |
| Inside Rust Blog | <https://blog.rust-lang.org/inside-rust/> |
| ZDNET — GKH Interview | <https://www.zdnet.com/article/greg-kroah-hartman-linux-kernel-rust/> |
| TIOBE Index | <https://www.tiobe.com/tiobe-index/> |
| GitHub — Rust 1.97.0 Release | <https://github.com/rust-lang/rust/releases/tag/1.97.0> |
| GitHub — Rust 1.96.0 Release | <https://github.com/rust-lang/rust/releases/tag/1.96.0> |
| I Programmer — Rust TIOBE | <https://www.i-programmer.info/news/98-languages/19006-rust-enters-tiobe-top-10.html> |
| TechTimes — v0 Mangling | <https://www.techtimes.com/articles/320051/20260710/rust-197-lands-v0-symbol-mangling-default-ending-eight-year-migration.htm> |
