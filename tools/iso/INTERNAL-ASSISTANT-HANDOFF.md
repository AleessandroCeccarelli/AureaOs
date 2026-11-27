# INTERNAL ASSISTANT HANDOFF — AureaOS ISO / Desktop
<!--
Questo file è un promemoria operativo interno per la continuità delle sessioni ChatGPT.
Non è documentazione utente né una specifica del prodotto.
Aggiornare questo file quando cambiano script, diagnosi o stato delle prove.
-->

Ultimo aggiornamento: 2026-11-27
Repository: AleessandroCeccarelli/AureaOs
Branch di lavoro: main

## PRIORITÀ E ISTRUZIONI DELL'UTENTE

- Da lunedì 23 novembre si lavora sul DESKTOP AureaOS in GitHub.
- L'utente ha richiesto esplicitamente la migrazione della ISO Factory a systemd-boot + UKI e l'avvio del desktop reale in modalità Live. Le modifiche ISO sono autorizzate su main.
- La ISO serve soltanto come ambiente Live di sviluppo/test. Installer, partizionamento, installazione Btrfs, Aurea Update e Aurea Recovery sono rinviati.
- L'utente chiede di non disperdere tempo su manutenzione ripetitiva di boot Debian già stabile: verificare la catena base come prerequisito e concentrare i controlli della factory sul fatto che produca un Debian Live coerente, senza attribuire a Debian i problemi sopra il punto di consegna AureaOS.
- Non generare ISO né eseguire build/test Rust remoti in questa sessione. L'utente eseguirà la verifica raggruppata su Fedora.
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
- precedente hook GRUB che rinominava la voce Debian Live in AureaOS (rimosso il 26 novembre).

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
- Lo script attuale preserva workspace e cache sia in caso di successo sia di errore; la vecchia nota sul trap che cancellava il workspace è storica e non descrive più `main`.
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
- Il workspace live-build viene preservato anche dopo il successo; l'assemblaggio UKI ora usa una copia isolata e non consuma i file kernel/initramfs necessari per riprendere una build.

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


## REGOLA PERMANENTE DI CONTINUITÀ — LETTURA OBBLIGATORIA

Questo file è il riferimento operativo principale per l'assistente ChatGPT che lavora su AureaOS. Prima di QUALSIASI attività sul repository, in ogni nuova chat o ripresa di sessione:
1. Aprire e leggere integralmente questo file dalla branch `main`.
2. Verificare il commit HEAD e i file effettivi interessati direttamente su GitHub; questo handoff non sostituisce l'ispezione del codice corrente.
3. Confrontare le istruzioni della chat attiva con le priorità e i vincoli qui registrati. L'istruzione più recente dell'utente prevale se cambia esplicitamente una decisione precedente.
4. Non chiedere all'utente di ripetere informazioni già registrate.
5. Prima di concludere una sessione, aggiornare questo file con decisioni, progressi effettivi, file/commit toccati, verifiche realmente eseguite, problemi aperti e prossima azione precisa.
6. Non dichiarare completato, funzionante o testato ciò che è soltanto progettato, dedotto o non verificato.
7. Aggiornare anche il diario pubblico quando si è prodotto un progresso tecnico o una decisione progettuale significativa; evitare duplicazioni inutili.

L'assistente è responsabile della manutenzione di questo handoff. Non è un promemoria facoltativo e non va sostituito con una ricostruzione basata solo sulla memoria della chat.

## STATO CORRENTE — 2026-11-23 — RIPARTENZA DAL DESKTOP

### Stato repository verificato

- Repository: `AleessandroCeccarelli/AureaOs`.
- Branch di lavoro: `main`.
- HEAD osservato durante la sessione: `798d181e0228940686cedeb9718fc4dbcd2d39b7`.
- Commit HEAD: aggiornamento relativo allo sfondo predefinito provvisorio (verificato dalla cronologia Git nella sessione).
- Nessuna modifica al codice è stata eseguita in questa sessione; è stata svolta un'ispezione documentale e di alcuni sorgenti.
- Non sono stati eseguiti `cargo fmt`, `cargo check`, test, build Rust o build ISO.

### Priorità utente ribadita

La priorità immediata NON è rendere funzionante il puntatore. Prima bisogna costruire una base desktop reale e visibile, sulla quale in seguito integrare puntatore e interazioni.

Per la fase corrente:
- costruire il desktop di AureaOS partendo dalla superficie di lavoro di base;
- Dock escluso temporaneamente;
- Launcher escluso temporaneamente;
- puntatore e relative funzionalità esclusi temporaneamente come obiettivo di implementazione;
- nessuna build ISO fino alla data già concordata, salvo nuova richiesta esplicita;
- lavorare su `main`;
- nessuna build/test remoto: l'utente esegue i controlli Rust su Fedora quando concordato.

### Ispezione sorgenti effettuata il 23 novembre

File letti su `main`:

- `components/aurea-compositor/src/render.rs` — blob `5f7fb6a7eee0a212f33afd238f12152f2476b559`.
- `components/aurea-compositor/src/renderer.rs` — blob `016ccae1392e6dd4a59c9bf8bead7cb688651ea7`.
- `components/aurea-compositor/src/compositor.rs` — blob `f3a77804393414df78d6919f5df48b68bf8940d2`.
- `components/aurea-shell/src/desktop/dock.rs` — blob `6731b3d898fb27b9cbb9d510830757a1bf0027a9`.
- `aurea-human-interfaces/DESKTOP-EXPERIENCE-SPEC.md` — specifica di prodotto letta integralmente.

Risultati circoscritti e verificati:
- `RenderScene` in `render.rs` raccoglie attualmente toplevel mappati e cursore; il metodo `render_elements` produce elementi Wayland delle finestre. Nel contenuto ispezionato non è presente un elemento dedicato alla superficie wallpaper/desktop.
- `RenderTarget` in `renderer.rs` rappresenta output abilitati, modalità, scala, trasformazione e origine logica; è quindi una base esistente per il rendering per-monitor e coordinate locali.
- `RenderBackend::render_frame` definisce l'interfaccia di rendering; il file è backend-independent.
- `AureaCompositor` coordina già output manager, window manager, frame scheduler e percorso rendering, oltre ai servizi di input. Non aggiungere responsabilità desktop indiscriminatamente a questo file.
- `dock.rs` contiene lo stato/policy della Dock; la Dock esiste come modulo ma è fuori ambito nella fase corrente.
- La specifica `DESKTOP-EXPERIENCE-SPEC.md` descrive l'esperienza desktop completa (wallpaper, Dock, launcher, workspace, finestre, notifiche, quick settings), ma dichiara esplicitamente di essere una specifica e non una prova di implementazione.
- La presenza di output manager e render target NON prova che un desktop di sfondo sia già visualizzato.
- Non è ancora stata verificata in questa sessione l'intera implementazione del backend Vulkan, il caricamento runtime del wallpaper, la persistenza della preferenza wallpaper o la composizione effettiva di una superficie desktop vuota. Non affermare che questi elementi funzionino finché non ispezionati e testati.

### Direzione architetturale concordata per il desktop di base

Obiettivo iniziale: all'avvio della sessione, visualizzare una superficie desktop vuota e stabile anche quando non esistono finestre applicative, senza Dock, Launcher o puntatore come obiettivi della prima fase.

Separazioni di responsabilità da rispettare:
- stato/preferenze desktop nella Shell o API appropriata;
- rappresentazione della superficie desktop e wallpaper nel percorso di composizione/rendering;
- gestione output e coordinate per-monitor attraverso le strutture già esistenti;
- rendering e presentazione tramite backend Vulkan esistente;
- niente duplicazione di output manager, renderer o gestione finestre;
- evitare mega-file: ogni nuova responsabilità in modulo piccolo, con test mirati.

Non creare subito nuovi file o API prima di aver completato l'audit dei percorsi già presenti. In particolare, prima di implementare:
1. leggere il modulo rendering del compositor e il backend Vulkan effettivo;
2. leggere il wiring della Shell/sessione desktop e l'avvio del runtime;
3. individuare dove viene costruita la scena per ogni output e dove avviene la presentazione;
4. verificare come sono gestiti frame vuoti, clear color, formati texture e caricamento asset;
5. definire il punto minimo di integrazione della desktop surface senza introdurre dipendenze circolari.

### Sequenza di lavoro approvata

1. Audit del percorso di rendering attuale, dal frame scheduler al backend Vulkan e presentazione output.
2. Desktop surface minima: background/clear surface visibile anche senza finestre.
3. Wallpaper: caricamento, fallback robusto, dimensionamento/crop e gestione per output.
4. Collegamento dello stato tema chiaro/scuro alla palette e alla composizione effettiva.
5. Verifica geometria e comportamento su più monitor, riutilizzando OutputManager e RenderTarget.
6. Solo dopo che il desktop base è visibile e stabile: riprendere puntatore e interazioni.
7. Dock e Launcher restano esplicitamente rinviati.

### Regole per lo stato e i risultati

- Stato attuale: architettura di rendering e gestione output esistenti; superficie wallpaper dedicata non individuata nel file RenderScene ispezionato; implementazione completa del desktop base NON ancora verificata.
- Nessun test o build è stato eseguito in questa sessione.
- Non dichiarare il desktop funzionante finché non esiste una prova su Fedora/hardware o un test di rendering appropriato.
- Alla prossima ripresa: leggere prima questo handoff, verificare HEAD, poi continuare l'audit dei file rendering Vulkan e wiring Shell/runtime. Non ripartire dal puntatore.


## DECISIONE ARCHITETTURALE — DESKTOP MODULARE E DESKTOP MANAGER (2026-11-23)

L'utente richiede una base desktop costruita in modo esplicito e modulare: cartelle per responsabilità, file piccoli e un manager centrale. Il lavoro deve iniziare dal desktop vuoto, senza Dock, Launcher o puntatore come obiettivi della fase.

### Evidenze dal codice attuale (ricontrollate su main dopo il commit 6b5690b)

- `components/aurea-compositor/src/vulkan/surface_compositor.rs` contiene già il percorso di composizione SHM Vulkan. Per una scena senza superfici visibili e senza cursore chiama funzioni di clear/release o clear/readback.
- `components/aurea-compositor/src/vulkan/surface_compositor/readback.rs` e `release.rs` usano il colore `BACKGROUND_COLOR` definito in `surface_compositor.rs`, attualmente blu fisso `[0.08, 0.18, 0.32, 1.0]`. Quindi esiste già un fallback grafico uniforme per il frame vuoto; non equivale ancora a un wallpaper o a un desktop configurabile.
- `components/aurea-compositor/src/vulkan/clear_pass.rs` esegue il clear dell'immagine Vulkan, ma non possiede policy desktop.
- `components/aurea-compositor/src/compositor/rendering.rs` costruisce `RenderScene` per ogni output e la passa al backend.
- `components/aurea-compositor/src/render.rs` contiene la scena con finestre e cursore, ma non una rappresentazione esplicita della superficie desktop.
- `RenderTarget` riutilizza già dimensioni fisiche, scala, trasformazione e origine logica dell'output; non creare un secondo output manager.
- `components/aurea-shell/src/desktop/mod.rs` espone già `DesktopState` con Dock, Launcher, workspaces, pannelli, tema e Night Light. È stato progettato per stato/presentazione Shell, non per possedere risorse Vulkan.
- I token canonici per la superficie desktop si trovano in `aurea-human-interfaces/design-system/tokens/index.css`: light `#edf4ff`, dark `#0d1320`.

### Confini di responsabilità decisi per evitare duplicazioni

- **DesktopManager (compositor):** unico proprietario della policy della superficie desktop renderizzata: tema effettivo del fondo, sorgente del background, fallback e snapshot immutabile da consegnare alla scena di rendering. Non gestisce finestre, input, output discovery, Dock o Launcher.
- **Desktop Surface:** rappresentazione logica della superficie che sta sotto tutte le finestre e copre l'intera area logica del desktop; il primo backend usa un colore opaco, successivamente una texture wallpaper.
- **Background:** scelta e risoluzione della sorgente (inizialmente tema/colore; in seguito wallpaper). Il caricamento/decodifica delle immagini non va inserito nel manager.
- **Theme:** valori semantici coerenti con i token AureaOS; non duplicare i token nelle UI. Per il primo clear Vulkan usare i valori desktop light/dark ricavati dai token.
- **RenderScene:** contiene uno snapshot del desktop per il frame. La scena è costruita una volta per frame e riutilizzata per ogni output, con geometria locale calcolata dal RenderTarget esistente.
- **Vulkan surface compositor:** consuma lo snapshot e disegna prima il desktop, poi superfici applicative e infine il cursore. Il clear pass resta un'operazione Vulkan generica, senza conoscere il tema.
- **Shell DesktopState:** continua a possedere stato di launcher/dock/workspace/pannelli e preferenze utente. Non deve possedere immagini Vulkan né duplicare il DesktopManager del compositor. Il ponte tra preferenze Shell e policy compositor sarà definito tramite aurea-desktop-api dopo la prima superficie funzionante.

### Struttura di cartelle prevista (da implementare gradualmente)

```text
components/aurea-compositor/src/desktop/
  mod.rs
  manager.rs          # coordina lo stato desktop renderizzato
  surface.rs          # descrizione della superficie desktop
  background.rs       # sorgente e fallback dello sfondo
  theme.rs            # risoluzione dei token tema per il compositor
  snapshot.rs         # dati immutabili per un frame/output

components/aurea-compositor/src/render/
  ...                 # RenderScene riceve lo snapshot desktop

components/aurea-compositor/src/vulkan/
  surface_compositor/
    ...               # consuma lo snapshot: desktop, finestre, cursore

components/aurea-shell/src/desktop/
  mod.rs
  ...                 # stato e presentazione Shell già esistenti
  manager.rs          # valutare solo quando serve un coordinatore Shell distinto
```

Non creare tutti questi file in anticipo. Aggiungere un file solo quando ha una responsabilità reale e un chiamante reale. Evitare di creare un secondo `DesktopManager` nella Shell finché non c'è una necessità dimostrata.

### Sequenza implementativa approvata

1. Completare il tracciamento dei punti di costruzione scena e delle due vie Vulkan (DMA-BUF/KMS e CPU readback), verificando che entrambe consumino la stessa descrizione desktop.
2. Introdurre il modello desktop minimo e il DesktopManager nel compositor, con fallback opaco e tema chiaro predefinito. Non alterare il clear pass generico.
3. Aggiungere lo snapshot desktop alla RenderScene e disegnare la superficie prima delle finestre in entrambe le vie di presentazione.
4. Aggiungere test puri per tema, fallback, validazione colore e snapshot; l'utente esegue cargo fmt/check/test su Fedora.
5. Solo dopo il primo desktop vuoto stabile: introdurre wallpaper (decodifica separata, cache/upload Vulkan separati, policy di scaling separata).
6. Poi integrare preferenze Shell, tema effettivo e gestione multi-monitor. Riutilizzare RenderTarget e OutputManager.
7. Puntatore, Dock e Launcher rimangono rinviati fino a desktop base verificato.

### Stato implementazione dopo questa decisione

- La struttura `components/aurea-compositor/src/desktop/` NON è ancora stata creata.
- Nessun file Rust è stato modificato in questa sessione.
- Non è stato eseguito alcun test/build Rust.
- La modifica al colore fisso non va fatta isolatamente: prima va passato un modello/snapshot desktop comune attraverso RenderScene e entrambe le vie Vulkan, così non si ottengono comportamenti diversi tra presentazione DMA-BUF e CPU readback.
- HEAD verificato: `6b5690bf7551bc0aaf2971b89aba7973605cb34f`.
- Prossima azione: ispezionare la seconda metà del percorso Vulkan di presentazione e verificare ogni chiamata alle funzioni clear/readback/release; poi implementare il primo vertical slice piccolo e coerente.


## IMPLEMENTAZIONE DESKTOP — 2026-11-23 (PRIMO VERTICAL SLICE)

### Commit effettivi su main

- `542f9182d5e36975d1f5b0eefbb1ab8c0cd67041` — `feat(desktop): add desktop manager and themed Vulkan base surface`
- `2826f2d588e18138dcd433e1b8b0e9a035f13e63` — `fix(desktop): pass selected surface color through Vulkan clear paths`
- `e0fb92e214aaafe5957e04eb8e4b0e3d1764917b` — `fix(desktop): pass selected surface color through Vulkan release paths`

HEAD verificato dopo gli interventi: `e0fb92e214aaafe5957e04eb8e4b0e3d1764917b`.

### Codice creato/modificato

Nuova cartella `components/aurea-compositor/src/desktop/`:
- `mod.rs`: esportazioni del sottosistema desktop.
- `manager.rs`: DesktopManager, unico proprietario della policy di tema/background e produttore dello snapshot frame.
- `theme.rs`: temi Light/Dark e colori desktop coerenti con i token AureaOS.
- `background.rs`: sorgente ThemeDefault o SolidColor, validazione dei canali e opacità garantita.
- `surface.rs`: descrizione immutabile della superficie desktop.
- `snapshot.rs`: snapshot immutabile della superficie per frame.

Integrazioni:
- `src/lib.rs`: esportato il modulo desktop.
- `src/compositor.rs`: AureaCompositor possiede DesktopManager e offre accesso controllato.
- `src/compositor/rendering.rs`: ogni RenderScene riceve lo snapshot corrente del DesktopManager.
- `src/render.rs`: RenderScene conserva DesktopSnapshot.
- `src/vulkan/surface_compositor.rs`: il colore desktop della scena è usato sia per frame vuoti sia per scene con superfici.
- `src/vulkan/surface_compositor/readback.rs`: colore propagato alle vie readback, incluse le varianti con display color.
- `src/vulkan/surface_compositor/release.rs`: colore propagato alle vie di rilascio scanout KMS, incluse le varianti con display color.

### Comportamento introdotto

- Il desktop ha ora un modello di stato dedicato nel compositor, separato dalla Shell.
- Default: superficie opaca Light basata sul token `#edf4ff`.
- Tema Dark: superficie opaca basata sul token `#0d1320`.
- È possibile configurare un colore RGBA personalizzato; input non finiti o fuori intervallo vengono rifiutati e il canale alpha è normalizzato a 1.0.
- Il manager produce un DesktopSnapshot immutabile per il frame.
- Il Vulkan surface compositor usa lo stesso colore per il percorso normale, empty scene, readback e release-to-KMS. Il clear pass generico non conosce i temi.
- Il puntatore/Dock/Launcher non sono stati oggetto di questo intervento.
- Wallpaper image loading, texture upload, scaling/crop e persistenza NON sono ancora implementati.
- Non è ancora stata eseguita una prova su Fedora/hardware.

### Test aggiunti ma non eseguiti

Test unitari in `desktop/theme.rs`, `desktop/background.rs` e `desktop/manager.rs` coprono:
- token Light/Dark opachi e distinti;
- risoluzione del background tema;
- rifiuto di canali NaN/fuori intervallo;
- normalizzazione alpha;
- default Light;
- cambio tema;
- colore personalizzato indipendente dal tema e ritorno al background tema.

L'utente deve eseguire su Fedora `cargo fmt --all`, `cargo check -p aurea-compositor --features drm` e i test mirati/workspace secondo i tempi concordati. Nessun test/build è stato eseguito da ChatGPT.

### Stato e prossime azioni

Il primo vertical slice del desktop è stato scritto e pubblicato, ma NON è dichiarato compilante o funzionante finché Fedora non lo verifica. Prima del successivo sviluppo:
1. L'utente esegue format/check/test su Fedora e condivide l'output.
2. Correggere ogni errore reale prima di estendere il sistema.
3. Dopo esito positivo, implementare il caricamento wallpaper separando decoder, cache CPU, upload Vulkan e policy di scaling.
4. Integrare preferenze Shell solo tramite un contratto API chiaro; evitare duplicazione di DesktopManager.
5. Poi completare geometria per-output/multi-monitor e verificare il desktop visibile senza finestre.
6. Puntatore, Dock e Launcher restano fuori dalla priorità immediata.

Non affermare che AureaOS mostra già un desktop funzionante: è stato implementato il primo livello di policy e propagazione del colore di sfondo nel renderer, da validare su Fedora.


### Ultimo controllo repository — 2026-11-23

- HEAD corrente dopo sincronizzazione del diario: `3954457b621e166329ae8db26db3db2952100f73`.
- Il primo nucleo desktop è stato pubblicato su main con i commit elencati sopra.
- Le vie Vulkan sono state ricontrollate dopo la correzione: entrambe le varianti clear/readback e clear/release ricevono il colore desktop esplicitamente; nessun riferimento residuo al vecchio colore costante `BACKGROUND_COLOR` nei file readback/release verificati.
- La cronologia pubblica è stata aggiornata in italiano e inglese.
- Nessuna verifica Cargo è stata eseguita. La prossima azione pratica è far sincronizzare `main` su Fedora e ottenere `cargo fmt --all` + `cargo check -p aurea-compositor --features drm` prima di aggiungere wallpaper.


## STATO OPERATIVO — 2026-11-23 — PRIMO MODULO WALLPAPER

### Verifica prima dell'intervento

- HEAD di partenza verificato tramite cronologia Git: `5486fef16c8834fa4e9197d2f61eda8e456a3f3e`.
- Letto integralmente questo handoff prima di modificare il repository.
- Confermata l'esistenza dell'asset selezionato:
  `aurea-human-interfaces/resources/wallpapers/aureaos-blue-flow.jpg`.
- Il README della raccolta conferma che `aureaos-blue-flow.jpg` è il wallpaper predefinito provvisorio scelto; il documento dichiara anche che il caricamento runtime non era ancora implementato.
- Il workspace `aurea-compositor` non contieneva un decoder JPEG tra le dipendenze dichiarate o nel Cargo.lock. Non è stata aggiunta una dipendenza senza poter rigenerare e verificare il lockfile su Fedora.

### Codice pubblicato su main

Creato il sottosistema:
`components/aurea-compositor/src/desktop/wallpaper/`

- `mod.rs`: esportazioni del sottosistema.
- `asset.rs`: asset JPEG predefinito incorporato nel binario con `include_bytes!`; parser limitato ai marker JPEG SOF per verificare intestazione e dimensioni senza decodificare i pixel.
- `manager.rs`: selezione del wallpaper predefinito, snapshot immutabile, possibilità di cancellare la selezione e usare il fallback colore.
- `snapshot.rs`: riferimento immutabile all'asset selezionato per il frame.

Integrazione eseguita:
- `desktop/mod.rs` esporta i tipi wallpaper.
- `desktop/manager.rs` possiede `WallpaperManager` e include la selezione nel `DesktopSnapshot`.
- `desktop/snapshot.rs` trasporta superficie e wallpaper opzionale.
- `render.rs` mantiene il costruttore di fallback compatibile con il nuovo snapshot.
- La scena del compositor riceve quindi i byte JPEG originali attraverso il DesktopManager e il RenderScene.

### Limiti da non nascondere

Questo è il primo livello di asset management, NON il wallpaper già disegnato a schermo:
- il JPEG è incorporato nel binario e i suoi marker/dimensioni vengono verificati;
- il dato compresso è disponibile nello snapshot della scena;
- la decodifica JPEG in pixel RGBA non è ancora implementata;
- la creazione/upload della texture Vulkan non è ancora implementata;
- il rendering del wallpaper, il fit/crop per monitor e la cache GPU non sono ancora implementati;
- il colore desktop continua a essere usato dal renderer Vulkan come sfondo effettivo.

Non dichiarare il wallpaper visibile fino a quando decoder, texture upload, draw pass e percorsi empty/populated/readback/KMS sono integrati e verificati.

### Verifiche

- Nessun `cargo fmt`, `cargo check`, test o build è stato eseguito: restano responsabilità dell'utente su Fedora.
- Nessuna modifica alla ISO o agli script ISO.
- Il codice è stato scritto direttamente su `main`; i commit esatti sono nella cronologia Git successiva a `5486fef`.

### Prossimo passo tecnico

1. Su Fedora eseguire `cargo fmt --all -- --check` e `cargo check -p aurea-compositor --features drm`; correggere ogni errore prima di proseguire.
2. Se il primo controllo passa, selezionare e bloccare una libreria Rust di decodifica JPEG compatibile con il progetto, aggiungendola al Cargo.toml e rigenerando Cargo.lock su Fedora (mai lasciare lockfile non aggiornato).
3. Implementare un decoder isolato che converta JPEG in RGBA8 con limiti su dimensioni, overflow e memoria; testare input troncati e dimensioni non valide.
4. Collegare i pixel decodificati al percorso upload Vulkan esistente (riusare le primitive di upload/immagine già presenti, senza duplicare allocatori).
5. Aggiungere una texture wallpaper persistente e un pass di disegno sotto le superfici applicative; coprire sia scena vuota sia scena con finestre, readback e rilascio KMS.
6. Implementare modalità di adattamento e clipping per ogni output usando il RenderTarget e l'origine logica esistenti.
7. Aggiornare diario e questo handoff a ogni sessione, distinguendo sempre implementato, compilato, testato e validato su hardware.


### Chiusura sessione wallpaper — stato Git aggiornato

- HEAD `main` verificato dopo gli interventi: `75643e93f49e0f531809e6fe1f150cbbdf465787`.
- Commit di codice:
  - `88020d7` — nuovo modulo wallpaper e manager.
  - `3b8fe2b` — validazione JPEG e metadata.
  - `bae72e5` — snapshot wallpaper confrontabile.
  - `3b53c9a` — manager con default selezionato e fallback.
  - `4f76412` — integrazione nel DesktopManager.
  - `4441108` — snapshot desktop con wallpaper opzionale.
  - `ce2b4e4` — trasporto dell'asset nella RenderScene.
- Commit documentazione:
  - `24540e6` — handoff aggiornato con stato e piano.
  - `7d7a81b` — diario italiano.
  - `d5f688a` — diario inglese.
  - `75643e9` — stato runtime nel README dei wallpaper.
- Nessuna build o verifica Rust è stata eseguita. L'utente deve eseguire i controlli su Fedora.
- Il wallpaper non è ancora disegnato: manca la decodifica pixel e il pass Vulkan. Il prossimo step non va saltato: prima verificare compilazione e test del livello asset appena aggiunto, poi implementare decoder e texture.


## AUDIT TECNICO WALLPAPER — 2026-11-23 (PROSECUZIONE)

### HEAD e stato verificati

- Repository: `AleessandroCeccarelli/AureaOs`, branch `main`.
- HEAD osservato dopo i commit del primo modulo wallpaper: `0d8421d2475c19bb01d2f556c2ef02f5370383f9`.
- Riletto integralmente questo handoff prima di proseguire.
- Riletti su `main` i moduli wallpaper, DesktopManager, DesktopSnapshot, RenderScene, Vulkan surface compositor, texture cache e upload.

### Risultati dell'audit Vulkan

- `vulkan/shm_texture.rs` implementa già una texture RGBA8 campionabile, usando `VulkanSharedImage`, `VulkanImageView`, `VulkanUploadBatch` e `VulkanPendingUpload`.
- `vulkan/surface_texture_cache.rs` implementa una cache GPU limitata per superfici Wayland, indicizzata da `ObjectId` e revisione. Non va riutilizzata direttamente per il wallpaper: il wallpaper non è una superficie Wayland e ha un diverso ciclo di vita.
- `vulkan/surface_compositor.rs::render_frame` effettua il clear usando `scene.desktop().surface().background_color()`, prepara le texture delle superfici e, se non ci sono superfici visibili né cursore, esegue il ramo clear/readback o clear/release e ritorna subito.
- Nel percorso con superfici, il clear precede il ciclo che disegna le superfici; il wallpaper dovrà essere disegnato subito dopo il clear e prima del ciclo delle superfici.
- Il ramo scena vuota deve essere modificato: se esiste un wallpaper, deve comunque registrare e presentare il draw pass; non può più ritornare dopo il solo clear.
- La pipeline di composizione e il descriptor binding già supportano texture campionabili e regioni textured. Riutilizzare questi componenti.
- `RenderTarget` resta la fonte della geometria/scala/origine per-output; il wallpaper va adattato al rettangolo locale dell'output, non renderizzato con coordinate globali grezze.
- `VulkanShmTexture::submit_snapshots` accetta `ShmBufferSnapshot` e gestisce upload RGBA8 con risorse pending fino al completamento GPU. È una base riutilizzabile, ma l'asset wallpaper deve avere un adattatore/texture owner distinto e cache per asset/versione, non un ObjectId fittizio.

### Decodifica JPEG e Cargo

- Il Cargo.toml di `aurea-compositor` dichiara al momento soltanto `aurea-desktop-api`, `tracing` e Smithay.
- Una ricerca del Cargo.lock e del codice del repository non ha trovato `jpeg-decoder`, `image` o un decoder JPEG già direttamente disponibile.
- Non introdurre una dipendenza solo nel Cargo.toml lasciando Cargo.lock incoerente.
- Non scrivere un decoder JPEG proprietario: sarebbe un rischio di correttezza e sicurezza non giustificato.
- Il prossimo passo per la decodifica è scegliere una dipendenza Rust mantenuta, aggiungerla e generare/aggiornare Cargo.lock con Cargo su Fedora. Dopo il lockfile aggiornato, implementare un modulo decoder con limiti espliciti di dimensioni e byte, test su JPEG valido/troncato e dati non validi.

### Prossima implementazione, dopo la verifica locale

1. L'utente esegue su Fedora `cargo fmt --all -- --check` e `cargo check -p aurea-compositor --features drm` sul codice già pubblicato; correggere prima eventuali errori reali.
2. Aggiungere il decoder JPEG e aggiornare Cargo.lock su Fedora.
3. Creare un tipo CPU `WallpaperPixels` (dimensioni + RGBA8 con overflow check e limite memoria) distinto da `ShmBufferSnapshot`.
4. Estendere o generalizzare il percorso di upload texture senza accoppiare il wallpaper agli identificatori Wayland; mantenere risorse pending fino al completamento GPU.
5. Creare una cache wallpaper GPU dedicata, con invalidazione per asset e device e budget memoria.
6. Integrare il binding wallpaper nel compositor: clear fallback, draw wallpaper, draw superfici, draw cursore; garantire il draw anche nel caso di scena vuota, e coprire readback e release/KMS.
7. Aggiungere fit/crop/center per output, usando il RenderTarget e scissor locale.
8. Aggiornare diario e handoff dopo il codice e distinguere chiaramente implementato, compilato, testato e validato su hardware.

### Stato di questa prosecuzione

- È stato svolto audit del percorso di texture upload e composizione; nessun file Rust è stato modificato in questa prosecuzione.
- Nessun Cargo fmt/check/test/build è stato eseguito, in conformità alla regola di lasciare le verifiche Rust all'utente su Fedora.
- Nessun lavoro ISO, Dock, Launcher o puntatore.


## PROSECUZIONE DESKTOP — LAYOUT E PIXEL RGBA8 (23-11-2026)

L'utente ha autorizzato esplicitamente la creazione e modifica dei file necessari. Il lavoro resta sul branch `main`, con priorità desktop e senza build ISO.

### Codice aggiunto su main

- `components/aurea-compositor/src/desktop/wallpaper/layout.rs`
  - `WallpaperFitMode`: Fill, Fit, Stretch, Center.
  - `WallpaperLayout`: separa rettangolo sorgente (crop in pixel immagine) e rettangolo destinazione (pixel fisici locali dell'output).
  - Calcolo intero senza allocazioni; Center può produrre coordinate negative e richiede scissor dell'output.
  - Test unitari per crop, contain, stretch, center e dimensioni nulle.
- `components/aurea-compositor/src/desktop/wallpaper/pixels.rs`
  - `WallpaperPixels`: buffer RGBA8 posseduto, pronto per l'upload Vulkan.
  - Verifica dimensioni non nulle, overflow, lunghezza esatta e limite di memoria di 256 MiB.
  - Test unitari per buffer valido, lunghezza errata, dimensioni nulle, limite memoria e overflow.
- `wallpaper/mod.rs` esporta layout e pixel buffer.

### Commit di questa fase

- `7b15482` — aggiunto il buffer pixel wallpaper con limiti e test.
- `84b3f00` — esportato il tipo WallpaperPixels.
- `e1a1fdb` — corretto il test del caso di overflow dimensionale.

### Limite tecnico da non mascherare

Il draw pass Vulkan non è ancora stato inserito. Il motivo è concreto: la scena contiene ancora i byte JPEG compressi e non esiste ancora una texture Vulkan wallpaper. Il ramo scena vuota non può disegnare correttamente un'immagine senza decoder, buffer RGBA e texture campionabile. Non inserire una chiamata fittizia né riutilizzare la cache SHM tramite ObjectId inventati.

### Prossimi passaggi

1. Fedora: pull, fmt/check e test dei nuovi moduli.
2. Aggiungere un decoder JPEG mantenuto e generare Cargo.lock con Cargo su Fedora.
3. Convertire JPEG in WallpaperPixels con limiti di dimensione.
4. Creare upload/texture wallpaper e cache dedicata, mantenendo vive le risorse fino al completamento GPU.
5. Integrare il draw dopo il clear e prima delle superfici; il ramo scena vuota deve disegnare il wallpaper prima del readback/release.
6. Verificare readback, display-color pass e rilascio KMS.
7. Solo dopo il wallpaper statico, estendere i formati PNG/BMP.

Nessuna build, formattazione o test Rust è stato eseguito da remoto. I test presenti sono codice aggiunto, non risultati eseguiti.


## DECISIONE PRODOTTO — SFONDI ANIMATI E DINAMICI (23-11-2026)

L'utente vuole che AureaOS supporti anche sfondi animati, come esperienza aggiuntiva rispetto alla gestione classica di Windows e agli sfondi dinamici/animati di macOS. Il wallpaper già scelto, `aureaos-blue-flow.jpg`, resta lo sfondo predefinito: non va sostituito.

È stato creato `docs/design/WALLPAPER-SYSTEM.md` come specifica di riferimento. Punti vincolanti:

- Due categorie: video/immagini animate in loop e raccolte dinamiche di immagini/scene che cambiano in base all'orario (es. alba/giorno/tramonto/notte).
- Assegnazione per monitor, stessa immagine su tutti, e modalità panoramica Span su gruppo di monitor.
- Audio disattivato per default; nessun suono inatteso.
- Decodifica fuori dal render thread; il compositor riceve frame RGBA timestampati e pronti.
- Sospendere decoding/frame production quando sfondo coperto, output disabilitato, sessione bloccata o sistema sospeso.
- Ripresa secondo policy definita; frame statico di fallback in caso di errore.
- Limiti su dimensioni, frame rate, frame queue e memoria; rispettare battery saver e preferenza reduced-motion.
- Cache wallpaper separata dalle superfici Wayland; nessun ObjectId fittizio.
- Backend video non ancora scelto. Valutare GStreamer o FFmpeg solo dopo audit manutenzione, packaging Fedora/distro, licenze/codec, API Rust e fallback software/hardware. Non aggiungere dipendenze o lockfile a caso.

Ordine: prima wallpaper statico funzionante (JPEG decode + Vulkan upload/cache + draw); poi PNG/BMP e Tile/Span; poi interfaccia frame animato e clock deterministico; infine backend media e policy power/sessione/UI. Non saltare direttamente al video prima di avere un percorso statico affidabile.

La specifica distingue implementato, non ancora verificato, e pianificato. Nessuna build o test Rust è stato eseguito.


## IMPLEMENTAZIONE — CONTRATTI ANIMAZIONE WALLPAPER (23-11-2026)

Dopo la specifica di prodotto è stato aggiunto un primo livello di codice Rust, pubblicato su `main`.

File:
- `desktop/wallpaper/animation.rs`: policy di riproduzione, condizioni ambientali, decisioni Playing/Paused con motivo, frame RGBA timestampato, errore provider e trait backend-neutral `WallpaperFrameProvider`.
- `manager.rs`: `WallpaperManager` possiede ora anche `WallpaperPlaybackPolicy` e la trasferisce negli snapshot.
- `snapshot.rs`: `WallpaperSnapshot` include asset e policy.
- `mod.rs`: esporta i contratti pubblici.

Policy predefinita: 30 fps massimi, audio muto, pausa quando il wallpaper è coperto, output disabilitato, sessione bloccata, sistema sospeso o power saving attivo. Reduced motion forza la pausa. Il frame rate è limitato a 1–60 fps. Il provider produce frame RGBA8 con indice e presentation timestamp.

IMPORTANTE: è un contratto e una policy integrata nel WallpaperManager, NON un decoder funzionante. Nessun decoder GIF/APNG/WebP/video è stato aggiunto; nessun frame viene ancora renderizzato. Il trait provider deve essere invocato solo da un worker, mai dal render thread Vulkan. Il prossimo ordine resta: validare questo codice su Fedora, completare decode/upload/cache/draw statico, poi implementare provider GIF/video e collegare worker e coda di frame.

Commit:
- `8eb57d1` — contratti animazione e policy.
- `0734539` — policy integrata nel WallpaperManager.
- `67a2bff` — policy negli snapshot.
- `f650e00` — export API.

Non è stato eseguito Cargo fmt/check/test o build remota. Nessuna verifica hardware.



## STATO OPERATIVO — 2026-11-24 — RIPRESA WALLPAPER / DECODER JPEG

### Repository
- Repository: `AleessandroCeccarelli/AureaOs`.
- Branch: `main` (unica branch restituita dalla ricerca branch).
- HEAD osservato all'inizio della sessione: `66a0b6ce0657a8be35baa169ae1cb6d3fa5420eb`.
- La cronologia include i commit documentali del 24 novembre relativi al blocco di zune-jpeg 0.5.15 e alla verifica compatibilità Debian. Non è stata eseguita alcuna build remota.

### Priorità corrente
Completare il percorso del wallpaper statico in quest'ordine:
1. Selezionare un decoder JPEG stabile e compatibile con Debian 13.
2. Definire un adapter AureaOS piccolo, isolato e sicuro.
3. Decodificare su worker limitato in `WallpaperPixels` RGBA8.
4. Solo dopo, creare la texture/cache Vulkan dedicata e disegnare il wallpaper sotto le finestre e sopra il clear color, anche nella scena senza finestre.
5. Mantenere separati decoder, pixel CPU, upload/cache GPU e draw pass.

Non fare ISO prima di venerdì 27 novembre 2026, salvo nuova richiesta esplicita. Non eseguire Cargo fmt/check/test/build o CI remoti: le verifiche Rust sono eseguite dall'utente su Fedora al momento concordato. Lavorare su main e non creare branch.

### Decisione decoder provvisoria — non approvata
- Debian 13 Trixie amd64 fornisce `libturbojpeg0` e `libturbojpeg0-dev` 1:2.1.5-4.
- Fedora fornisce `libjpeg-turbo-devel` e `turbojpeg-devel`; le versioni esaminate appartengono alla serie 3.1.x.
- Le note upstream libjpeg-turbo 3.x documentano un wrapper di compatibilità TurboJPEG 2.x. Un adapter FFI AureaOS basato sull'API legacy è una candidata plausibile tra Debian 2.1 e Fedora 3.1, ma ABI, simboli, header e linking devono essere verificati sugli ambienti reali prima dell'approvazione.
- I binding Rust `turbojpeg` / `turbojpeg-sys` esaminati espongono API 3.0 `tj3*`: non usarli assumendo compatibilità con Debian 2.1.
- `zune-jpeg` 0.5.15 è bloccato per regressione cromatica; la correzione identificata è in 0.5.16-rc3, pre-release.
- `jpeg-decoder` 0.3.2 è in maintenance mode; Debian fornisce 0.3.0. Non selezionato come primario.
- Tenere tutta la parte unsafe FFI in un solo modulo; esporre un contratto sicuro verso `WallpaperPixels`; limiti su input, dimensioni, overflow e memoria; decodifica fuori dal render thread.

### Modifiche documentali di questa sessione
- Aggiornati `docs/development/DEPENDENCY-REGISTER.md` e `docs/it/REGISTRO-DIPENDENZE.md`.
- Aggiornati `docs/development/DEVELOPMENT-LOG.md` e `docs/it/DIARIO-DI-BORDO.md`.
- Nessun file Rust, Cargo.toml o Cargo.lock modificato.
- Nessun pacchetto installato; nessun build, check, format o test eseguito.

### Prossima azione esatta
Ispezionare l'header `turbojpeg.h` e la tabella dei simboli esportati dai pacchetti reali Debian 13 e Fedora, con un piccolo smoke test C/FFI solo dopo il consenso al primo ciclo di verifica locale. Non iniziare ancora la texture Vulkan.


### ABI probe prepared — not yet executed

- Added `tools/dependencies/jpeg/turbojpeg-abi-probe.c` and `tools/dependencies/jpeg/README.md`.
- The C probe exercises the legacy API `tjInitDecompress`, `tjDecompressHeader3`, `tjDecompress2`, and `tjDestroy`, outputting RGBA8 and checking opaque alpha.
- Safety limits: 64 MiB compressed input, 16384 pixels per axis, 256 MiB decoded RGBA8; initial color-space scope RGB/YCbCr/grayscale, explicit CMYK/YCCK rejection.
- No compile or execution has been performed. Do not claim that the ABI is validated.
- When the user is back on Fedora, run the exact commands in `tools/dependencies/jpeg/README.md`; then repeat against Debian 13's `libturbojpeg0-dev` 2.1.5. Only after both succeed should the Rust FFI adapter be implemented.


## DECISIONE ARCHITETTURALE MEDIA — 2026-11-24

- Decisione proposta: creare un crate Rust proprietario `aurea-media` come servizio/libreria in-process, non un daemon IPC nella prima fase.
- AureaOS possiede API, limiti, validazione, scheduling, errori, dispatch e integrazione. Il codec JPEG resta un backend sostituibile.
- Candidato backend: API TurboJPEG legacy via FFI Rust isolato in un solo modulo. Non implementare JPEG da zero.
- Confine: il servizio restituisce pixel RGBA8 CPU-owned; non conosce Vulkan, texture, wallpaper layout o scene compositor.
- Il decoder gira su worker in background, mai sul render thread. Valutare helper process ristretto prima di esporre decode a file/download/preview non fidati o app terze.
- Specifica completa: `docs/architecture/AUREA-MEDIA-SERVICE.md`.
- Debian 13 Trixie libturbojpeg 2.1.5; Fedora streams verificati 3.1.x. La compatibilità API legacy deve essere provata localmente su entrambi gli ambienti prima di introdurre crate/dependency.

### Correzione importante al probe

- L'ispezione dell'header TurboJPEG 2.1.5 Debian ha rivelato che `TJCS_DEFAULT` e `TJCS_UNKNOWN` non sono definiti nell'enum legacy 2.1.
- Corretto `turbojpeg-abi-probe.c`: il parametro di output colorspace viene inizializzato a `-1`, poi scritto da `tjDecompressHeader3()`.
- Aggiunta nota di compatibilità al README.
- `TJFLAG_LIMITSCANS` è presente in 2.1.5; non va rimosso.
- Probe non compilato né eseguito: la compatibilità effettiva resta non verificata.

## DIREZIONE PRODOTTO E STRATEGIA TECNOLOGICA — 2026-11-24

### Principio non negoziabile
AureaOS deve essere il desktop di tutti: bambini, anziani, principianti, studenti, creativi, professionisti, amministratori e sviluppatori. Un solo desktop coerente, senza edizioni stereotipate per età.
- Formula prodotto: semplice per impostazione predefinita, potente quando serve, accessibile fin dall'inizio.
- Accessibilità, multimedialità e produttività sono requisiti fondamentali.
- Progressive disclosure: non imporre la complessità ai principianti e non togliere strumenti agli utenti avanzati.
- Non confondere direzione prodotto con implementazione: documenti e catalogo devono indicare esplicitamente cosa è pianificato, candidato o verificato.

### Nuova specifica
- Leggere docs/architecture/UNIVERSAL-DESKTOP-TECHNOLOGY-STRATEGY.md prima di proporre nuove dipendenze o scelte UI/media.
- Tre livelli: fondamenta mature Debian; integrazione di infrastrutture Linux mature dietro UX/API AureaOS; ricerca/prototipi isolati per tecnologie nuove o acerbe.
- Debian stable è l'autorità di compatibilità; Fedora è sviluppo rapido, non prova Debian.

### Tecnologie raccomandate / da valutare
- PipeWire + WirePlumber per trasporto/routing audio-video e policy sessione; verificata disponibilità Debian Trixie (PipeWire 1.4.2, WirePlumber 0.5.8).
- GStreamer 1.x per pipeline media; pacchetti Debian Trixie esaminati nella famiglia 1.26.2. Non installare tutti i plugin/codec: verificare licenze e necessità per feature.
- XDG Desktop Portal 1.20.3 in Debian Trixie come interfaccia permessi/interoperabilità Flatpak. Serve un backend AureaOS per le interfacce supportate; non dipendere dai backend GNOME/KDE.
- AT-SPI2 come interoperabilità assistive; Orca per test reali; AccessKit come candidato per widget Rust, non sostituto di AT-SPI2.
- Parley e cosmic-text come candidati di text layout, non UI toolkit.
- UI toolkit nativo ancora da scegliere: prima prototipo reale con Wayland-only, Vulkan, AT-SPI2/AccessKit, IME, testo complesso, touch, scaling, performance e packaging Debian.

### Confini
- aurea-media: decode immagini e pixel CPU con limiti.
- aurea-capture: coordinamento sessione e autorizzazioni.
- PipeWire: trasporto/routing stream.
- GStreamer: pipeline playback/capture/encoding/muxing selezionate.
- aurea-compositor: superfici, frame finali autorizzati e presentazione Vulkan; mai codec, mix audio, muxing o scrittura file nel render thread.
- Nessun daemon multimediale monolitico ora. Valutare helper ristretto per media non fidati solo quando giustificato.

### Modifiche documentali del 24 novembre
- Creato docs/architecture/UNIVERSAL-DESKTOP-TECHNOLOGY-STRATEGY.md.
- Aggiornati VISION.md, docs/it/VISIONE.md, TECHNOLOGY.md e docs/it/PRINCIPI-TECNOLOGICI.md.
- Collegata la strategia dai README EN/IT.
- Aggiornati i registri dipendenze EN/IT.
- Nessun codice Rust, manifest Cargo, lockfile o pacchetto modificato; nessun build, format, check, test o CI eseguito.
- Continuare su main. Non fare ISO prima della data già concordata (27 novembre) e non eseguire verifiche Rust remote: l'utente le esegue su Fedora.

## CROSS-DEVICE SHARING — AUREA LINK / AUREA SHARE — 2026-11-24

- Read docs/architecture/AUREA-LINK-SHARE.md before proposing implementation.
- Aurea Link = device pairing/discovery, trusted peer identities, local transfer, receive lifecycle and safe storage.
- Aurea Share = system Share action and destination registry; it hands selected content to Aurea Link or installed supported apps such as Telegram/WhatsApp. The user confirms the final recipient/action in the app. Never automate private messaging APIs.
- First protocol candidate: published LocalSend Protocol v2.x (v2.2 spec reviewed), not a new proprietary wire protocol. No crate or dependency selected yet.
- Prefer authenticated HTTPS app-to-app transfer, receiver consent, integrity verification, bounded streaming, safe temporary storage and atomic publication. Discovery is not authentication. Add manual IP/QR fallback.
- Review current protocol license/security/upstream and Debian package/toolchain requirements before implementation. Protocol interoperability does not authorize copying the LocalSend application.
- KDE Connect is optional secondary interoperability. AirDrop/AWDL/OWL is experimental research only; no compatibility promise.
- Future mobile companion may integrate Android Sharesheet and iOS Share Extensions. Existing LocalSend clients can provide initial macOS/iOS/Android interoperability while Aurea companion work is deferred.
- No always-on monolithic daemon for milestone 1. Keep network transfers outside compositor, render thread, Aurea Media decoder and GStreamer pipeline.
- First milestone is local file/link/text transfer only. No live media streaming, cloud relay, message/notification/contact sync or automatic clipboard sharing.
- Docs created EN/IT; strategy, dependency registers, catalog, README and diary updated. No Rust/Cargo source changes and no checks run.

## AUREA SHARE ONLINE — SERVICE INTEGRATIONS — 2026-11-24

- Read docs/architecture/AUREA-SHARE-ONLINE-INTEGRATIONS.md and its Italian counterpart before implementation.
- User intent: from Fides or an image/PDF/media/document viewer, press Share and send/publish the currently selected local content to an online service without manually opening the destination app and reselecting/attaching the file.
- This is distinct from Aurea Link, which is local device-to-device transfer. Aurea Share includes online service adapters.
- Use official APIs only for supported account types/actions; fallback to official share/deep link or registered desktop app handoff. Never private endpoints, scraping, browser automation to evade limits or silent sending.
- Provider caveats: Telegram Bot API is not personal-account messaging; Telegram API/TDLib imply custom-client scope. WhatsApp Cloud API is Business Platform, not generic personal WhatsApp. Facebook Graph publishing is Page-oriented. Instagram publishing targets eligible Professional accounts. Reddit requires OAuth and may require explicit access approval; commercial use needs written permission. YouTube Data API uploads to authenticated channels, not chat attachments.
- Every adapter declares supported content/actions, account class, OAuth scopes, MIME/size/format limits, quotas, approval/commercial restrictions and whether final confirmation is required.
- OAuth: PKCE where supported, least privilege, system credential store, token revocation, no embedded confidential secrets. Always show account, destination, file and visibility before action.
- Keep online provider adapters outside compositor, decoder and GStreamer pipeline. Begin with core contracts and one provider at a time; no daemon without a demonstrated need.
- EN/IT architecture docs created; product catalog, universal technology strategy, dependency registers, README and diaries updated. No Rust/Cargo implementation, dependency, OAuth registration or tests were added.
## AUREA SHARE — ACCOUNT MODEL CLARIFICATION — 2026-11-24

- Do not design only for personal users or only for professionals. AureaOS must serve children, older adults, everyday users, creators, professionals and businesses without assuming persona.
- Account linking is optional and user-initiated, never required during OS installation.
- Build an account manager: connected accounts per provider, account class, permissions/scopes, connection state, capabilities, disconnect/revoke.
- Support multiple accounts per provider only where official provider auth/API permits; isolate tokens by provider and account.
- Provider capability depends on account class, scopes, app review, region, selected content and current provider limits.
- Share chooser remains useful with zero linked accounts via supported desktop handoff; offer account connection only when an API-backed action needs it.
- Read the account model sections in both EN/IT Aurea Share Online architecture documents before implementation.


## DESKTOP — POINTER MOTION OPTIMIZATION — 2026-11-24

- Audited the existing libinput → compositor pointer path and the multi-output coordinate constraint helper.
- Removed the temporary output-geometry Vec built in `AureaCompositor::handle_pointer_motion`; enabled output geometry is now passed directly as an iterator.
- Reworked `constrain_to_outputs` to evaluate outputs in one pass without collecting another Vec. It returns immediately for a point already inside an output, otherwise retaining only the nearest clamped point.
- Added unit coverage for non-finite requested pointer coordinates. Existing tests cover no outputs, gaps, edges, negative origins and mixed-scale layouts.
- Commits on main: `e0fee072` (single-pass constraint), `182ba0d5` (stream output geometry), `ced13560` (non-finite coordinate test).
- No behavior/API/dependency change intended. No Cargo fmt/check/test/build or CI was run. User must verify locally on Fedora; no ISO was generated.
- Continue the desktop work from the current repository state. Keep Debian 13 as compatibility authority and Fedora as development/build host. Generate and test an ISO only at a meaningful milestone, then validate on real hardware.


## WALLPAPER JPEG HEADER HARDENING — 2026-11-24

- Updated `components/aurea-compositor/src/desktop/wallpaper/asset.rs`: JPEG SOF metadata is accepted only when component count is non-zero and the segment length is exactly `8 + 3 * component_count`.
- Added tests for a valid minimal one-component SOF, a truncated component table and a zero component count.
- Commit: `0c7aa3f0`.
- This does not decode JPEG data and does not remove the ABI gate. Before introducing the Rust `aurea-media` backend, the existing C probe must compile and run against both Fedora development packages and Debian 13 Trixie libturbojpeg0-dev. Do not claim that gate passed without the user's local results.
- No Cargo manifest/dependency changes; no format/check/test/build/CI or ISO run.


## VULKAN WALLPAPER TEXTURE RESOURCE — 2026-11-24

- Added `components/aurea-compositor/src/vulkan/wallpaper_texture.rs` with `VulkanWallpaperTexture`. It consumes `WallpaperPixels`, allocates a sampled `R8G8B8A8_UNORM` shared image, uploads via the existing staging batch, records explicit UNDEFINED → TRANSFER_DST → SHADER_READ transitions, and waits for the upload fence before returning the owned texture.
- Exported the texture/error from `vulkan.rs` and re-exported `WallpaperPixels`/`WallpaperPixelsError` from `desktop/mod.rs`.
- Commits: `84aab939` (Vulkan module registration), `9a2e2ab6` (texture implementation), `00dc99bc` (desktop pixel-contract export).
- Important limitations: no wallpaper identity cache, no render-scene integration, no draw-under-surfaces yet. `from_pixels` waits for the one-time upload; call only from a resource-preparation/background path, never a render callback.
- JPEG decoder/ABI gate remains pending. No dependencies/manifests changed; no fmt/check/test/build/CI or ISO run. Local Fedora verification is required before the next integration step.

- Follow-up compile-safety review: `wallpaper_texture.rs` imports `WallpaperPixels` through the public `crate::desktop::WallpaperPixels` re-export (the internal `desktop::wallpaper` module is private). Fix commit: `881c88f9`.


## ASYNC WALLPAPER UPLOAD — 2026-11-24

- Commit: `8d0bb2e5` in `components/aurea-compositor/src/vulkan/wallpaper_texture.rs`.
- Added `VulkanWallpaperTexture::submit_pixels(Arc<VulkanLogicalDevice>, WallpaperPixels)`, returning `VulkanPendingUpload<Arc<VulkanWallpaperTexture>>`. The pending object retains staging and texture resources until its fence completes.
- `from_pixels` remains as a blocking convenience wrapper implemented via `submit_pixels(...).wait()`; never call it from the latency-sensitive render callback.
- This removes the synchronous-only limitation at the low-level upload API, but there is still no wallpaper cache, decoder, render-scene wiring, or draw-under-surfaces pass.
- No dependency/manifests changed. No Cargo fmt/check/test/build/CI or ISO run. User must run local Fedora verification; Debian 13 remains the compatibility target.


## WALLPAPER TEXTURE CACHE — 2026-11-24

- Commits: `7696e483` (new cache), `0e5ad381` (module registration), `0c5bc317` (per-device surface pipeline cache integration), `8e881c7d` (poll fences during frame preparation), `f09710bc` / `3c29e0bd` / `91a3087c` / `1f817319` (cleanup and explicit deferred-use annotations).
- New `VulkanWallpaperTextureCache` key is `{ asset_id: &'static str, revision: u64 }`, independent of Wayland ObjectIds. It keeps one selected ready texture plus up to two pending uploads, capped at 256 MiB aggregate pending bytes.
- Upload completion is polled at the start of Vulkan surface frame preparation. A texture is exposed only after its upload fence signals and only if its key is still selected. Stale completed uploads are discarded.
- `VulkanSurfacePipelineCache` now owns this cache and exposes crate-internal preparation, texture access, and clear entry points. The decoder does not yet provide pixels and the wallpaper draw pass is not implemented, so these preparation/access methods are deliberately marked as deferred integration points. Polling is active.
- No dependency/manifests changed. No Cargo fmt/check/test/build/CI or ISO run. User must run local Fedora verification; Debian 13 remains the compatibility target.


## WALLPAPER — OUTPUT DRAW GEOMETRY UPDATE — 2026-11-24

- New `components/aurea-compositor/src/desktop/wallpaper/draw_geometry.rs`: `WallpaperDrawGeometry` maps the existing fit layout to output-clipped destination pixels and normalized UV coordinates.
- Handles Fill source crop, Fit letterboxing, Stretch, and Center clipping. The returned destination is safe for output-local Vulkan viewport/scissor use; UVs are adjusted for clipping.
- New tests cover the four modes and empty/non-intersecting output. Types exported through `desktop`.
- Commits: `a5b33ccd`, `93284f76`, `804f9e67`, `96c49b75`.
- CPU geometry only; no shader, descriptor binding, wallpaper draw submission, JPEG decoder or display integration yet. No local checks run.
- Next: add dedicated wallpaper UV shader/pipeline path and integrate its draw before SHM surfaces, including the empty-scene path. Do not modify the existing surface shader push-constant ABI casually. JPEG decoder remains gated by the Fedora + Debian 13 TurboJPEG ABI probe.


## DEDICATED WALLPAPER SHADER / PIPELINE — 2026-11-24

- Added `shaders/wallpaper_texture.vert` and `.frag`. The vertex shader accepts a normalized UV rectangle as four float push constants; the fragment shader samples the wallpaper texture.
- `build.rs` now tracks and compiles these shaders for DRM builds and exports their SPIR-V output paths.
- `VulkanSurfacePipelineCache` owns a separate wallpaper pipeline per target format. It reuses the sampled-texture descriptor layout while declaring an independent 16-byte vertex push-constant range. The existing surface shader and `u32` transform ABI were not changed.
- Commits: `e0823ebe`, `fa3d28e4`, `41b24c1c`, `7a742fe6`.
- Important: pipeline infrastructure only. No wallpaper texture descriptor binding or draw call exists yet. No decoder/cache population exists yet. Do not claim wallpaper is visible.
- No Cargo fmt/check/test/build/CI or ISO was run. Next: add a draw-pass API for the UV push constant, build output-local viewport/scissor from `WallpaperDrawGeometry`, bind a ready cached texture, and draw immediately after clear and before SHM windows/cursor. Ensure the empty-scene path also records wallpaper. JPEG decoding remains gated on the user's Fedora and Debian 13 TurboJPEG ABI probe.

 
## VULKAN TEXTURED DRAW PASS — GENERIC PUSH CONSTANTS — 2026-11-24

## WALLPAPER UV PUSH-CONSTANT ENCODING — 2026-11-24

## WALLPAPER FRAME DRAW INTEGRATION — 2026-11-24

## WALLPAPER ASSET IDENTITY GUARD — 2026-11-24

- Added `texture_for_asset` to the wallpaper texture cache and exposed it through `VulkanSurfacePipelineCache`.
- Frame composition now matches the selected `DesktopSnapshot` asset ID to the ready GPU texture key before binding/drawing. This prevents rendering a stale previous wallpaper during a selection replacement upload.
- Commits: `e5faff5a`, `337b403a`, `4bbde807`, `2c2e0e3e`.
- Still no JPEG decoder or producer of `WallpaperPixels`; do not claim wallpaper is visible. No Cargo fmt/check/test/build, CI or ISO was run.

- `surface_compositor::render_frame` now binds a selected wallpaper only when its Vulkan upload is complete, then records the dedicated wallpaper pipeline directly after clear and before SHM surfaces and cursor.
- The frame uses output-local physical dimensions and `WallpaperFitMode::Fill`; a ready wallpaper is drawn even for an otherwise empty scene. Clearing wallpaper selection clears the GPU cache.
- Commits: `30582074`, `a347fde8`.
- Critical remaining dependency: no JPEG decoder or pixel producer currently calls `prepare_wallpaper_texture`; the default embedded JPEG is still metadata-only. Do not claim the wallpaper is visible. Do not add a decoder/dependency until the Fedora and Debian 13 TurboJPEG ABI probe is approved. No Cargo fmt/check/test/build, CI or ISO was run.

- Added `WallpaperUvRect::to_push_constant_bytes`, serializing min X, min Y, max X, max Y as four native-endian `f32` values matching the wallpaper shader's 16-byte `vec4` push-constant ABI.
- Added a round-trip unit test for component order. Commit: `f474ddfd`. Test authored, not run.
- This only prepares the CPU payload. Descriptor binding, draw recording, and empty-scene integration remain outstanding. Do not claim wallpaper is visible.

- Added `VulkanDrawPass::record_textured_region_with_push_constants`, allowing dedicated textured pipelines to provide their own push-constant bytes without changing the existing surface transform ABI.
- Existing surface draws still use the four-byte `u32` transform; the wallpaper pipeline can provide its independent 16-byte UV rectangle.
- Payload validation rejects empty, non-four-byte-aligned, and greater-than-128-byte data. Unit tests cover the 16-byte wallpaper payload and invalid sizes.
- Commits: `7b9b4b37` (draw-pass API), `d8a31412` (validation/tests). Tests were authored but not run.
- This remains infrastructure only: no wallpaper descriptor binding or draw submission has been integrated. Next: convert `WallpaperDrawGeometry` into a safe Vulkan viewport/scissor and 16-byte UV payload, bind a ready cached texture, and draw after clear but before SHM surfaces/cursor, including the empty-scene path. JPEG decoding remains gated on the Fedora + Debian 13 TurboJPEG ABI probe.
- No Cargo fmt/check/test/build, CI or ISO was run.


## 2026-11-24 — Wallpaper product decision superseding older handoff notes

Current decision: AureaOS wallpapers are still raster images only. GIF is rejected entirely (not even first-frame import); animated wallpaper and video playback are out of scope. Do not reintroduce the previous animation provider, frame queue, playback policy or GIF decoder.

The implementation on `main` has removed:
- `ImageFormat::Gif` and the `image` crate GIF feature;
- `WallpaperFormat::Gif` and GIF signature detection;
- the wallpaper animation module and playback state in manager/snapshot.

TIFF remains a still-image input for professional photography, scans and publishing. It can be lossless/high-bit-depth and large; multi-page TIFF is treated as a static image from its first page.

Current raster decoder set: PNG, BMP, TIFF and WebP, alongside the existing JPEG backend. HEIC/HEIF and AVIF remain pending. No build, test, rustfmt, Cargo check or CI was run for the GIF-removal changes; validate as a grouped block on Fedora and regenerate Cargo.lock there.

## DESKTOP POINTER SELECTION / CONTEXT MENU INPUT — 2026-11-25

- Commit `80483c23ea50ae1649fe9f2f553445a3b38018d7` wires `pointer_button.rs` into the existing `select_desktop_item_at` path.
- Primary click on empty desktop now selects the hit item, emits the existing activation intent on a recognized double click, or clears selection on empty space.
- Secondary click on empty desktop now selects/clears first and sends `ShowContextMenu` with global logical pointer coordinates. Window surfaces retain precedence; application right-clicks remain delivered to their Wayland clients.
- `DesktopPanelState` already retains panel kind and menu position, but there is no graphical context-menu surface or actionable entries yet. Do not claim it is visually displayed.
- Selection state reaches the frame snapshot, but Vulkan selection highlighting is not yet drawn. Trash icon rendering is also still gated on generating/visually inspecting the 24 RGBA8 assets and connecting the asset loader/cache.
- No Cargo fmt/check/test/build, CI or physical-device validation was run. Continue accumulating changes and leave grouped verification to Fedora; do not build after each correction.

## STATO CORRENTE — ISO LIVE SYSTEMD-BOOT / UKI — 2026-11-26

### Perimetro utente
- Obiettivo: avviare il desktop AureaOS reale dal repository in modalità Live, per osservare i progressi ISO dopo ISO su hardware fisico.
- Nessun desktop finto o sessione dimostrativa alternativa.
- Nessun installer, partizionamento, layout ESP/Btrfs installato, Aurea Recovery o aggiornamento kernel installato in questo blocco.
- Target ISO: Debian Trixie amd64, UEFI-only, systemd-boot, UKI, Secure Boot disabilitato. BIOS Legacy/CSM non supportato.
- Fedora + Podman + container Debian Trixie. L'utente usa Ventoy già configurato: copiare la ISO nella partizione dati esistente, non proporre di riformattare la chiavetta.
- Continuare su main; non fare build Rust/ISO/test remoti. Accumulare modifiche e verificare in un unico ciclo su Fedora.

### File aggiornati il 26 novembre
- tools/iso/os-release — identità AureaOS incorporata nella UKI.
- tools/iso/scripts/build-systemd-boot-live-iso.sh — genera UKI da kernel/initramfs live-build, crea immagine EFI FAT con systemd-boot e voci BLS, rimasterizza ISO UEFI-only.
- tools/iso/build-live-iso.sh — integra la fase UKI e la validazione finale; GRUB EFI resta soltanto scaffolding intermedio live-build e viene rimosso prima della ISO pubblicata.
- tools/iso/build-live-iso-podman.sh — installa systemd-boot-efi, systemd-ukify, mtools e dosfstools nel container Debian Trixie.
- tools/iso/tests/validate-live-image.sh — verifica ESP FAT, fallback EFI/BOOT/BOOTX64.EFI, systemd-boot, loader.conf, UKI, boot=live, filesystem Live e catalogo El Torito.
- tools/iso/hooks/020-brand-boot-menu.hook.binary — eliminato.
- ISO Factory EN/IT, log di sviluppo EN/IT e README aggiornati.

### Catena Live prevista
1. Firmware UEFI carica EFI/BOOT/BOOTX64.EFI dall'immagine EFI El Torito.
2. systemd-boot mostra AureaOS Live e lancia la UKI sulla stessa ESP.
3. UKI include kernel Debian, initramfs, os-release AureaOS e cmdline boot=live.
4. live-boot (initramfs-tools) cerca il supporto con /live/filesystem.squashfs.
5. systemd avvia aurea-desktop.service.
6. PAM/logind registrano la sessione tty1/seat0; il servizio lancia il binario Rust aurea-desktop.
7. Smithay/DRM/Vulkan inizializzano compositor, output e puntatore.

### Desktop realmente incluso
Il chroot hook continua a compilare:
cargo +stable build --manifest-path /usr/src/aureaos/Cargo.toml --release --locked --features heif --package aurea-desktop --bin aurea-desktop
Questo compila il crate desktop e il grafo Cargo collegato (desktop API, shell e compositor DRM/Vulkan). Non significa che ogni crate workspace sia già un servizio/app runtime: aurea-media e aurea-capture non diventano automaticamente componenti runtime se non collegati al binario o a un servizio.

### Rischi e verifiche pendenti
- Nessun build ISO, bash -n, shellcheck, ukify, mtools, xorriso, QEMU/OVMF o test fisico è stato eseguito dopo le modifiche. Non dichiarare il nuovo percorso avviabile.
- Il panic precedente /scripts/live resta irrisolto. Il preflight chroot controlla pacchetti, hook e script live, rigenera initramfs e verifica scripts/live; il generatore incorpora quello stesso initramfs nella UKI. Solo un boot reale può dimostrare il successo.
- Verificare sintassi/deps del generatore, path Debian Trixie di systemd-bootx64.efi e linuxx64.efi.stub, ukify build/inspect, mtools e xorriso.
- Verificare catalogo El Torito verso /boot/efi.img, contenuto ESP, UKI con sezioni kernel/initrd/.osrel/.cmdline e assenza di GRUB nell'immagine finale.
- Verificare live-boot con /live/filesystem.squashfs via Ventoy e avvio systemd del servizio desktop.
- La ESP predefinita da 256 MiB è solo per la ISO Live, non il layout da 1 GiB ipotizzato per il futuro sistema installato.

### Comandi Fedora per il prossimo ciclo autorizzato
```bash
git status --short --branch
git switch main
git pull --ff-only origin main
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo check -p aurea-compositor --features drm
bash -n tools/iso/build-live-iso.sh
bash -n tools/iso/build-live-iso-podman.sh
bash -n tools/iso/scripts/build-systemd-boot-live-iso.sh
bash -n tools/iso/tests/validate-live-image.sh
bash tools/iso/build-live-iso-podman.sh
```
Poi ispezionare checksum/El Torito/ESP e avviare da Ventoy su hardware. Registrare ogni risultato; non dichiarare riuscito ciò che non è stato osservato.

### Correzione successiva al primo pass ISO (26 novembre)
- Il coordinator ora copia `tests/validate-chroot-boot.sh` dentro `config/includes.chroot/usr/local/libexec/aureaos/validate-chroot-boot.sh`; prima creava la directory ma non copiava lo script invocato dal chroot hook.
- Il final validator controlla ora le sezioni `.linux`, `.initrd`, `.osrel`, `.cmdline`, il parametro `boot=live` e `ID=aureaos`.
- La documentazione chiarisce che il payload finale contiene `/live/filesystem.squashfs` e UKI, non kernel/initramfs separati. Ambito UEFI-only.
- Anche queste correzioni non sono state sottoposte a `bash -n`, build, ukify, ISO o boot fisico.

## 2026-11-26 — Current Live desktop milestone

- The immediate product target is a bootable Debian Live image that starts the real `aurea-desktop` binary from the repository. Do not begin the installer, disk partitioning, installed-system Btrfs setup, Aurea Update or Aurea Recovery in this milestone.
- Approved boot policy remains UEFI-only, systemd-boot + UKI, Secure Boot disabled. GRUB may remain an internal `live-build` staging dependency only if required by Debian live-build; it must not remain in the published ISO boot path.
- `tools/iso/scripts/build-systemd-boot-live-iso.sh` assembles the systemd-boot EFI image and UKI from the Debian Live kernel/initramfs. `tools/iso/tests/validate-chroot-boot.sh` checks the live-boot hook/script and confirms `scripts/live` is embedded in the generated initramfs. `validate-live-image.sh` validates the final UEFI payload.
- `tools/iso/build-live-iso.sh` now validates the binary tree and exact temporary ISO before publication. It retains uniquely named candidate ISOs, SHA-256 sidecars and JSON provenance manifests; the stable `AureaOS.iso` alias is promoted only after validation.
- The manifest records the exact Git commit and working-tree cleanliness. The source archive is `git archive HEAD`; uncommitted changes are therefore not included even when the manifest records a dirty working tree. Always commit and push the intended desktop state before generating a release candidate.
- The Podman wrapper installs the Debian Trixie build tools, including Python 3 for manifest generation. Run the same documented Fedora command; do not install build dependencies on the Fedora host beyond Podman/Git/sudo.
- **No verification has yet been performed for this publication change:** do not claim the script passes shell syntax checks, UKI generation, ISO validation, QEMU/OVMF or physical boot until those checks are actually run. The previous `/scripts/live` kernel panic is still an open hardware-validation issue.
- The current ESP image sizing is deliberately guarded. If the UKI does not fit, fail with the explicit size diagnostic; do not silently truncate the UKI or publish an image without it. Measure real Trixie kernel/initramfs/UKI sizes before changing the EFI image sizing strategy.
- Keep work focused on getting the current desktop and pointer visible in Live mode. No fake/demo desktop and no speculative new applications.

- Follow-up hardening: the Podman wrapper now derives its default build-parent name from the checked-out source commit and refuses a dirty/untracked repository. This prevents `RESUME=1` from silently reusing a completed live-build chroot containing an older desktop binary after a new commit. Resume only the same clean commit; a new commit gets a fresh workspace.
- The ISO validator's `--iso` mode now extracts `/boot/efi.img` to `$TEMP_DIR/boot/efi.img`, matching its later inspection path. Static fix only; still requires runtime validation.


## End-to-end audit — 2026-11-26

The ISO path was statically re-audited from package resolution through the first Wayland client. Corrections committed to `main`:

- Added missing `libheif-dev`, `libturbojpeg0-dev` and runtime `libturbojpeg0` to the target package manifest. `aurea-media` always builds its TurboJPEG C shim; the production desktop also enables HEIF. Build-only headers are purged after the binary is installed.
- Inner build workspaces now carry `.aureaos-source-commit`; dirty trees, missing markers and mismatched commits are rejected on resume.
- Chroot hook validates `ldd` output, runs `systemd-analyze verify`, and confirms `aurea-desktop.service` is enabled.
- The ISO coordinator generates the two bounded context-menu RGBA8 assets in the disposable Debian builder and stages only those raw files into the target rootfs. `librsvg2-bin` and Python are builder-only; Trash rasters remain deferred until the desktop-item draw pass uses them.
- Fixed Trash descriptor paths to be relative to the installed Human Interfaces root.
- Final-image validator extracts the actual SquashFS rootfs and verifies desktop executable, session launcher, systemd service, PAM definition, enablement symlink and exact asset payload lengths. Podman image now installs `squashfs-tools`.
- UKI helper passes the checked Debian `linuxx64.efi.stub` explicitly to `ukify`.
- Added explicit `live-config-systemd` because the image disables APT recommends; the chroot boot preflight now verifies the systemd late-userspace backend as well.

**No execution validation has been performed after these changes.** Do not claim the ISO boots. The next grouped Fedora check must first run shell syntax checks and the Podman ISO build, then inspect the manifest and validation logs. The Debian base gate is: Debian Trixie rootfs, systemd PID 1, required `live-boot`/`live-config` packages, and `scripts/live` present in the initramfs extracted from the exact UKI. Only after this gate passes should the test continue to the AureaOS service and desktop.

Important desktop boundary: source documentation still marks full scene composition and physical scanout validation incomplete. The Welcome client paints a real SHM preview after receiving XDG configure, but historical logs do not prove that configure or a visible frame was received/presented. The hardware test must capture those milestones explicitly.


## AGGIORNAMENTO ISO FACTORY — 2026-11-27

### Perimetro attuale richiesto dall'utente
La prima sessione Live deve limitarsi a:
1. boot da USB/Ventoy;
2. rilevamento del monitor e modalità video corretta;
3. wallpaper ufficiale `aureaos-blue-flow.jpg` (3840x2160) adattato all'output;
4. puntatore visibile e realmente mobile.

Nessun dock, launcher, icone desktop, Welcome window o altra UI in questa prova. Non ampliare il perimetro.

### Correzione sorgente ISO
- `tools/iso/build-live-iso.sh` crea lo snapshot con `git archive HEAD`; il repository deve essere pulito e le modifiche desiderate devono essere già committate. Non include modifiche non committate.
- Audit del 27 novembre: l'archivio includeva wallpaper e menu contestuale, ma ometteva `aurea-human-interfaces/icons/source/system/trash`. `components/aurea-shell/src/desktop/trash_artwork.rs` usa sei `include_bytes!` per gli SVG Light/Dark (empty/half/full). La cartella è stata aggiunta all'archivio e lo script ora verifica i sei file nello snapshot prima di avviare live-build.
- Il controllo preventivo del wallpaper 3840x2160 rimane attivo.
- Cursor source, manifest e tool Python sono inclusi nello snapshot; le due palette Xcursor vengono generate nel chroot e installate in `/usr/share/icons`.

### Dipendenze runtime e verifica finale
- La package list già dichiara kernel Debian, `firmware-amd-graphics`, `libvulkan1`, `mesa-vulkan-drivers`, DRM, GBM, libinput, libseat, udev, PAM/logind, TurboJPEG, libheif e plugin HEVC/AV1.
- Audit del 27 novembre: il validatore ISO ora verifica nel vero SquashFS i pacchetti runtime e i payload AMD amdgpu module/firmware, RADV ICD JSON, Vulkan loader, TurboJPEG e libheif.
- Questo controlla il contenuto dell'immagine, non la riuscita dell'inizializzazione GPU, del DRM/KMS scanout o del movimento fisico del mouse. Serve ancora il test Ventoy sul portatile.

### Commit delle modifiche di audit
- `46e5f42a75f665a5e72cf52cee81c738adab4f88` — aggiunta directory SVG Trash allo snapshot.
- `cfcfe9db7e0c2533d0f1d939e74fab78ddee129c` — preflight dei sei asset compile-time.
- `6ada63efdf940527fb4448721e7adaf3d144796c` — validazione dipendenze grafiche/media nel filesystem SquashFS.
- `952acde760e04f48e4f760fbbde0575cc089d728` — aggiornamento ISO Factory.

Nessuna build Rust, build ISO o prova hardware è stata eseguita durante queste modifiche. Prima della prossima ISO, controllare lo stato CI e poi lasciare che l'utente esegua la build Fedora/Podman secondo il flusso documentato.


### Correzioni aggiuntive del 27 novembre
- La package list Live ora include `xkb-data`: `libxkbcommon0` da solo non garantisce la presenza delle definizioni XKB sotto `/usr/share/X11/xkb`, necessarie alla mappatura della tastiera. Il validatore finale controlla anche il pacchetto.
- Il validatore in modalità `--iso` estrae ora l'intera directory `/boot`, non soltanto `boot/efi.img`. Prima, il controllo `ROOT/boot/grub` in modalità ISO poteva essere vuoto per costruzione e quindi non dimostrava l'assenza di GRUB nell'immagine masterizzata.
- Corretto il controllo dei file condivisi: il validatore non prova a dereferenziare librerie SONAME selezionate singolarmente da SquashFS, perché il target della symlink potrebbe non essere estratto. La presenza dei pacchetti è verificata nel database dpkg e la risoluzione delle librerie è già verificata con `ldd` nel chroot prima del purge.
- Commit aggiuntivi: `8fbcc701b8c0d6fe85199adb3729cd9901684c37`, `ecad5d469979e3b73c36e60c13ceeb3c49ae34d1`, `78290b4b0d46403b17a44365db67d1394bbb8e7f`, `036b55a6800e4e2bff9e1170fb6389aeeafa892e`, `06a96044fb712174ced80eec4ebd9f619199dfb6`, `37bb205158b3d8814b1e614b084f515f70c102ec`.


### Seconda passata tecnica — 2026-11-27
- Corretto un altro errore del validatore: Debian Trixie chiama il manifest RADV `/usr/share/vulkan/icd.d/radeon_icd.json` (non `radeon_icd.x86_64.json`). Il validatore estrae ora la directory ICD e `libvulkan_radeon.so`, verifica il riferimento nel JSON e la presenza della libreria. Evidenza Debian: file list ufficiale di `mesa-vulkan-drivers` Trixie amd64.
- Verificata la catena shader: `components/aurea-compositor/build.rs` usa `glslangValidator` con `--target-env vulkan1.3`; `glslang-tools` è presente tra le dipendenze temporanee della chroot e viene rimosso dopo la build.
- Verificata generazione cursori: usa moduli Python standard library, `rsvg-convert` e `xcursorgen`; i tool sono installati durante la build e rimossi dopo aver copiato i temi Xcursor nel rootfs.
- BLOCCO ARCHITETTURALE DA NON NASCONDERE: `VulkanRenderingFeaturePlan::from_device` rifiuta GPU con API inferiore a Vulkan 1.3 e richiede `dynamicRendering` e `synchronization2`. Gli shader sono compilati per Vulkan 1.3. La compatibilità 1.0–1.4 discussa in precedenza non è attualmente implementata. Prima di promettere l'avvio sul portatile occorre controllare la versione API esposta dalla GPU fisica (non quella del loader) oppure progettare un percorso renderer compatibile con versioni inferiori.
- Commit: `32ea575136aaf6a6d2b9a1efbed0c4f87b840b29`, `c9f76cec04d696c0af5e6980bbdd8ea0ba525a74`, `eeddc7a78717a91d3b0bf43fd223a6696bed14e8`.


### Terza passata tecnica — 2026-11-27
- Verificato il percorso di rendering di riserva: se la GPU associata al DRM non soddisfa i requisiti, AureaOS tenta un dispositivo Vulkan CPU (Lavapipe) e continua a presentare tramite GBM/KMS. La riserva richiede comunque Vulkan 1.3, quindi non risolve una mancanza di feature del dispositivo CPU; non va descritta come fallback universale.
- La factory installa già `mesa-vulkan-drivers`, che in Debian Trixie amd64 fornisce anche `/usr/share/vulkan/icd.d/lvp_icd.json` e `/usr/lib/x86_64-linux-gnu/libvulkan_lvp.so`. Il validatore controllava RADV ma non il payload Lavapipe: ora estrae e verifica anche manifest, riferimento JSON e libreria del driver CPU. Evidenza: file list ufficiale Debian Trixie amd64.
- La verifica riguarda la presenza dei componenti, non dimostra che Lavapipe riesca a creare il dispositivo o che il fallback sia selezionato correttamente sul portatile. Restano da provare runtime Vulkan e boot fisico.


### Boot-order and native-session audit — 2026-11-27
- Reconstructed the actual startup chain from the installed systemd unit through the PAM session, libseat/logind, udev DRM discovery, GBM, Vulkan, KMS outputs, Wayland output publication, pointer centering and Calloop rendering/input dispatch.
- The compositor deliberately does not sleep for an arbitrary duration or assume that device enumeration is complete at process launch: libseat owns access to the active seat; the Udev monitor supplies the DRM inventory; libinput is created against the same session handle and assigned to that seat. This is preferable to a fixed delay or a blanket `udevadm settle` dependency. systemd startup is parallelized, so `After=` expresses ordering only; hardware readiness remains an explicit runtime responsibility. Debian's systemd boot documentation describes this ordering model.
- Confirmed the compositor publishes outputs only after GBM/KMS surface initialization and validates that every enabled output has a retained native surface. The first frame is requested when entering the event loop; rendering fails explicitly if no GPU has an initialized Vulkan backend or if no compatible presentation path exists. Pointer centering occurs after outputs are published, and startup aborts if active outputs exist but the pointer cannot be placed.
- Strengthened the final SquashFS validator to check the complete systemd/logind/libseat/TTY contract: logind ordering and Wants, dedicated PAM registration, unprivileged `aurea` user, `LIBSEAT_BACKEND=logind`, `XDG_SEAT=seat0`, tty1 attachment, minimal live mode, session launcher and multi-user enablement link.
- This is static/source and image-validator coverage only. It does not prove the kernel's real device timing, logind session acquisition, DRM modeset, Vulkan device creation, physical pointer movement or visible wallpaper. Those remain tomorrow's Fedora/USB checks.


## 2026-11-27 — Vulkan loader dispatch and CI diagnosis

- The latest Rust CI run on the pre-fix commit `89b6d0bf` passed workspace compilation, DRM/Vulkan compilation, workspace tests, compositor tests, Debian 13 media compatibility and dependency audit. The formatting job failed on four Rust files; the exact rustfmt-only changes were applied on `main`.
- The dedicated Lavapipe offscreen smoke test then aborted in `ash` with `Unable to load cmd_begin_rendering`. The compositor was calling Vulkan 1.3 core dynamic-rendering commands while `VulkanContext::new` had requested only Vulkan 1.1 for the instance. The instance request is now Vulkan 1.3 (`Version::VERSION_1_3`) so the loader can expose the core 1.3 dispatch table used by this renderer.
- Fix commit: `452b6752`; formatting commits: `abcba730`, `cca1d72e`, `8e5d4c6e`, `01bf8b4e`.
- No Rust build, formatter, test, CI rerun, ISO build, QEMU/OVMF or physical-device test was run during this pass. The fix is source-level only until the user runs the grouped Fedora checks.
- Important: the new `legacy_render_pass.rs` module is only a render-pass/framebuffer primitive. It is not yet wired through draw passes, pipeline creation, image synchronization, wallpaper, cursor and surface composition. Do not mark Vulkan 1.0–1.2 compatibility as implemented or select it in AAG until the complete renderer path exists and is validated.
- The Live image remains UEFI-only systemd-boot + UKI. The ISO script and final SquashFS/UKI validator have been statically reviewed, but the new ISO has not been built or booted. The accepted physical milestone remains wallpaper plus visible, moving pointer only.


## 2026-11-27 — Live desktop readiness pass (continued)

- Re-read the actual wallpaper and cursor paths instead of relying on older TODO notes. Current source embeds `aureaos-blue-flow.jpg` directly in the compositor with `include_bytes!`; the desktop snapshot selects the default wallpaper; `VulkanSurfaceCompositor` starts background decode, polls the decode worker and GPU upload fence, binds the matching ready texture, and records the wallpaper draw before normal surfaces and the cursor. The native cursor path loads the selected Xcursor theme (with a built-in arrow fallback), uploads frames asynchronously, draws it in the compositor, and relative libinput motion updates Smithay pointer location and requests new output frames. This confirms source-level wiring, not visible output on physical hardware.
- `AUREAOS_LIVE_MINIMAL=1` is present in the Live systemd unit and suppresses the Welcome client. The target remains wallpaper plus pointer only.
- The most recent CI attempt (`45978950168`) passed workspace build/tests and Debian 13 media compatibility, but the formatting job still failed on three exact rustfmt differences in `legacy_render_pass.rs` and module/export ordering in `vulkan.rs`. Corrected those exact diffs in commits `2e21fb91` and `f674f8e5`.
- New CI run `45979780727` was created for `f674f8e5` and was still pending when checked. Do not treat it as passed until GitHub reports completion. The intermediate run on `2e21fb91` was superseded/cancelled by the subsequent push.
- Static source audit confirms the current renderer still intentionally requires physical-device Vulkan 1.3 plus `dynamicRendering` and `synchronization2`. Mesa's current RADV documentation says GFX6–7 expose Vulkan 1.3 and GFX8+ expose 1.4; the user's previously reported AMD Stoney/Radeon R2/R3/R4/R5 class should therefore be checked by the exact physical-device report from `vulkaninfo` on the target host, not inferred from the loader's instance version. Debian Live must also load the `amdgpu` kernel driver and RADV for the physical GPU. The Lavapipe route is only a fallback, not a substitute for proving DRM/KMS scanout.
- No local Fedora commands, new Rust build, tests, ISO build, QEMU/OVMF or physical boot were run in this pass. The ISO is not yet a verified bootable desktop. Tomorrow's sequence is: pull `main`; complete CI/source checks; build through the documented Podman ISO factory; inspect candidate manifest/checksum; boot Ventoy; verify DRM output, first wallpaper frame and physical pointer motion. If the GPU fails the 1.3 feature gate, stop and fix compatibility before claiming success.


## 2026-11-27 — Adaptive graphics test repair and compatibility boundary

- Re-opened the current adaptive_graphics.rs, Vulkan physical-device selection, rendering-feature negotiation, logical-device creation, shader build script and legacy render-pass primitive on main.
- Rust CI run 45981720509 completed with failure. Formatting, dependency audit, workspace check, compositor DRM/Vulkan check and Debian 13 media compatibility all passed. The workspace test job failed only in rejects_vulkan_13_devices_missing_required_core_features: a local variable named candidate shadowed the helper function, so the second test candidate could not be constructed (Rust E0618).
- Fixed the shadowing by giving the two test values distinct descriptive names. Commit: ffdc24fb46402eb98a9756bb9a54f410547b5e60. This is a source correction only; no CI rerun or local Rust tests were run.
- The live physical-device capability probe is correctly based on VkPhysicalDeviceProperties.apiVersion and VkPhysicalDeviceVulkan13Features, not the loader version. Both dynamicRendering and synchronization2 are required before the current renderer can be selected. Logical-device creation receives the verified feature plan.
- Current adaptive selection still has no usable hardware compatibility path: compatibility_renderer_available is always false in physical-device candidates. legacy_render_pass.rs supplies only a render-pass/framebuffer target. The active compositor pipeline, shader compilation target (vulkan1.3), image transitions, queue submission/synchronization, wallpaper, cursor and surface rendering still depend on the modern path. Do not turn on the compatibility flag or claim Vulkan 1.0–1.2 support.
- Mesa RADV current documentation states GFX6–7 (GCN 1–2) support Vulkan 1.3 and GFX8+ support Vulkan 1.4; actual support for AureaOS additionally depends on the exact device's required feature bits and DRM/KMS/GBM presentation. Source: https://docs.mesa3d.org/drivers/radv.html
- Next compatibility implementation sequence: (1) define the actual minimum API baseline and separate core-vs-extension feature negotiation; (2) create a legacy shader target (Vulkan 1.0/SPIR-V 1.0 where possible); (3) add legacy render-pass pipeline variants for surface, wallpaper and cursor; (4) provide Vulkan 1.0-compatible barriers/submission synchronization; (5) integrate the same frame/resource lifecycle and KMS presentation path; (6) add capability-policy and offscreen smoke tests for each backend. Until all six are complete and validated, AAG must keep the legacy path disabled.
- User's local verification remains grouped on Fedora; do not launch builds/tests or generate a new ISO remotely. Current next user-side check should include cargo fmt --all -- --check, cargo test --workspace, and cargo check -p aurea-compositor --features drm after pulling main.


## 2026-11-27 — Compatibility shader groundwork

- After the display-resolution policy update, resumed the Vulkan compatibility-renderer work on `main`.
- Audited the actual shader sources used by the compositor: surface texture, cursor trail, wallpaper texture, and display-color transform. The current shader sources use GLSL 450 constructs (vertex-index full-screen triangles, descriptor samplers, push constants, and ordinary texture operations); no Vulkan 1.3-only shader feature was identified in this source pass.
- Updated `components/aurea-compositor/build.rs` to compile a second SPIR-V variant for every existing DRM shader using `glslangValidator --target-env vulkan1.0`, while preserving the current Vulkan 1.3 outputs. The compatibility variants are exposed as separate `AUREA_*_COMPAT_SPV` compile-time environment variables for later renderer wiring.
- This is shader-toolchain groundwork only. No runtime code selects or loads the compatibility variants yet; AAG must continue to report CompatibilityVulkan unavailable. The complete legacy render-pass pipeline, legacy render scopes across all passes, device feature negotiation, and the shared wallpaper/surface/cursor frame path remain to be implemented.
- No build, shader compilation, rustfmt, cargo check, or tests were run. Do not claim the Vulkan 1.0 shader variants compile until the grouped Fedora verification runs.
- The current display policy is tiered: 1280x720/60 Hz minimum display compatibility, 1366x768/60 Hz low-end laptop reference, 1920x1080 or 1920x1200/60 Hz standard profile, and higher resolutions/refresh/multi-display as advanced certification tiers. This does not change the current Vulkan 1.3 runtime gate.



## 2026-11-27 — Legacy renderer integration audit

- Re-opened the current implementations on `main` before changing runtime code. The existing `VulkanLegacyRenderTarget` creates a one-color-attachment render pass and framebuffer, and its scope balances `vkCmdBeginRenderPass` / `vkCmdEndRenderPass`. It is still an isolated primitive.
- Confirmed the exact active-path blockers:
  1. `graphics_pipeline.rs` always creates pipelines with a null render-pass handle and chains `VkPipelineRenderingCreateInfo`; it has no traditional render-pass pipeline variant.
  2. `draw_pass.rs` always enters `VulkanDynamicRenderingScope`; it has no legacy target/scope route.
  3. `clear_pass.rs` and `display_color_pass.rs` also enter dynamic-rendering scopes directly.
  4. `logical_device_factory.rs` always calls `VulkanRenderingFeaturePlan::from_device`, whose current contract rejects API < 1.3 and requires both Vulkan 1.3 feature bits.
  5. `physical_device.rs` deliberately sets `compatibility_renderer_available: false`; AAG must remain disabled for legacy devices.
  6. `context.rs` requests a Vulkan 1.3 instance to ensure Ash's core 1.3 command dispatch is loaded for the active renderer. Lowering this request before removing all core-1.3 calls would reintroduce the known `cmd_begin_rendering` dispatch failure.
- The shared pipeline cache creates pipelines independently of a frame target, so the legacy render-pass handle must be created/reused in a format-compatible lifetime model; constructing a temporary render pass per draw and destroying it while a cached pipeline references it is not acceptable.
- Correct implementation order:
  1. Introduce an explicit renderer backend/feature plan (modern vs legacy) and query only feature structures valid for the physical device's API version; keep modern selection unchanged.
  2. Extend pipeline creation and cache identity so modern pipelines use dynamic rendering while legacy pipelines are created against a compatible persistent render pass.
  3. Add a backend-neutral render-scope interface and route draw, clear and display-color passes through it; preserve viewport/scissor, descriptor and push-constant behavior.
  4. Wire legacy SPIR-V variants into shader-module creation for surface, wallpaper, cursor and display-color pipelines.
  5. Replace any Vulkan 1.3-only synchronization/submit calls in the legacy frame path with Vulkan 1.0 barriers, fences and queue submission; retain current shared image-state/resource lifetime rules.
  6. Only then expose legacy capability to AAG and test modern + legacy paths, including wallpaper/surface/cursor composition and DRM presentation.
- No runtime code was modified in this audit. No build, rustfmt, tests, ISO build or hardware validation was run. This audit prevents a false claim that the isolated render-pass helper already constitutes a usable compatibility renderer.


## 2026-11-27 — Runtime shader-variant selection groundwork

- Added `VulkanShaderModule::from_renderer_variants`: it selects the modern SPIR-V when the logical device negotiated dynamic rendering, otherwise selects the Vulkan 1.0 compatibility SPIR-V. Existing `from_bytes` and validation behavior remain unchanged.
- Wired the selector into the compositor's cached surface, wallpaper and cursor-trail shader modules, and into both borrowed/shared display-color pass constructors.
- Commits on `main`: `a67fdfcf` (variant selection API), `01106adf` (surface/wallpaper/cursor cache wiring), `55609232` (display-color wiring).
- This is shader selection only. The current `VulkanRenderingFeaturePlan` still rejects devices below Vulkan 1.3 and always requires dynamicRendering + synchronization2, so all currently creatable logical devices continue to select the modern shaders. No legacy logical device can be created yet; AAG compatibility remains false. The traditional render-pass pipeline and draw/clear/display render scopes are still not wired.
- The selector assumes both SPIR-V variants are built from equivalent shader interfaces; this has not yet been verified by shader compilation or SPIR-V reflection. The build script currently exposes the compatibility shader environment variables added in the earlier shader groundwork.
- No build, rustfmt, cargo check, tests, CI rerun, ISO build or physical hardware test was run for these commits. Do not mark compatibility rendering implemented.
- Next implementation step: add an explicit renderer mode to the logical-device feature plan, querying Vulkan 1.0 core features safely for older devices, while preserving the modern 1.3 feature chain. Only after logical-device creation can instantiate a legacy device should the persistent render-pass/pipeline lifetime and render-scope integration proceed.

## 2026-11-27 — Vulkan capability inventory separated from backend activation

- Added `VulkanRenderingCapabilities::inspect` and `VulkanRendererMode` in `rendering_features.rs`. The inventory always queries Vulkan 1.0 core features and only chains `PhysicalDeviceVulkan13Features` when the physical device reports API 1.3 or newer.
- The inventory distinguishes `ModernDynamic` from `CompatibilityPending`. This is intentionally a statement about the current renderer implementation, not a claim that every older GPU is already renderable.
- Updated AAG candidate construction to report the actual queried `dynamicRendering` and `synchronization2` bits instead of deriving both from one modern-renderer boolean. The compatibility flag remains false, so no legacy GPU can be selected prematurely.
- Kept `VulkanRenderingFeaturePlan::from_device` strict: logical-device creation still accepts only the modern backend. This preserves the known-working code path and avoids creating a legacy device before traditional pipelines, render scopes and synchronization are integrated.
- Exported the capability model for subsequent backend negotiation and diagnostics.
- Commits: `8983a412` (capability inventory and explicit renderer mode), `786c084e` (AAG feature reporting), `429ef9b1` (public exports).
- Not yet run: rustfmt, cargo check, workspace tests, CI, shader compilation, ISO build or hardware tests. This is source-level implementation only.
- Next: inspect all active pipeline/render-scope call sites and design the persistent legacy render-pass lifetime. Then implement a mode-aware logical-device create-info that omits Vulkan 1.3 feature structures for the compatibility backend; do not enable that backend in AAG until its full render path is integrated.

## 2026-11-27 — Legacy graphics-pipeline creation groundwork

- Extended `VulkanGraphicsPipeline` with an explicit traditional-render-pass constructor, `new_with_render_pass`. It builds the same fixed-function state and pipeline layout as the modern path, but supplies the caller's render-pass handle and does not chain `VkPipelineRenderingCreateInfo`.
- The existing dynamic-rendering constructors remain unchanged in behavior and explicitly store `render_pass: None`. Legacy pipelines retain `Some(render_pass)`.
- Added render-pass compatibility inspection and tightened the existing dynamic-scope compatibility check so a legacy pipeline cannot accidentally be accepted by a dynamic-rendering draw pass.
- Lifetime contract is currently documented but not yet enforced by Rust types: the supplied render pass must outlive the pipeline. This is a known design item to resolve by separating reusable render-pass ownership from per-image framebuffer ownership before connecting frame recording.
- No draw/clear/display-color scope integration has been done yet. The new constructor is groundwork only; no claim of a working compatibility frame.
- Commits: `22800dfc` (traditional pipeline constructor), `a8a29e58` (prevent cross-mode pipeline acceptance).
- Not run: rustfmt, cargo check, tests, shader compilation, CI, ISO build or hardware tests.
- Next: refactor legacy render-pass resources into a reusable render-pass owner plus per-image framebuffer target, with a Rust-enforced lifetime relationship. Then add a mode-aware render-scope abstraction and adapt draw/clear passes without changing modern behavior.


## 27 NOVEMBRE — RENDERER VULKAN LEGACY: SEPARAZIONE RENDER PASS / FRAMEBUFFER

Modifiche pubblicate su `main`:
- `834b356d` — refactor iniziale di `legacy_render_pass.rs`: separati l'oggetto render pass e il target framebuffer; il target prende in prestito render pass e image view, e distrugge soltanto il framebuffer.
- `68ab0966` — il render pass riceve esplicitamente `&VulkanLogicalDevice` e `vk::Format`, così la sua ownership del device non dipende dalla durata di una image view.
- `52ff3d8e` — esportato il nuovo tipo `VulkanLegacyRenderPass`.

Nuovo modello:
- `VulkanLegacyRenderPass<'device>` possiede un render pass riutilizzabile per un formato e un load operation.
- `create_target(&view)` crea un framebuffer specifico per quella image view e controlla device e formato.
- `VulkanLegacyRenderTarget` prende in prestito sia il render pass sia la image view; Rust impedisce che il target sopravviva a una delle due risorse.
- Il drop del target distrugge solo il framebuffer; il drop del render pass distrugge il render pass.
- Lo scope RAII ora opera sul target e continua a bilanciare begin/end render pass.

Vincoli e stato:
- La creazione del render pass è ora indipendente dalla singola image view.
- La pipeline legacy usa ancora il costruttore con raw `vk::RenderPass`: la relazione di lifetime tra pipeline e render pass non è ancora codificata nei tipi. È il prossimo intervento, da fare senza rompere i costruttori moderni o i cache esistenti.
- Nessuna build, test o ISO eseguiti, in accordo con la richiesta dell'utente: verifiche raggruppate domani su Fedora.


## 27 NOVEMBRE — PIPELINE LEGACY VINCOLATA AL RENDER PASS

Modifiche aggiuntive pubblicate su `main`:
- `b4339833` / `6b596c28` — corretto l'export Rust dei tipi legacy nel modulo Vulkan (la precedente modifica aveva introdotto un separatore newline letterale errato).
- `7d1098a9` — aggiunto `VulkanLegacyGraphicsPipeline<'pass, 'device>`. Il wrapper possiede la pipeline Vulkan e prende in prestito `VulkanLegacyRenderPass<'device>`; il render pass non può quindi essere distrutto prima della pipeline. Il costruttore raw con `vk::RenderPass` è ora privato al modulo.
- `128f817c` — aggiunto il controllo dell'identità del dispositivo nel render pass.
- `3be59903` — esportato il nuovo wrapper.

Contratto:
- La pipeline legacy si costruisce da un riferimento tipizzato al render pass, non da un handle grezzo esposto al chiamante.
- Il wrapper espone le operazioni necessarie di bind, draw, viewport/scissor e push constants.
- La pipeline dinamica `VulkanGraphicsPipeline` e le cache moderne restano invariate.
- Il wrapper controlla che device e render pass coincidano; il formato è preso direttamente dal render pass, evitando divergenze tra formato dichiarato e formato effettivo.
- La durata del render pass è ora vincolata staticamente alla durata della pipeline legacy.

Prossimo blocco:
- Integrare il target e la pipeline legacy nel recorder di disegno senza confondere i due tipi di scope.
- Aggiungere un percorso clear tradizionale e verificare che le transizioni immagine siano compatibili con gli initial/final layout del render pass.
- Solo dopo collegare il percorso al selettore renderer compatibile.

Nessuna build, test o ISO eseguiti: le verifiche restano programmate su Fedora domani.


## 27 NOVEMBRE — PRIMO RECORDER LEGACY DI CLEAR E DRAW

Modifiche pubblicate su `main`:
- `9c67cf83` — `VulkanClearPass::record_legacy_color`: transizione immagine → scope render pass tradizionale → transizione finale.
- `6e3f6045` — il framebuffer target espone un riferimento tipizzato al render pass proprietario.
- `f7d0b75a` — `VulkanDrawPass::record_legacy`: percorso base di disegno non indicizzato nel target tradizionale, con verifica della compatibilità pipeline/render pass e transizioni esplicite.
- `085268b4` — il render pass conserva il load operation usato alla creazione.
- `d4b19210` — il recorder clear rifiuta render pass che non usano `CLEAR`.
- `67ae41ee` — il recorder draw base applica lo stesso controllo, evitando che il clear value venga passato a un render pass configurato per `LOAD` o `DONT_CARE`.

Stato della catena legacy:
- Render pass riutilizzabile e framebuffer per-image separati.
- Pipeline legacy vincolata al render pass tramite borrow Rust.
- Scope RAII per begin/end.
- Recorder minimo clear e recorder minimo draw con transizioni immagine prima e dopo.
- Il percorso moderno dinamico resta separato e non modificato nella sua logica.

Limiti ancora aperti:
- I recorder legacy supportano al momento il percorso base full-target; non sono ancora integrati i draw textured/region e le varianti display-color.
- La compatibilità delle dipendenze e gli stage/access mask delle barriere vanno riesaminati durante l'integrazione end-to-end.
- Il load operation è fissato alla creazione del render pass; per usare `LOAD` occorre un render pass creato esplicitamente con `VulkanColorLoad::Load` e un recorder dedicato che non presuma il clear.
- Nessun build/test/ISO eseguito: come richiesto, le verifiche saranno eseguite domani su Fedora.


## 2026-11-27 — Legacy textured-region recorder

- Added `VulkanDrawPass::record_legacy_textured_region` in `components/aurea-compositor/src/vulkan/draw_pass.rs`.
- The recorder uses the lifetime-bound `VulkanLegacyGraphicsPipeline`, a live `VulkanTextureBinding`, descriptor-set binding, caller-provided push constants and shader stages, and the same destination viewport/clipped-scissor model used by the dynamic textured-region path.
- It validates target color-attachment usage, pipeline/render-pass/device/format compatibility, finite image-state/load parameters, push-constant alignment/size, and scissor/viewport bounds before recording.
- The requested `VulkanColorLoad` is checked against the render pass's actual attachment load operation. This enables a future compositor sequence to use `LOAD` when overlaying a surface on an already-rendered wallpaper rather than clearing the output.
- The existing basic legacy recorder remains clear-only and unchanged; the modern dynamic-rendering path was not modified.
- Commit: `b6ce0e80`.
- No build, rustfmt, cargo check, tests, shader compilation, CI, ISO build or hardware validation was run, per the grouped Fedora verification plan.
- Important remaining integration work: legacy surface/wallpaper/cursor pipeline creation and persistent render-pass ownership in the per-format cache; legacy display-color pipeline/recording; end-to-end frame recording and presentation; and mode-aware logical-device creation/selection. AAG compatibility must remain disabled until these are complete and validated.


## 2026-11-27 — Legacy surface/wallpaper/cursor pipeline factories

- Added explicit traditional-render-pass pipeline factory methods to `VulkanSurfacePipelineCache`: surface texture, wallpaper texture, and cursor trail. Each factory uses the corresponding modern/compatibility shader pair and the existing descriptor-set/push-constant ABI.
- The factories return `VulkanLegacyGraphicsPipeline` values borrowing the supplied `VulkanLegacyRenderPass`; this deliberately avoids a self-referential cache and prevents a pipeline from outliving the render pass it was created against. The caller must retain the cache, render pass, and returned pipeline for their respective lifetimes.
- The cursor-trail vertex module now uses the renderer-variant selector too, rather than unconditionally loading the modern vertex SPIR-V.
- Strengthened `VulkanDrawPass::record_legacy_textured_region`: it now receives the source image view, verifies sampled usage and exact descriptor/source identity, rejects source/target aliasing and cross-device resources, transitions the source to shader-read and the target to color-attachment, then records the legacy textured region.
- Fixed the basic `record_legacy` path to remain independent of texture resources; it continues to validate and draw its own clear-enabled attachment.
- Commits: `bfa34c3d` (pipeline factories), `30f32017` (initial texture-source validation change), `78721c94` (corrected recorder placement and source/target transitions).
- These are source changes only. No build, rustfmt, cargo check, tests, CI, shader compilation, ISO generation or hardware validation was run. Tomorrow's grouped Fedora verification remains the gate.
- Compatibility is still not end-to-end: current logical-device creation remains Vulkan 1.3-only, and the renderer-variant selector cannot yet instantiate a legacy logical device. These factories prepare pipeline creation but do not activate AAG compatibility or prove wallpaper/surface/cursor presentation on older Vulkan hardware.


## 2026-11-27 — Legacy display-color pass groundwork

- Added `VulkanLegacyDisplayColorPass<'pass, 'device>` in `components/aurea-compositor/src/vulkan/display_color_pass.rs`.
- The new pass creates its display-color shaders through the modern/compatibility shader selector, creates a traditional graphics pipeline tied to a caller-owned persistent `VulkanLegacyRenderPass`, and owns its descriptor layout and linear-clamp sampler.
- Added source binding and a legacy recording method. Recording validates sampled/color-attachment usage, source-binding identity, descriptor-layout identity, device/dimensions/format, non-aliasing, render-pass compatibility and intensity range; it transitions source/target with Vulkan 1.0 image barriers and records the fullscreen triangle inside the traditional render-pass scope.
- The legacy display-color render pass is deliberately restricted to a render pass configured with `DONT_CARE`, matching the existing dynamic display-color pass semantics. The caller must supply the matching persistent render pass and per-image framebuffer target.
- Exported `VulkanLegacyDisplayColorPass` from the compositor Vulkan module.
- Commits: `5ffa53ff` (legacy pass implementation), `c59d6941` (public export).
- This is not yet connected to frame orchestration or the existing display-color manager, which still owns modern dynamic passes. The legacy render-pass lifetime/cache strategy and actual source/target image lifecycle must be integrated before the backend can be selected.
- No build, rustfmt, cargo check, tests, shader compilation, CI, ISO build or hardware validation was run. The implementation is source-level only; compile correctness and Vulkan behavior remain unverified until the grouped Fedora verification.
- Next: audit the frame orchestration and display-color manager interfaces, then introduce an explicit per-backend render-resource owner so the legacy render pass, pipeline, framebuffer targets and sampled bindings share a safe lifetime without self-referential caches. Keep AAG compatibility disabled and keep the logical-device Vulkan 1.3 gate unchanged.


## 2026-11-27 — Shared lifetime foundation for legacy render resources

- Added `VulkanLegacyRenderPass::new_shared(Arc<VulkanLogicalDevice>, format, load)`, returning a render pass with a retained shared device owner and a `'static` Rust lifetime.
- Refactored borrowed and shared constructors through one internal creation path; render-pass attachment format, load operation, subpass, dependencies and destruction behavior remain common.
- This is groundwork for a renderer resource owner that can cache render passes independently from a temporary borrow of the Vulkan logical device. It does not yet solve the pipeline's render-pass borrow: `VulkanLegacyGraphicsPipeline` still borrows a render pass, and framebuffer targets still borrow both render pass and image view.
- Additional display-color hardening: legacy pass now checks the same optimal-tiling format features as the modern pass (color attachment, sampled image, linear filtering) and reports missing target color-attachment usage distinctly. Commit `c83dcc6c`.
- Shared render-pass commit: `8264360d`.
- The display-color pass remains unintegrated with `VulkanDisplayColorManager` and `render_frame`; no compatibility renderer is advertised or enabled.
- No build, rustfmt, cargo check, tests, shader compilation, CI, ISO build or hardware validation was run. Changes remain source-level and unverified.
- Next: make the legacy graphics-pipeline wrapper own a shared render-pass reference (rather than a borrowed reference) and add shared descriptor-layout/sampler ownership for the display-color pass. Then build a per-format legacy resource manager and frame-scoped framebuffer targets. Only after that should a separate legacy frame-recording path be introduced.


## 2026-11-27 — Shared ownership for legacy graphics pipelines

- Added `VulkanSharedLegacyGraphicsPipeline` in `graphics_pipeline.rs`. It owns a traditional graphics pipeline plus an `Arc<VulkanLegacyRenderPass<'static>>`, so a long-lived resource owner can cache the pipeline without a self-referential borrow.
- Its constructor accepts a shared logical device, the shared render pass, descriptor-set-layout handles and push-constant ranges. It validates that the render pass belongs to the same Vulkan device and creates the pipeline against that exact render-pass handle (without `VkPipelineRenderingCreateInfo`).
- The wrapper exposes bind, draw, viewport/scissor and push-constant operations, and compatibility checks require the same shared render-pass allocation.
- Exported the new wrapper from the compositor Vulkan module. Existing borrowed legacy pipelines and all modern dynamic-rendering constructors remain available.
- Commits: `f07b83ee` (shared wrapper), `091704b8` (render-pass identity check), `b169249b` (public export).
- Descriptor-layout and sampler types already provide shared-device constructors, so no duplicate ownership abstraction was introduced.
- This is still resource-lifetime groundwork: no per-format manager, frame-target integration, display-color-manager integration or legacy frame orchestration has been added. The Vulkan 1.3 logical-device gate and AAG compatibility=false remain unchanged.
- No build, rustfmt, cargo check, tests, shader compilation, CI, ISO build or hardware validation was run. All changes remain unverified source-level work.
- Next: introduce a narrowly scoped per-format legacy resource owner that retains the shared render pass, shared descriptor layout/sampler and shared pipeline objects; create framebuffer targets per frame/image without storing self-referential borrows. Then integrate that owner with display-color and frame recording.


## 2026-11-27 — Per-format legacy render-pass cache

- Added `VulkanLegacyRenderPassManager` in `legacy_render_pass_manager.rs`. It owns persistent shared render passes, keyed separately by format and attachment load operation (`CLEAR`, `LOAD`, `DONT_CARE`).
- Framebuffer targets are deliberately excluded from this manager: they remain per-image/per-frame objects borrowing both the cached render pass and their image view.
- Integrated the manager into `VulkanSurfacePipelineCache` and exposed internal accessors for clear, load and discard render passes.
- Added shared-ownership legacy pipeline factories for surface texture, wallpaper texture and cursor trail. These factories build the Vulkan 1.0 render-pass pipelines against the exact shared render-pass allocation and use the existing descriptor-layout and push-constant ABI.
- Commits: `741390ab` (manager), `56c114c8` (module wiring), `6d24c140` (cache integration), `7256ba7c` (device ownership initialization order), `b3e688f9` (shared pipeline factories).
- Important boundary: these factories currently return owned pipeline wrappers; the cache caller is responsible for retaining them in a format/load-op keyed collection. The manager caches render passes only. Existing modern caches and borrowed legacy factories remain unchanged.
- No build, rustfmt, cargo check, tests, shader compilation, CI, ISO build or hardware validation was run. All work remains unverified source-level changes.
- Next: introduce a dedicated legacy pipeline resource collection keyed by format and load operation, then create a shared-ownership legacy display-color pass and connect it to a legacy display-color manager. After that, wire per-frame framebuffer targets and a separate legacy frame-recording path. Keep Vulkan 1.3 logical-device creation and AAG compatibility disabled until the full path is integrated and validated.


## Vulkan legacy continuation — 2026-11-27 (shared caches and display-color pass)

Additional source changes committed directly to `main`; none have been built, tested, formatted, or run on hardware.

- `55465f1`: shared legacy graphics pipelines can validate a borrowed per-frame render target by exact render-pass handle and format.
- `d601fca`: added `VulkanSharedLegacyDisplayColorPass`. It owns a shared legacy graphics pipeline, shared descriptor-set layout, and shared sampler. It validates format features, render-pass device/load-op, sampled source, target usage, binding identity/layout, aliasing, dimensions, device, and exact render-pass compatibility before recording Vulkan 1.0 transitions and a traditional render-pass draw.
- `c216ed0`: added `VulkanLegacyDisplayColorManager`, caching the shared display-color pass per output format. It owns the discard render pass used by the color transform.
- `51f7f42`: wired the shared pass and manager into the DRM-gated Vulkan module exports.
- `a53f07e`: integrated the manager into `VulkanSurfacePipelineCache` with `legacy_display_color_pass(format)`.
- `4fc7aca`: added shared legacy surface, wallpaper, and cursor-trail pipeline collections keyed by (format, load operation).
- `3a797bd`: added a distinct legacy render-pass error variant to the surface-pipeline cache rather than masking it as a generic pipeline error.

Cautions:
- The shared legacy surface/wallpaper/cursor pipeline cache methods currently exist but still need a caller/recording orchestrator.
- The legacy display-color manager is now reachable through the surface pipeline cache, but no compatibility frame path invokes it yet.
- Validate all lifetimes, method signatures, feature gates, and enum names during the next grouped Fedora build. Do not enable compatibility mode until the full Vulkan 1.0 path, including logical-device feature requirements and every recording call, has been audited.
- Continue respecting the no-build/no-test policy until the user authorizes the grouped validation.


- `41e63f8`: gated the legacy display-color manager field, import, initialization, and accessor behind the existing `drm` feature. The general surface-pipeline cache is compiled without DRM too, so this gate is necessary to avoid making non-DRM builds depend on DRM-only modules.


## Vulkan legacy draw recording — 2026-11-27

- `216456e`: added `VulkanDrawPass::record_shared_legacy_textured_region` and `record_shared_legacy_clear`.
  - Both operate with `VulkanSharedLegacyGraphicsPipeline`, whose render pass is retained through shared ownership.
  - Textured-region recording validates push-constant size/alignment, sampled usage, exact descriptor-to-view identity, no source/target alias, same device, target color-attachment usage, exact render-pass compatibility, matching load operation, and viewport/scissor bounds.
  - It records Vulkan 1.0 image barriers, begins/ends the traditional render pass, binds descriptors/push constants, sets viewport/scissor, draws, and transitions the target to the requested final state.
  - The clear recorder validates clear/load compatibility and records a full-target draw.
- These recorders are primitives for the later frame orchestrator; they do not yet replace `surface_compositor::render_frame` or make the legacy backend selectable.
- No build, test, rustfmt, shader compilation, or ISO build was run.
