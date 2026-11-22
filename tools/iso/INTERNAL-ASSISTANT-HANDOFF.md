# INTERNAL ASSISTANT HANDOFF — AureaOS ISO / Desktop
<!--
Questo file è un promemoria operativo interno per la continuità delle sessioni ChatGPT.
Non è documentazione utente né una specifica del prodotto.
Aggiornare questo file quando cambiano script, diagnosi o stato delle prove.
-->

Ultimo aggiornamento: 2026-11-22
Repository: AleessandroCeccarelli/AureaOs
Branch di lavoro: main

## PRIORITÀ E ISTRUZIONI DELL'UTENTE

- Da lunedì 23 novembre si lavora sul DESKTOP AureaOS in GitHub.
- NON fare build ISO, NON avviare verifiche ISO e NON modificare gli script ISO durante il lavoro desktop, salvo richiesta esplicita.
- Venerdì 27 novembre si riprende la verifica ISO su Fedora.
- L'utente usa Ventoy: copia direttamente `AureaOS.iso` nella partizione dati di Ventoy e la seleziona al boot. NON proporre di preparare/formattare una nuova chiavetta.
- Lavorare su `main`, senza creare branch salvo richiesta esplicita.
- L'utente esegue localmente su Fedora `cargo fmt`, `cargo check`, test e build. Non lanciare CI, build Rust o test remoti per conto suo.
- Regole architetturali: Wayland-only, Vulkan-only, Smithay; no X11/XWayland/OpenGL, no GNOME/KDE/XFCE come dipendenze del prodotto. Modularità rigorosa: un file = una responsabilità, file piccoli, evitare mega-file e modifiche strutturali non necessarie.
- Non chiedere all'utente di ripetere contesti già registrati qui. Leggere questo handoff e la cronologia Git prima di intervenire.

## STATO DEL DESKTOP

Progetto Rust con componenti principali:
- components/aurea-desktop-api
- components/aurea-compositor
- components/aurea-shell

Focus recente precedente alla build ISO: percorso input/puntatore e touchpad, integrazione tra eventi e shell, gestione display/monitor e fondamenta del desktop. Non deviare nuovamente sul touchpad se la priorità corrente è un'altra: seguire la richiesta dell'utente nella chat attiva. Non fare modifiche al desktop solo per preparare una ISO.

Ultime verifiche storiche riferite dall'utente:
- `cargo check -p aurea-compositor --features drm` terminato con successo dopo correzioni.
- In precedenza `cargo check --workspace` ha mostrato un errore di re-export privato per VulkanPendingUpload; non assumere che sia ancora presente senza controllare il codice attuale.
- Test Rust e build devono essere eseguiti dall'utente su Fedora.

## CRONOLOGIA RECENTE ISO (22 NOVEMBRE 2026)

Commit recenti su main:
- `47fd5ed7f5ee0830db37266c6d7858b140e8a57c` — chore: sync local repository changes
- `6bfc72c74d63741cc70ab16b220f2a7bdf91482c` — test(compositor): verify touchpad gestures reach desktop shell
- `ad524e94293697beb1f8aab4f3dde364ef9644b6` — style(compositor): tidy gesture integration tests
- `f38bd6ac03ccced2d3580d155161f97b1c8f55a3` — fix(iso): serialize access to persistent live-build workspace
- `63c4bed8f309e4fadb7857af99e5f52f710d53b5` — fix(iso): recover stale live-build lock on resume

Il lavoro ISO ha richiesto gran parte della giornata. Problemi affrontati:
- workspace live-build annidato erroneamente come `BUILD_ROOT/config/config`; migrazione prevista verso layout Debian standard `BUILD_ROOT/config` e `BUILD_ROOT/chroot`.
- problemi di mount/chroot, incluso `lb chroot_devpts`; il log finale indica che il problema è stato recuperato, non era l'errore terminale.
- problema precedente di `pam_systemd.so` mancante; hook attuale lo verifica esplicitamente in `/usr/lib/x86_64-linux-gnu/security/pam_systemd.so`.
- necessità di serializzare accesso al workspace con `flock` per evitare due live-build concorrenti.
- recupero del file lock live-build obsoleto durante `--resume`, con controlli di sicurezza su file/symlink.
- hook GRUB che rinomina la voce Debian Live in AureaOS.

Log della build caricato dall'utente il 22 novembre (file: Testo incollato(20261122-191410).txt):
- `live-build` termina con `P: Build completed successfully`.
- kernel e initramfs sono stati generati.
- hook `020-brand-boot-menu.hook.binary` ha personalizzato GRUB.
- `xorriso` ha generato la ISO.
- ISO: `/home/alessandro/Documenti/AureaOS.iso`
- sidecar: `/home/alessandro/Documenti/AureaOS.iso.sha256`
- SHA-256: `1d929c0f99b665f1307191d9502f5c094a25452e32dd28615309feb8350c0704`

Controlli Fedora eseguiti:
- `ls -lh`: ISO 350M, timestamp 22 nov 20:12.
- `sha256sum -c AureaOS.iso.sha256`: `AureaOS.iso: OK`.
- `file AureaOS.iso`: ISO 9660, DOS/MBR boot sector, bootable, volume `AUREAOS_BLUE_TEST`.
- Installato `xorriso` da Fedora con `sudo dnf install xorriso`.
- `xorriso -indev AureaOS.iso -report_el_torito as_mkisofs` rileva El Torito, MBR protective label, GPT, GRUB BIOS `/boot/grub/grub_eltorito` e EFI image `/boot/grub/efi.img`.
- Questi controlli provano che l'immagine è stata generata e contiene strutture di boot, NON che il sistema Live sia avviabile correttamente.

## RISULTATO DEL PRIMO BOOT REALE CON VENTOY

Hardware: portatile Lenovo, BIOS indicato nella foto come LENOVO 81D6/LNVNB161216, BIOS BUCN19WW 05/14/2021.

Ventoy ha avviato GRUB e il kernel Debian:
- kernel visualizzato: `6.12.117+deb13-amd64`
- il boot arriva al kernel, quindi non è un semplice problema di menu GRUB o di mancata lettura ISO.
- fallimento durante l'avvio init, prima del desktop.

Messaggi decisivi nella foto dell'utente:
```
init: line 249: can't open '/scripts/live': No such file or directory
Kernel panic - not syncing: Attempted to kill init: exitcode=0x00000200
```

Diagnosi certa: lo script `/scripts/live` non è presente/raggiungibile nel filesystem initramfs al momento in cui l'init lo invoca. Il kernel panic è conseguenza del fallimento di init. Non attribuire questo problema a GPU AMD, Vulkan o Smithay: non siamo ancora arrivati al lancio del compositor. La causa precisa resta da determinare (contenuto initramfs, hook initramfs-tools/live-boot, pacchetti, parametri kernel o combinazione di questi).

## SCRIPT ISO: STATO E ANALISI DEL CODICE SU MAIN AL 22 NOV

File letti direttamente da GitHub main e relativi blob SHA:
- `tools/iso/build-live-iso.sh` — blob `cdc5353fa4f353b6e8dbd2f30ff08f3df3022eae`
- `tools/iso/build-live-iso-podman.sh` — blob `eeef903db78844a0d8137704aeaedcc8da7bb649`
- `tools/iso/hooks/010-build-aureaos.hook.chroot` — blob `0124dd907b844a81591e69975ec2f0cd6bb98029`
- `tools/iso/hooks/020-brand-boot-menu.hook.binary` — blob `f3e1042b2c0d7ef4bb7b1458ca25bb238a3df6d1`
- `tools/iso/package-lists/aureaos.list.chroot` — blob `739af694db68019e51e6572dd21e2475154c93b7`

### build-live-iso.sh — comportamento attuale
- Debian Trixie, amd64, `--binary-image iso-hybrid`, bootloaders `grub-pc grub-efi`.
- `--bootappend-live` attuale: `boot=live components quiet loglevel=3 systemd.show_status=false live-config.nocomponents=user-setup live-config.nottyautologin`.
- Config standard in `BUILD_ROOT/config`.
- `refresh_inputs` copia package-list e hook in config e crea snapshot sorgenti tramite `git archive HEAD`.
- supporta `--resume`; migra vecchio workspace annidato `BUILD_ROOT/config/config` se la struttura è esattamente quella prevista.
- se resume e chroot esiste ma stage `.build/chroot_hooks` manca, esegue `sudo lb chroot_includes_after_packages --force`.
- alla fine esegue `sudo lb build`, cerca la ISO, la copia in output e calcola SHA-256.
- ATTENZIONE: il trap `cleanup` elimina con `sudo rm -rf BUILD_ROOT` l'intero workspace quando la build finisce con status 0. Quindi una build riuscita distrugge il workspace/cache e rende impossibile riutilizzarlo per una successiva build incrementale. Questo è un punto da riprogettare venerdì: preservare workspace e cache in modo sicuro, con pulizia esplicita/opzionale e non automatica dopo successo. Non modificare fino a venerdì.
- Il controllo di successo attuale verifica solo che esista un file ISO non vuoto e che `lb build` sia terminato: non verifica initramfs, `/scripts/live`, kernel, moduli, pacchetti runtime o boot entries.

### build-live-iso-podman.sh — comportamento attuale
- Usa `docker.io/library/debian:trixie-slim`.
- Richiede host `podman git realpath flock`.
- workspace persistente in `${BUILD_PARENT}/build`, lock `${BUILD_PARENT}/.aureaos-live-build.lock`.
- monta repo read-only, output e build-parent read/write.
- avvia container rootful con `sudo podman run --rm --privileged --security-opt label=disable`.
- ad ogni esecuzione nel container esegue `apt-get update` e installa `live-build git tar sudo ca-certificates xorriso`; questa fase può contribuire al tempo e va valutata per caching/immagine builder, senza introdurre complessità inutile.
- serializza il workspace con `flock -n`.
- `RESUME=1` abilita resume.
- output atteso `~/Documenti/AureaOS.iso` e sidecar sha.
- Non assumere che un workspace sia preservato dopo una build riuscita: lo script interno lo cancella col trap.

### 010-build-aureaos.hook.chroot — comportamento attuale
- verifica sorgenti Cargo e integrazioni sessione.
- verifica `pam_systemd.so`.
- scarica/installa Rust stable via rustup nel chroot; build `cargo +stable build --manifest-path ... --release --locked --package aurea-desktop --bin aurea-desktop`.
- installa binario, unit systemd, PAM service e script live session.
- crea utente aurea, maschera getty template, abilita aurea-desktop.service e imposta multi-user target.
- purge esplicito di curl, toolchain di build e header, senza autoremove.
- cancella sorgenti e cache Rust dal sistema immagine.
- Questo hook installa il desktop, ma non dimostra la corretta presenza degli script live nel initramfs.

### package-lists/aureaos.list.chroot — comportamento attuale
Include:
`live-boot`, `live-config`, `systemd-sysv`, `udev`, `dbus`, `linux-image-amd64`, `initramfs-tools`, firmware AMD, Vulkan/Mesa, DRM/GBM, libinput/libseat/libudev/xkbcommon/Wayland runtime e vari pacchetti di sviluppo/build.
- `live-boot` è dichiarato nella lista, ma il boot reale dimostra che la sola presenza del pacchetto nella lista non garantisce che `/scripts/live` sia incluso/usabile nell'initramfs finale.
- L'hook purge non rimuove esplicitamente live-boot/live-config; verificare comunque lo stato reale del chroot e del contenuto initramfs.

### 020-brand-boot-menu.hook.binary
- trova `grub.cfg` in `binary/boot/grub` o `boot/grub`.
- rinomina `Live system (amd64)` in `AureaOS`, imposta default 0 e timeout 5.
- la personalizzazione GRUB è riuscita nel log.

## PIANO TECNICO PER VENERDÌ 27 NOVEMBRE — NON ESEGUIRE PRIMA

### A. Ispezionare la ISO esistente prima di qualsiasi rebuild
L'utente deve avviare Fedora e mantenere ISO esistente. Non cancellare né ricompilare subito.
1. Montare ISO read-only:
   `sudo mkdir -p /mnt/aureaos-iso && sudo mount -o loop,ro ~/Documenti/AureaOS.iso /mnt/aureaos-iso`
2. Identificare kernel e initrd:
   `find /mnt/aureaos-iso -type f \( -iname '*initrd*' -o -iname '*vmlinuz*' \) -print`
3. Verificare contenuto filesystem ISO e GRUB:
   `find /mnt/aureaos-iso -maxdepth 5 -type f | sort`
   `grep -RniE 'scripts/live|boot=live|init=' /mnt/aureaos-iso/boot/grub 2>/dev/null`
4. Individuare strumenti disponibili su Fedora: `lsinitrd`, `lsinitramfs`, `unmkinitramfs`, `cpio`, `gzip`, `zstd`. Non supporre che siano già installati; installare solo lo strumento necessario.
5. Ispezionare l'initramfs della ISO e verificare se contiene `scripts/live`, `live-boot` e gli hook richiesti. Cercare anche file `conf/param.conf`, script initramfs-tools e moduli.
6. Smontare dopo l'ispezione: `sudo umount /mnt/aureaos-iso`.

### B. Se i dati della ISO non bastano
- Ispezionare il workspace live-build conservato (se esiste); non presumere che esista: il trap post-successo lo cancella.
- Se non esiste, non fare un'altra build senza aver prima capito il problema; riprodurre una sola volta con una modifica mirata e log completo, mantenendo il workspace.
- Nel chroot/build workspace verificare `dpkg-query -W live-boot live-config initramfs-tools`, contenuto di `/usr/share/initramfs-tools/hooks/`, `/usr/share/initramfs-tools/scripts/`, `/etc/initramfs-tools/conf.d/`, output `lsinitramfs` e log initramfs.
- Verificare che `live-boot` sia effettivamente installato nel chroot prima di generare initramfs e che il suo hook initramfs-tools venga eseguito.
- Verificare ordine dei hook e se la fase di installazione/purge modifica file o dipendenze prima/dopo la generazione initramfs.
- Verificare `boot=live` nei parametri kernel generati in GRUB e l'eventuale `init=` sovrascritto.
- Non concludere che la causa sia una singola cosa finché l'initramfs non è ispezionato.

### C. Correggere script e pipeline solo dopo la diagnosi
Requisiti per il prossimo script funzionante:
- preflight host (podman, git, flock, spazio libero, permessi, path e pacchetti host necessari).
- preflight repo (Cargo workspace, file ISO input, hook eseguibili, package list).
- preflight Debian chroot (pacchetti runtime richiesti realmente installati, hook initramfs di live-boot presente).
- verifiche initramfs PRIMA di produrre ISO: errore immediato se mancano `/scripts/live` o componenti live-boot.
- controllare il kernel e initrd selezionati e le voci GRUB BIOS/UEFI.
- testare che la voce GRUB contenga `boot=live` e parametri coerenti.
- post-build: ISO non vuota, SHA-256, ISO9660, El Torito BIOS/UEFI, contenuto atteso; salvare report di validazione.
- mantenere workspace/caches dopo successo per build incrementali; rimozione solo con flag esplicito tipo `--clean`, mai automatica.
- evitare di riscaricare/installare gli stessi strumenti builder ad ogni esecuzione: valutare immagine Podman versionata/cachata.
- lock robusto e recupero sicuro, senza cancellare lock attivo o seguire symlink.
- resume deterministico: aggiornare solo input necessari, rispettare stage marker live-build; non forzare fasi a caso.
- log completo persistente con timestamp e riepilogo di errore.
- nessuna build dichiarata riuscita se non supera tutti i controlli di contenuto e boot.
- mantenere piccoli script modulari, ciascuno con una responsabilità; coordinatore breve.

## COMANDI DI BUILD ATTUALI (SOLO RIFERIMENTO, NON ESEGUIRE FINO A VENERDÌ)

Build fresca (attenzione: oggi il cleanup cancella il workspace al successo):
`bash tools/iso/build-live-iso-podman.sh`

Resume di build fallita:
`RESUME=1 BUILD_PARENT=/path/to/previous-build bash tools/iso/build-live-iso-podman.sh`

Default:
- output: `$HOME/Documenti/AureaOS.iso`
- parent workspace: `${TMPDIR:-/var/tmp}/aureaos-podman-build`
- build directory container: `$BUILD_PARENT/build`
- volume repo: `/workspace:ro`
- volume output: `/output:rw`
- volume build: `/build-parent:rw`

## PROSSIMA AZIONE ALLA RIPRESA DELLA CHAT

1. Leggere questo file per primo.
2. Rispettare la priorità corrente: lavorare sul desktop fino a venerdì, nessun task ISO.
3. Venerdì iniziare dall'ispezione dell'ISO già generata e dell'initramfs, senza rebuild immediato.
4. Dopo diagnosi, progettare un refactor piccolo e verificabile dello script; non riscrivere tutto alla cieca.
5. Prima di ogni modifica ISO, riaprire i file attuali su main perché possono essere cambiati nel frattempo.


## DOCUMENTATION SYNC — 2026-11-22

The repository's public project records were updated after the first real ISO boot:
- `docs/it/DIARIO-DI-BORDO.md`: Italian chronological entry for 22 November.
- `docs/development/DEVELOPMENT-LOG.md`: matching English engineering entry.
- `ROADMAP.md` and `docs/it/ROADMAP.md`: current desktop-first priority and deferred ISO investigation.
- `README.md`: corrected statement of ISO validation status.

Operational priority remains unchanged: desktop work first from 23 November; no ISO work until Friday 27 November unless the user explicitly changes the plan. On Friday inspect the existing initramfs before rebuilding.
