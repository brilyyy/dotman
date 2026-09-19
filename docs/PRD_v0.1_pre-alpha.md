
# 📋 Product Requirements Document (PRD): Dotfile Manager (`dotman`)

**Status:** Pre-Alpha v0.1

**Target Bahasa Pemrograman:** Rust

**Kolaborator:** Antigravity CLI

---

## 1. Tujuan & Ruang Lingkup (Objective & Scope)

Membangun aplikasi CLI berbasis bahasa **Rust** yang cepat, aman, dan transparan untuk memusatkan file/folder konfigurasi sistem (dotfiles) ke dalam satu repositori terpusat. Aplikasi ini dilengkapi dengan sistem mitigasi risiko file korup, manajemen folder secara rekursif, serta sistem manajemen dependensi aplikasi secara deklaratif.

---

## 2. Fitur Utama (Core Features - Pre-Alpha / v1)

### A. Inisialisasi & Manifest Pusat (`dot.toml`)

* **`dotman init`**: Menginisialisasi direktori saat ini sebagai repositori dotfile, membuat file manifest pusat (`dot.toml`), serta menyiapkan struktur folder karantina/backup `.bak/`.
* **Struktur Manifest Pusat (`dot.toml`)**:
```toml
[settings]
backup_enabled = true
backup_dir = ".bak"

[items]
# Manajemen File Tunggal
"zsh/zshrc" = { target = "~/.zshrc", type = "file" }

# Manajemen Folder Utuh (Contoh: Neovim)
"nvim" = { target = "~/.config/nvim", type = "folder", tags = ["dev"] }

[dependencies]
core = ["git", "curl", "zsh", "tmux"]
rust_tools = ["ripgrep", "bat", "eza", "bottom"]

```



### B. Penambahan File & Folder (`add <path>`)

* **Deteksi Otomatis:** CLI mendeteksi apakah path masukan berupa **file** atau **folder** (direktori).
* **Migrasi & Symlink:** Memindahkan file/folder dari sistem asli ke dalam folder repositori dotfile, lalu membuat *symlink* kembali ke lokasi asalnya.
* **Registrasi Otomatis:** Mendaftarkan jalur pemetaan baru secara otomatis ke dalam file `dot.toml`.

### C. Deployment Massal (`deploy`)

* **`dotman deploy`**: Membaca file `dot.toml` dan menautkan ulang seluruh file/folder ke sistem target (sangat berguna saat *fresh install*).
* **Tags / Profiling:** Mendukung argumen filter untuk deployment spesifik (misal: `dotman deploy --tag dev`).
* **Interactive Pre-flight Check:** Memeriksa ketersediaan program terkait. Jika program belum terpasang, CLI memberikan opsi interaktif (lewati atau lanjutkan).

### D. Manajemen Dependensi Sistem (`install-deps`)

* **`dotman install-deps`**: Membaca daftar paket aplikasi yang terdeklarasi di dalam manifest `dot.toml`.
* Menjalankan instalasi paket secara berurutan menggunakan package manager bawaan sistem operasi (seperti `pacman`/`paru` di Arch atau `apt`/`nala` di Debian/Ubuntu).

### E. Mitigasi Bencana & Anti-Corrupt (`Safety Net`)

* **Automatic Backup / Quarantine:** Sebelum *symlink* dibuat di atas file/folder sistem yang sudah ada, CLI wajib memindahkan file lama ke folder arsip (`.bak/timestamp_nama-item/`) terlebih dahulu guna mencegah kehilangan data.
* **Conflict Resolution:** Jika terjadi bentrok, CLI memunculkan prompt interaktif:
1. *Overwrite* (Backup & Timpa)
2. *Keep Existing* (Lewati)
3. *Diff* (Bandingkan perubahan)



### F. Pemeriksaan Kesehatan (`status`)

* **`dotman status`**: Memeriksa status seluruh *symlink*. Menandai apakah tautan tersebut berstatus **Valid**, **Broken** (terputus), atau **Conflict** (tertimpa file biasa).

---

## 3. Rencana Pengembangan Bersama Antigravity CLI

* **Fase 1 (Design & Parsing):** Membangun struktur proyek Cargo, parsing argumen CLI menggunakan `clap`, dan pembacaan file manifest `dot.toml` via `serde`.
* **Fase 2 (Symlink & Safety Engine):** Mengimplementasikan logika dasar pembuatan symlink (termasuk penanganan direktori rekursif) serta mekanisme backup otomatis ke folder `.bak/`.
* **Fase 3 (Dependency & Execution):** Mengembangkan perintah `install-deps` untuk memanggil eksekusi *package manager* secara asinkron/berurutan.
* **Fase 4 (Testing & Robustness):** Pengujian skenario konflik file, penanganan izin akses (*permissions*), dan validasi path ekstrem.
