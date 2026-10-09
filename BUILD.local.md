# BUILD.local.md — fork berpatch dari `0xMH/sudo-mcp`

Repo ini adalah fork pribadi: **branch `main` = upstream `ac47cc8` + 1 commit patch lokal**.

| Remote | URL | Isi |
|---|---|---|
| `origin` | https://github.com/0xMH/sudo-mcp | upstream, dasar rebase |
| `mine` | https://github.com/inimuqsith/SudoMcp | fork publik (branch `main` = versi berpatch) |

Server MCP yang dijalankan: `~/.cargo/bin/sudo-mcp` (dari `cargo install --locked --path .`).
Helper dialog: `/usr/local/bin/ssh-askpass` (root:root 0755), sumbernya `ssh-askpass` di repo ini.

## Kenapa patch ini ada

| Perubahan | Alasan |
|---|---|
| `src/sudo.rs` → `PathBuf::from("/usr/bin/sudo")` | upstream mencari `sudo` lewat `$PATH`; `$PATH` milik user bisa ditulisi → vektor substitusi |
| `src/sudo.rs` → `cmd.arg("-A").arg("-k").arg("--")` | upstream tidak memakai `-k`: setelah satu approval, semua panggilan dalam `timestamp_timeout` (5 menit) jalan senyap sebagai root |
| `src/prompt.rs` → `Command::new("/usr/local/bin/ssh-askpass")` | upstream memilih helper dialog dengan `which()` atas `ssh-askpass`/`ksshaskpass`/`zenity`/`kdialog` di `$PATH`; diganti path absolut root-owned |
| `ssh-askpass` (file baru) | helper dialog (PyGObject: Gtk 4, fallback Gtk 3). Dipasang root-owned supaya tidak bisa diganti proses lain sebagai user yang sama |

Catatan penting: `/usr/libexec/gcr4-ssh-askpass` **bukan** helper standalone — itu klien
socket gcr (`GCR_SSH_ASKPASS_SOCKET`); dipanggil langsung selalu exit 2 dengan
`this program is not meant to be run directly`.

## Build & install

```bash
cd ~/Projects/SudoMcp

# 1. helper dialog (root-owned) — butuh root
install -m 0755 -o root -g root ssh-askpass /usr/local/bin/ssh-askpass

# 2. server MCP
cargo install --locked --path .        # → ~/.cargo/bin/sudo-mcp
```

Setelah rebuild, server MCP yang **sedang berjalan** masih memakai binary lama
(proses memegang inode lama). Jalankan `/mcp reload` di harness, lalu cek kesegaran
dengan membandingkan inode+mtime `/proc/<pid>/exe` terhadap binary di disk.

## Verifikasi

- Patch benar-benar ada di binary: literal `/usr/local/bin/ssh-askpass` (ada),
  `which_executable` dan `no askpass program found` (hilang), dan `-k` muncul
  **tepat 1×** bersebelahan dengan `-A` dan `--`.
  Catatan: string 13 byte seperti `/usr/bin/sudo` dirakit compiler dari dua immediate
  8-byte, jadi `grep -c '/usr/bin/sudo'` bisa 0 — cari `bin/sudo` + `/usr/bin`.
- Uji helper tanpa mengganggu: jalankan dengan stdout ke `/dev/null`, tunggu ~2 s —
  proses harus masih hidup dan punya fd `/dmabuf:` (GTK merender dialog).
  Jangan pernah mengalirkan stdout helper ke transkrip: stdout-nya adalah password.
- End-to-end: panggil tool `mcp__sudo_mcp_sudo_run` (`argv:["id"]`) → dialog muncul,
  hasil `uid=0(root)`. Panggilan kedua harus **juga** memunculkan dialog (bukti `-k`;
  panggilan instan <1 s berarti cache timestamp yang dipakai).

## Patch yang diexport

`patches/*.patch` di repo ini — hasil:

```bash
git format-patch -o patches origin/main..main
```

## Menyusul rilis upstream berikutnya

```bash
git fetch origin
git rebase origin/main              # sedang di branch main
cargo install --locked --path .     # lalu `/mcp reload`
git format-patch -o patches origin/main..main
git add patches && git commit -m "patches: export ulang setelah rebase"
git push mine main
```
