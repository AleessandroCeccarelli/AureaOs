<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS — Libro di manutenzione, inventario e revisione tecnica

**Edizione:** 0.1 — prima ricognizione strutturale  
**Data:** 25 novembre 2026  
**Ramo esaminato:** `main`  
**Commit esaminato:** `238db4d9b14b9b55e48219c6a44b896543d3f9c9`  
**Scopo:** documento operativo per riprendere il progetto in sessioni future senza perdere decisioni, dipendenze, stato reale, problemi e procedure di verifica.

> Questo documento è un registro vivo. Non sostituisce il codice, i test, le decisioni architetturali o il diario cronologico. Distingue sempre fatti verificati, deduzioni da confermare e attività non ancora eseguite. La ricognizione è in corso: le sezioni marcate **DA COMPLETARE** non devono essere considerate certificate.

---

## 1. Regole operative non negoziabili

1. **Debian 13 Trixie amd64 è la piattaforma di destinazione.** Fedora è l'ambiente di sviluppo, non la base del prodotto.
2. **Wayland nativo, Smithay, Vulkan.** Nessun X11, XWayland o backend OpenGL nel progetto desktop.
3. Non introdurre dipendenze senza controllare disponibilità, versione, licenza, ABI, pacchetti Debian Trixie e comportamento in runtime.
4. Un modulo presente non equivale a una funzione completa. Tracciare sempre il percorso: configurazione → avvio → API → servizio → evento → UI → persistenza → test.
5. **Modularità:** un file deve avere una responsabilità chiara. Evitare file monolitici; obiettivo ordinario sotto 400 righe. I file più grandi richiedono motivazione e piano di suddivisione.
6. Non eseguire build locali, `cargo fmt` o test sul Fedora dell'utente a ogni modifica. Accumulare modifiche e fare una verifica raggruppata quando l'utente torna al computer. CI GitHub può essere usata per verifiche mirate.
7. Non generare ISO durante la fase di costruzione e audit del desktop. L'ISO si riprende solo quando concordato.
8. Non dichiarare verificato hardware, DRM/KMS fisico, sessione grafica completa o boot solo perché una build CI è verde.
9. Non copiare codice MAME. Il codice AureaOS deve rimanere originale e rispettare GPL-3.0-or-later e le licenze delle dipendenze.
10. Ogni decisione importante, correzione e verifica va registrata nel diario di sviluppo, con commit, workflow e limiti espliciti.

## 2. Visione e decisioni di prodotto consolidate

AureaOS Roma 1.0 è una distribuzione Debian con un livello desktop progettato e mantenuto dal progetto. La direzione è una GUI coerente, familiare a chi proviene da Windows/macOS, con strumenti grafici per le attività ordinarie.

### 2.1 Piattaforma e sistema
- Debian GNU/Linux 13 Trixie, amd64.
- Btrfs come filesystem predefinito scelto per l'installazione; Btrfs snapshot non è il flusso di backup previsto.
- Aurea Time dovrà gestire backup e ripristino.
- UEFI come direzione della prima release; Secure Boot escluso dalla prima release.
- La documentazione corrente indica systemd-boot/UKI come scelta architetturale più recente. Non confonderla con le precedenti discussioni su GRUB2: verificare e mantenere una sola decisione coerente nei documenti, script e manifest.
- GPL-3.0-or-later per il lavoro originale AureaOS; branding e proprietà del prodotto non significano software closed-source.

### 2.2 Stack desktop
- Rust, Smithay, Wayland, Calloop, libseat, DRM/KMS, Udev, GBM e Vulkan.
- PipeWire per multimedia, NetworkManager per rete, systemd per servizi.
- Flatpak e Flathub per applicazioni di terze parti.
- La compatibilità con applicazioni X11/OpenGL non è garantita dal solo supporto Flatpak: valutazione applicazione per applicazione.

### 2.3 Esperienza desktop richiesta
- Desktop nativo con finestre, focus, gestione finestre e puntatore.
- Dock flottante in basso; pulsante Aurea “A” e Super aprono lo stesso Launcher.
- Launcher sempre sopra le finestre, ricerca pronta, Super/Esc per chiudere.
- Dock con app fissate, app in esecuzione e indicatori; modalità visibile e auto-hide.
- Alt+Tab per passare tra finestre; quattro workspace.
- Quick Settings e notifiche; impostazioni coerenti e un'unica fonte di verità.
- Tema chiaro/scuro, Night Light, gestione display e profili colore.
- Desktop con Cestino predefinito, icone e file/cartelle posizionabili.
- Supporto a più monitor (obiettivo 3–4) con disposizione, risoluzione e frequenza.
- Gestione input: mouse, touchpad, touchscreen, tavoletta grafica e gesti.
- Screenshot e registrazione schermo sono obiettivi di prodotto, non vanno dichiarati implementati senza verifica.

### 2.4 Wallpaper
- Solo immagini statiche per il perimetro attuale. **Nessun wallpaper video**.
- Formati previsti: JPEG, PNG, BMP, TIFF, WebP, HEIF/HEIC e AVIF.
- Modalità: Fill, Fit, Stretch, Center.
- Sfondi animati in stile macOS restano fuori dallo scope attuale: non implementare GIF/video senza nuova decisione esplicita.
- Immagine di riferimento: `aureaos-midnight-flow.jpg`, 3840×2160, 16:9.

## 3. Cronologia tecnica sintetica e passaggio di fase

La storia va letta insieme a `docs/development/DEVELOPMENT-LOG.md`, che è il diario cronologico autorevole.

1. **Fondazioni e identità:** scelta Debian come base, nome AureaOS, licenza GPL-3.0-or-later, architettura modulare e identità visiva.
2. **Bootstrap e packaging:** script Debian, manifest dei pacchetti, locale inglese e tastiera italiana. La presenza dei manifest non equivale a rootfs validato o sistema installabile.
3. **Compositor nativo:** Smithay/Wayland, socket Wayland, seat, superfici SHM, keyboard/pointer, DRM device discovery e primi percorsi Vulkan.
4. **GPU e output:** inventario GPU, gestione DRM, registrazione/rimozione output, pubblicazione Wayland, geometria multi-output e scanout. La validazione fisica resta distinta dalla CI.
5. **Desktop e input:** finestre/workspace, selezione desktop, mouse, touchpad, touchscreen, tablet tool e gesti. Il supporto varia per famiglia di eventi e hardware.
6. **Shell e servizi:** modelli Dock/Launcher/pannelli, Night Light, tema, Cestino, gestione colore e API condivise. Gran parte della UI grafica finale e dell'integrazione end-to-end resta da completare.
7. **Wallpaper e media:** pipeline di caricamento/decodifica asincrona e rendering Vulkan; aggiunta del rilevamento HEIC/AVIF e integrazione opzionale libheif.
8. **Audit corrente:** fermare l'aggiunta casuale di feature, tracciare ogni collegamento e preparare verifiche riproducibili prima di riprendere lo sviluppo.

## 4. Inventario dei crate Rust

Il workspace `Cargo.toml` contiene sette crate. Il README che parla di cinque crate è obsoleto e deve essere aggiornato.

| Crate | Responsabilità osservata | Stato iniziale dell'audit |
|---|---|---|
| `aurea-desktop-api` | Contratti condivisi, interazioni, input, display, tastiera, desktop | Implementato come API; audit di stabilità e completezza contratti da completare |
| `aurea-shell` | Ciclo di vita sessione, runtime, eventi e stato desktop | Parzialmente integrato; modelli presenti, UI finale non dimostrata |
| `aurea-compositor` | Server Wayland, input, finestre, output, DRM e rendering Vulkan | Ampio prototipo implementato; hardware/integrazione end-to-end non verificati |
| `aurea-desktop` | Composition root, avvio sessione, factory e client Welcome | Entry point operativo in CI; non è ancora il desktop completo |
| `aurea-media` | Rilevamento e decodifica immagini/media | Decodifica immagini con HEIF opzionale; fixture reali e hardening da completare |
| `aurea-color` | Parsing/identità ICC, storage profili e trasformazioni colore | Moduli implementati; integrazione end-to-end display/UI da verificare |
| `aurea-capture` | Richieste, frame, coda e coordinamento cattura | Core presente; adapter compositor/media e workflow utente non implementati |

**Regola di manutenzione:** mantenere questo inventario aggiornato quando cambiano i membri del workspace o la responsabilità di un crate.

## 5. Avvio e collegamenti principali osservati

### 5.1 Avvio della sessione
Percorso osservato:
1. `components/aurea-desktop/src/main.rs` inizializza diagnostica, carica la preferenza layout tastiera e costruisce `DesktopCompositorFactory`.
2. `components/aurea-shell/src/app.rs` crea `DisplayColorController`, il canale touchpad, `ShellRuntime`, event bus e canale interazioni.
3. Il thread compositor invoca `CompositorFactory::create` passando display policy, interazioni e ricevitore touchpad.
4. `components/aurea-desktop/src/compositor_factory.rs` crea il compositor e avvia `WelcomeClientProcess`.
5. `components/aurea-compositor/src/service.rs` costruisce il server Wayland e applica le preferenze iniziali supportate.
6. Il runtime shell riceve eventi e interazioni compositor.

**Limite:** questo percorso avvia il compositor e un client Welcome. Non dimostra l'esistenza del Dock, Launcher, Settings o desktop grafico finale.

### 5.2 Tastiera e shortcut
- `components/aurea-compositor/src/compositor/keyboard.rs` inizializza XKB, inoltra gli eventi e filtra shortcut globali.
- `components/aurea-compositor/src/input/shortcuts/` risolve gli shortcut registrati.
- Attualmente la risoluzione documentata nel codice copre Super+Ctrl+Freccia sinistra/destra per i workspace.
- Super per Launcher, Alt+Tab e gli altri shortcut richiesti non risultano implementati in questo registry.
- I client Wayland focalizzati ricevono gli eventi non intercettati.

### 5.3 Touchpad
- Il percorso di produzione in `aurea-shell/src/app.rs` crea controller e receiver e passa il receiver alla factory del compositor.
- Il compositor applica i profili tramite libinput e `input/pointing/touchpad_settings.rs`.
- Le impostazioni vengono applicate per dispositivo e il codice distingue Unsupported da Failed.
- `ShellRuntime::with_display_color()` crea un receiver locale che viene scartato: non usarlo per un percorso che si aspetta l'applicazione dei comandi touchpad. È un costruttore secondario da rendere impossibile da usare in modo ambiguo o da documentare/limitare.
- Persistenza delle preferenze touchpad e UI Settings: da verificare.

### 5.4 Interazioni desktop e Shell
- `aurea-shell/src/desktop/` contiene modelli Dock, Launcher, Night Light, pannelli, workspace e stato Cestino.
- `aurea-shell/src/runtime.rs` applica cambi workspace, overview, context menu e selezione.
- L'attivazione di un elemento desktop viene conservata per un consumer successivo; il runtime commenta esplicitamente che non avvia applicazioni.
- I modelli non sono una UI: manca ancora il percorso completo eventi → superfici grafiche → azioni utente.

### 5.5 Wallpaper
- `aurea-compositor/src/desktop/wallpaper/` contiene asset, decodifica asincrona, layout, geometria, pixel e manager.
- `aurea-desktop/src/wallpaper_preferences_store.rs` salva path e modalità di scala.
- `aurea-desktop/src/main.rs` e `compositor_factory.rs` non caricano né passano attualmente lo store delle preferenze wallpaper.
- L'esistenza del decoder e del renderer non dimostra che la selezione dell'utente sia persistita e applicata a ogni output.
- Il parser del path presenta un problema di escaping: la sostituzione sequenziale di `\\n` e `\\\\` non è un decoder inverso corretto per tutti i path. Un path contenente letteralmente backslash seguito da `n` può essere alterato. Sostituire con escaping a scansione carattere per carattere o un formato robusto e aggiungere round-trip test.
- Il file temporaneo usa il PID e `create_new`; un file residuo dopo crash può impedire i salvataggi successivi. Definire pulizia/nomi temporanei collision-safe e testare gli errori.
- I test attuali del parser non coprono un vero round-trip del salvataggio su filesystem.

## 6. Stato CI verificato al 25 novembre 2026

Workflow più recente esaminato: [Rust CI #1782](https://github.com/AleessandroCeccarelli/AureaOs/actions/runs/45556415685), commit `238db4d9b14b9b55e48219c6a44b896543d3f9c9`.

| Job | Esito | Significato |
|---|---|---|
| Debian 13 media dependency compatibility | PASS | libheif Trixie, decoder HEIF attivo, desktop feature HEIF compilata |
| Rust dependency security audit | PASS | `cargo audit` sul lockfile usato dal job |
| Build and test workspace | PASS | script controllati, workspace check/test, compositor DRM/Vulkan, shader, smoke test lavapipe |
| Rust formatting | FAIL | `cargo fmt --all -- --check` segnala formattazione da sistemare |

Il workflow complessivo risulta **failure** perché il job di formattazione è fallito. Non descriverlo come CI completamente verde.

La CI non verifica:
- DRM/KMS su GPU fisiche;
- login/sessione su hardware reale;
- monitor hotplug e layout su più GPU/monitor;
- input su ogni dispositivo;
- qualità e sicurezza di decoder con fixture reali ostili;
- desktop grafico completo;
- installazione e boot di una ISO completa.

**Procedura concordata:** non correggere adesso il formatting. Accumulare le modifiche e affrontare `cargo fmt`, check e test in un unico passaggio su Fedora quando concordato.

## 7. Problemi e rischi già individuati

| ID | Priorità | Problema | Evidenza / area | Azione |
|---|---|---|---|---|
| AUD-001 | Alta | README dichiara cinque crate mentre il workspace ne ha sette | `README.md`, `Cargo.toml` | Aggiornare inventario e stato |
| AUD-002 | Alta | README riporta una CI verde del 18 novembre come ultima, ma esistono run successivi | `README.md`, GitHub Actions | Aggiornare status con run 1782 e limiti |
| AUD-003 | Alta | Preferenze wallpaper non collegate all'avvio/compositor factory | `main.rs`, `compositor_factory.rs`, store | Collegare caricamento, validazione, applicazione e salvataggio |
| AUD-004 | Alta | Decoder del path wallpaper non round-trip safe | `wallpaper_preferences_store.rs` | Escaping robusto e test round-trip |
| AUD-005 | Media | File temporaneo wallpaper può restare bloccato dopo crash | `wallpaper_preferences_store.rs` | Strategia temp collision-safe e recovery |
| AUD-006 | Media | Costruttore `ShellRuntime::with_display_color` scarta receiver touchpad | `runtime.rs` | Rimuovere/limitare API ambigua; test esplicito |
| AUD-007 | Alta | Shortcut Super→Launcher e Alt+Tab assenti dal registry esaminato | `input/shortcuts`, `keyboard_shortcuts.rs` | Implementare solo dopo disegno eventi/UI |
| AUD-008 | Alta | Dock/Launcher esistono come modelli, non come UI integrata dimostrata | `aurea-shell/src/desktop`, runtime | Completare consumer grafico e lifecycle |
| AUD-009 | Alta | README e documenti di stato rischiano di sovrastimare il prodotto | README, roadmap, design docs | Audit incrociato delle dichiarazioni |
| AUD-010 | Media | Due PR Dependabot aperte: ctrlc 3.5.3 e sha2 0.11.1 | PR #22 e #23 | Verificare API, MSRV, compatibilità, CI; non merge automatico |
| AUD-011 | Alta | Branch `feat/surface-output-tracking` ancora presente oltre a main e branch Dependabot | Branch list | Verificare commit, merge/cherry-pick o eliminazione dopo conferma |
| AUD-012 | Media | File Rust di dimensione rilevante richiedono verifica righe/responsabilità | `vulkan/surface_compositor.rs`, `compositor.rs`, `server/input/dispatch.rs`, test output | Contare righe, verificare responsabilità e pianificare split |
| AUD-013 | Alta | Lockfile e risoluzione dipendenze devono essere trattati come artefatti riproducibili | `Cargo.lock`, CI artifacts | Confermare lockfile versionato e allineamento dei job |
| AUD-014 | Alta | Wallpaper HEIF/AVIF necessita fixture reali, limiti e audit sicurezza | `aurea-media`, libheif, packaging | Testare HEIC/AVIF validi, corrotti, grandi, multi-image e plugin assenti |

Questa lista è una prima raccolta, non un elenco esaustivo. Le priorità potranno cambiare con l'audit dei singoli sottosistemi.

## 8. Vincoli e lacune da completare nell'audit

### 8.1 Compositor, DRM e Vulkan — DA COMPLETARE
- Tracciare creazione device → renderer → swapchain/framebuffer → compositing → present → recovery.
- Verificare ownership e cleanup in hotplug, disconnect, suspend/resume e crash.
- Verificare formati DRM/GBM, modifier, buffer import, fence/synchronization, stride, pitch e pixel format.
- Separare i test offscreen lavapipe dalla validazione GPU fisica.
- Verificare se il backend CPU è fallback intenzionale e quando può essere selezionato.
- Esaminare i file più grandi e i confini tra renderer, scene, readback e scanout.

### 8.2 Output e multi-monitor — DA COMPLETARE
- Enumerazione, identificazione stabile, EDID, connessione/disconnessione.
- Layout logico, coordinate negative, gap, scale, fractional scale, rotazione.
- Modalità risoluzione/refresh, clone/extend, primary output.
- Hotplug e ripristino dopo errori modeset.
- Mapping touchscreen/tablet/absolute pointer e comportamento quando sparisce un monitor.
- Test reali su 1, 2 e più monitor.

### 8.3 Input — DA COMPLETARE
- Mouse: movimento, click, doppio click, drag, scroll, focus e confini output.
- Touchpad: tap, drag, gesture, natural scroll, DWT, persistenza e hotplug.
- Touchscreen: slot, multi-touch, trasformazioni e calibrazione.
- Pen tablet: proximity, tip, button, pressure/tilt, mapping e calibrazione.
- Tablet pad: inventario sì; Smithay 0.7 non espone il relativo evento nell'InputEvent documentato.
- Switch/lid/tablet mode: solo diagnostica, policy energia/sessione da progettare.
- Accessibilità: tasti permanenti, tasti lenti, pointer keys, riduzione movimento e target grandi.

### 8.4 Shell e UX — DA COMPLETARE
- Dock grafico, Launcher, ricerca, app registry e lancio processi.
- Focus, stacking, Alt+Tab, minimizzazione, massimizzazione, resize, move, tiling se previsto.
- Quick Settings, notifiche, menu contestuali, drag/drop e clipboard.
- Tema e preferenze persistenti, Night Light scheduler, profili colore.
- Monitor Settings e UI di input.
- Desktop file/folder, icone, selezione, posizionamento persistente e Trash monitorata.
- Session lock/logout/shutdown e gestione crash app/compositor.

### 8.5 Media, capture e colore — DA COMPLETARE
- HEIF/AVIF: fixture reali, multi-image/sequence, metadata, HDR, memory limits, malformed input, decoder process isolation.
- JPEG/PNG/BMP/TIFF/WebP: supporto dichiarato e test per ciascun formato.
- Capture: collegare request/session/frame queue al compositor e alle destinazioni file/clipboard.
- ICC: parsing robusto, storage, profili display e pipeline effettiva; evitare che l'esistenza di API venga scambiata per color management completo.
- PipeWire audio/video: integrazione, permessi e portale.

### 8.6 Debian, boot e installazione — DA COMPLETARE
- Pacchetti Trixie, versioni minime, feature flags e dipendenze native.
- Bootstrap su rootfs pulito e verifica risoluzione pacchetti.
- Utente, gruppi, polkit, libseat, udev rules, servizi, rete, firewall e firmware.
- Btrfs partitioning/subvolumes/mount/initramfs.
- Coerenza tra scelta systemd-boot/UKI e manifest/script.
- Installer, recovery, update e rollback.
- ISO esclusa dall'attività corrente; nessuna nuova build ISO senza autorizzazione.

### 8.7 Sicurezza, licenze e supply chain — DA COMPLETARE
- Threat model compositor, parser media, IPC, filesystem, process launch e Flatpak.
- Permessi rootfs, symlink, file modes e confini utente/root.
- Licenze e provenienza di codice, font, icone, cursori, suoni e wallpaper.
- Dipendenze Rust e native Debian, advisory, MSRV, ABI e aggiornamenti.
- Verifica PR Dependabot senza assumere che un bump semver sia compatibile.

## 9. Manutenzione dei documenti

- Questo file: inventario operativo, collegamenti, stato e rischi.
- `docs/development/DEVELOPMENT-LOG.md`: cronologia append-only di modifiche realmente committate e verifiche realmente eseguite.
- `ROADMAP.md`: priorità di prodotto e stato macro, senza dettagli duplicati.
- `docs/architecture/*.md`: contratti e decisioni tecniche di sottosistema.
- `docs/it/`: documentazione italiana per utenti e visitatori.
- README: panoramica breve e stato corrente, non cronologia completa.

Quando un audit trova un problema:
1. Registrare il sintomo e il percorso esatto.
2. Riprodurre o dimostrare staticamente il problema.
3. Classificare gravità e impatto.
4. Individuare il proprietario del componente.
5. Proporre una correzione minima e test che falliscano prima e passino dopo.
6. Aggiornare il documento e il diario.
7. Non dichiarare risolto finché la correzione è verificata nel livello appropriato.

## 10. Procedura di verifica raggruppata su Fedora

Da eseguire solo quando l'utente è al computer e autorizza la sessione di verifica:

```bash
git status --short
git switch main
git pull --ff-only origin main

cargo fmt --all -- --check
cargo check --workspace
cargo check -p aurea-compositor --features drm
cargo test --workspace
cargo test -p aurea-compositor --features drm
```

I test hardware e Vulkan fisici sono separati e richiedono la macchina e i dispositivi appropriati. Non interpretare un errore di rustfmt come errore di compilazione; registrare separatamente ogni job.

## 11. Prossima sequenza dell'audit

1. Completare mappa repository, crate, moduli, dipendenze e file fuori standard.
2. Audit compositor/Vulkan/DRM e lifecycle.
3. Audit output e multi-monitor.
4. Audit input e shortcut, inclusi percorsi touchpad/touchscreen/tablet.
5. Audit shell, Dock, Launcher, desktop e servizi.
6. Audit wallpaper/media/capture/colore.
7. Audit Debian, packaging, boot, security e CI.
8. Correggere le incongruenze documentali e pubblicare inventario italiano/inglese.
9. Solo dopo l'audit, ordinare le correzioni per rischio e dipendenze e riprendere lo sviluppo.

---

**Nota di affidabilità:** questo documento registra una prima ricognizione del ramo `main` al commit indicato. Le conclusioni su funzioni non testate sono marcate come statiche o non verificate. Non costituisce una certificazione di release né una dichiarazione che AureaOS sia pronta per l'uso quotidiano.
