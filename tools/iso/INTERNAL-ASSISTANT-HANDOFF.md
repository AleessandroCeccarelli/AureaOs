# INTERNAL ASSISTANT HANDOFF — AureaOS ISO / Desktop
<!--
Questo file è un promemoria operativo interno per la continuità delle sessioni ChatGPT.
Non è documentazione utente né una specifica del prodotto.
Aggiornare questo file quando cambiano script, diagnosi o stato delle prove.
-->

Ultimo aggiornamento: 2026-11-23
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
  `aurea-human-interfaces/resources/wallpapers/aureaos-midnight-flow.jpg`.
- Il README della raccolta conferma che `aureaos-midnight-flow.jpg` è il wallpaper predefinito provvisorio scelto; il documento dichiara anche che il caricamento runtime non era ancora implementato.
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

L'utente vuole che AureaOS supporti anche sfondi animati, come esperienza aggiuntiva rispetto alla gestione classica di Windows e agli sfondi dinamici/animati di macOS. Il wallpaper già scelto, `aureaos-midnight-flow.jpg`, resta lo sfondo predefinito: non va sostituito.

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
