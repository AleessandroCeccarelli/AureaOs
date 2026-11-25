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
- Il registry globale risolve `Super + Ctrl + Left/Right` per i workspace, `Super + Ctrl + Up` per Overview e `Super + Ctrl + Down` per App Exposé.
- Le quattro combinazioni sono filtrate dal compositor prima dell'inoltro al client Wayland; press, repeat e release della combinazione riconosciuta vengono consumati dal filtro tastiera.
- I cambi workspace usano l'operazione compositor che aggiorna anche il focus; Overview e App Exposé passano dal canale semantico compositor-shell e attivano i rispettivi stati pannello.
- Le gesture restano eventi touchpad: su un sistema senza touchpad non vengono generate. Le scorciatoie sono l'alternativa indipendente per mouse e tastiera.
- Super per Launcher e Alt+Tab non risultano ancora implementati in questo registry.
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


## 12. Audit approfondito — GPU, Vulkan e presentazione DRM/KMS (25 novembre 2026)

### 12.1 Percorso di inizializzazione verificato

Percorso staticamente ricostruito:
1. `server/vulkan_lifecycle.rs::initialize_vulkan_backends` enumera ogni DRM node aperto.
2. Per ogni node prova `VulkanBackend::new_for_drm_node`, che crea un `VulkanContext`, cerca una GPU Vulkan associata al DRM node e crea il logical device.
3. Se la GPU abbinata non è utilizzabile, prova `VulkanBackend::new_software_fallback`, che seleziona un dispositivo Vulkan CPU (es. lavapipe).
4. Se anche il fallback CPU non è disponibile, il backend non viene registrato per quel node.
5. Il renderer aggiorna il metadata hardware/software della GPU e inserisce il backend nella mappa keyed by DRM node.
6. `server/rendering.rs` sceglie per ogni output una GPU di rendering inizializzata e una GPU di presentazione. Se coincidono e tutte le capability sono presenti, sceglie direct Vulkan; altrimenti tenta la composizione Vulkan + readback CPU + upload GBM/KMS.
7. Direct Vulkan può essere disabilitato per output dopo un errore pre-presentazione; se il frame è già pending o il device è lost, non viene tentato un retry potenzialmente insicuro.

Questo percorso è più maturo di un semplice renderer offscreen: il codice ha un dispatch per output e una via CPU-readback. Rimane tuttavia una verifica statica; l'effettiva affidabilità di ogni combinazione GPU/driver/connector richiede hardware reale.

### 12.2 Requisito Vulkan 1.3: scelta esplicita, impatto rilevante

I seguenti punti rendono il requisito Vulkan 1.3 un vincolo reale, non solo una preferenza:
- `vulkan/context.rs` crea l'istanza con `Version::VERSION_1_3`.
- `vulkan/rendering_features.rs` rifiuta GPU sotto Vulkan 1.3 e richiede `dynamicRendering` e `synchronization2`.
- `vulkan/logical_device_factory.rs` abilita queste feature nel logical device.
- Shader e pass principali sono compilati/documentati per Vulkan 1.3.
- `HARDWARE-REQUIREMENTS.md` esclude esplicitamente GPU senza Vulkan 1.3 e tali feature.

**Decisione attuale nel repository:** Roma 1.0 richiede hardware Vulkan 1.3. Il desiderio espresso in una fase precedente di coprire Vulkan 1.0–1.4 non è compatibile con questa decisione attuale. Non abbassare il requisito senza un progetto di backend alternativo: servirebbero feature negotiation, fallback render pass tradizionale o altro backend, shader target differenziati e una matrice hardware/driver dedicata. Il requisito 1.3 riduce il parco macchine supportato ma semplifica il renderer iniziale.

**Rischio di prodotto:** il portatile Fedora AMD A9/Stoney menzionato nei log precedenti deve essere trattato come hardware da verificare, non come prova di supporto. L'utente ha già osservato Vulkan loader 1.4, ma ciò non dimostra che la GPU esponga Vulkan 1.3 o le due feature richieste. La CI lavapipe verde non sostituisce la prova su quella GPU.

### 12.3 Direct scanout e fallback

- Il percorso direct richiede atomic KMS, GPU di rendering uguale alla GPU di presentazione, estensioni DMA-BUF, formato/modifier compatibile e `VK_EXT_queue_family_foreign`.
- `VK_EXT_image_drm_format_modifier` è opzionale nel piano estensioni; il codice di `VulkanFormatCapabilities::query` però restituisce una lista vuota se l'estensione non è esposta. Non sintetizza il modifier LINEAR dai flag generici di tiling, scelta prudente per non scambiare linear image support per DMA-BUF esportabile.
- Il percorso CPU compatibility usa rendering Vulkan, readback in memoria host e upload su GBM/KMS. Non equivale a rendering CPU puro: il fallback software richiede un dispositivo Vulkan CPU compatibile con il requisito 1.3.
- Il codice non tenta fallback dopo `VK_ERROR_DEVICE_LOST`, né dopo una submission/presentazione di stato incerto. Questa è una scelta di sicurezza della ownership, non un difetto da rimuovere con un retry generico.
- Se una GPU non ha un renderer hardware compatibile ma non esiste un device Vulkan CPU 1.3 funzionante, il backend per quel DRM node non viene creato e il rendering per quell'output non è disponibile.

### 12.4 Incongruenza documentale da risolvere

`VULKAN-DEVICE-FOUNDATION.md` afferma che, quando `VK_EXT_image_drm_format_modifier` manca, l'implementazione può interrogare il percorso core linear-tiling. Il codice attuale in `vulkan/format_capabilities.rs` interroga i modifier solo quando l'estensione è presente e restituisce altrimenti una lista vuota; inoltre commenta espressamente che non deve inventare `DRM_FORMAT_MOD_LINEAR`. Aggiornare la documentazione per distinguere:
- query Vulkan core per immagini lineari locali;
- export DMA-BUF con modifier e layout verificati;
- effettiva importabilità GBM/KMS.

Non si deve modificare il codice per “far combaciare” la documentazione senza prima provare la semantica corretta delle API e del driver.

### 12.5 Lifecycle, sincronizzazione e recovery

Il codice separa correttamente, almeno a livello di modello:
- completamento del lavoro GPU;
- proprietà temporanea dell'immagine da parte della foreign queue/KMS;
- page-flip completion;
- reacquire Vulkan;
- retirement della risorsa.

`DrmOutputFrameCoordinator` e `DrmPresentationLifecycle` tracciano questi passaggi. La distruzione di `VulkanLogicalDevice` attende `device_wait_idle`; se l'attesa fallisce, il codice conserva intenzionalmente viva l'istanza invece di distruggere risorse potenzialmente ancora usate.

**Limite deliberato:** il device-loss è fail-stop. Non esiste ancora ricreazione completa di Vulkan device, cache texture, pool frame, binding output e pipeline. La sessione può terminare in caso di device loss. Non aggiungere un retry locale: occorre un lifecycle di ricostruzione transazionale.

### 12.6 Test e limiti

- Sono presenti test unitari per selezione GPU, output, format/modifier, frame pool, frame coordinator, readback e rendering offscreen.
- CI esegue check/test workspace, feature DRM/Vulkan, shader compilation e smoke test lavapipe.
- Non risulta un test end-to-end che crei una finestra Wayland client reale, la componga e la presenti su display fisico.
- Nessun test CI può certificare hot-unplug, suspend/resume, multi-GPU ibrida, driver NVIDIA, scanout atomic reale o recupero da device-loss.

### 12.7 Nuovi elementi del registro audit

| ID | Priorità | Elemento | Classificazione | Azione |
|---|---|---|---|---|
| AUD-015 | Alta | Requisito Vulkan 1.3 non allineato al precedente obiettivo 1.0–1.4 | Decisione di prodotto da confermare; il repository attuale è esplicito | Mantenere il requisito 1.3 per Roma 1.0 salvo decisione nuova e progetto tecnico |
| AUD-016 | Media | Descrizione del fallback linear-tiling in `VULKAN-DEVICE-FOUNDATION.md` non corrisponde al codice attuale | Documentazione incoerente | Correggere la distinzione core linear vs DMA-BUF modifier |
| AUD-017 | Alta | Il fallback software richiede comunque Vulkan CPU 1.3 | Limite di compatibilità | Documentare come requisito; verificare lavapipe Trixie e CPU reali |
| AUD-018 | Alta | Nessuna ricreazione del renderer dopo `VK_ERROR_DEVICE_LOST` | Mancanza deliberata di recovery | Progettare in futuro un lifecycle globale; non tentare retry parziali |
| AUD-019 | Alta | Mancano prove di scanout su GPU/driver fisici rappresentativi | Non verificato | Piano hardware AMD/Intel/NVIDIA, laptop ibridi e multi-monitor |
| AUD-020 | Media | Il percorso direct è disabilitato per output dopo un errore recuperabile | Fallback per-output presente | Verificare che la disabilitazione venga resettata solo quando il modo/device cambia e non perda opportunità di riattivazione |

### 12.8 File di riferimento del sottosistema

- `components/aurea-compositor/src/server/vulkan_lifecycle.rs`
- `components/aurea-compositor/src/server/rendering.rs`
- `components/aurea-compositor/src/vulkan/context.rs`
- `components/aurea-compositor/src/vulkan/physical_device.rs`
- `components/aurea-compositor/src/vulkan/rendering_features.rs`
- `components/aurea-compositor/src/vulkan/logical_device_factory.rs`
- `components/aurea-compositor/src/vulkan/logical_device.rs`
- `components/aurea-compositor/src/vulkan/format_capabilities.rs`
- `components/aurea-compositor/src/vulkan/drm_compatibility.rs`
- `components/aurea-compositor/src/drm/scanout_renderer.rs`
- `components/aurea-compositor/src/drm/cpu_presentation_backend.rs`
- `components/aurea-compositor/src/drm/frame_coordinator.rs`
- `docs/architecture/VULKAN-DEVICE-FOUNDATION.md`
- `docs/architecture/HARDWARE-REQUIREMENTS.md`

**Esito fase GPU/Vulkan:** architettura e percorsi principali ricostruiti staticamente; nessuna modifica al codice e nessuna build locale eseguita. Audit di ogni resource pool e prova fisica ancora da completare.


## 13. Audit approfondito — output, monitor e geometrie multi-display (25 novembre 2026)

### 13.1 Identificazione e discovery
- `drm/discovery.rs` enumera connector DRM connessi e ignora connector disconnessi o senza mode.
- `OutputId` è un identificatore runtime composto da DRM device ID e connector ID. È stabile rispetto all'ordine di enumerazione, ma non è un'identità persistente del monitor.
- La scelta della modalità iniziale usa il flag DRM PREFERRED, altrimenti la prima modalità esposta dal connector.
- EDID mancante o non valido non blocca l'output: in quel caso non si crea automaticamente identità fisica persistente.
- L'identità fisica combina EDID validato e percorso sysfs canonicalizzato GPU/connector; senza seriale unitario il dato rimane da sottoporre a revisione manuale.
- `DrmDisplayIdentityRegistry` mantiene evidenza associata agli OutputId correnti e sostituisce gli elementi per device durante reconciliation.

### 13.2 Modello e geometria
- `OutputManager` mantiene registry e layout logico.
- Registrazione batch e configurazione sono transazionali: validano duplicati, numero massimo di output, scala, mode e overflow prima di committare.
- `OutputManager::reconcile_outputs` preserva posizione, scala, trasformazione, stato enabled e mode ancora supportato per gli output mantenuti.
- Nuovi output vengono posizionati a destra dell'output più a destra; non viene applicata automaticamente una disposizione verticale o una topologia preferita.
- I bounds usano coordinate globali logiche, rettangoli half-open, scale intere/frazionarie e dimensioni scambiate per rotazioni di 90/270 gradi.
- `SurfaceOutputTracker` calcola enter/leave delle superfici rispetto agli output e pubblica la scala frazionaria preferita; se una superficie interseca più output sceglie la scala massima tra quelli intersecati.
- La pubblicazione Wayland usa `wl_output` e il protocollo fractional-scale; il manager pubblica mode, transform, scale e posizione.
- Il limite massimo di output attivi è preso dal contratto API condiviso, non duplicato localmente.

### 13.3 Lacuna concreta: configurazione non equivale a modeset
`AureaCompositor::configure_output` applica la configurazione a `OutputManager`, aggiorna il global Wayland e ridisegna. Il commento del metodo dice esplicitamente che il modeset DRM/KMS resta responsabilità del backend attivo. Nell'audit del codice di output e dei backend DRM non è stato trovato un percorso che riceva una richiesta di cambio mode/transform/enable dalla configurazione desktop e la traduca in una nuova configurazione atomica KMS, con TEST_ONLY, commit, rollback e aggiornamento dei frame pool.

Il percorso KMS di presentazione usa `commit_pending` per il modeset iniziale, ma questo non è prova di una procedura generale per modificare la modalità dopo l'avvio.

**Classificazione:** lacuna di integrazione ad alta priorità, non errore del calcolo geometrico. La UI Display Settings non deve esporre controlli mode/enable come se fossero operativi finché la richiesta non attraversa il backend DRM/KMS e la transazione non viene confermata.

### 13.4 Altri limiti da verificare
- Il posizionamento automatico dei nuovi output è solo orizzontale; il layout desiderato dall'utente e la persistenza non sono ancora dimostrati.
- `OutputId` non deve essere usato come chiave di persistenza. Le preferenze per monitor devono usare l'identità fisica con gestione dei casi EDID duplicato/assente, MST, dock e cambio GPU.
- La discovery converte `vrefresh()` in millihertz moltiplicando per 1000: la frequenza può essere arrotondata rispetto al pixel clock/htotal/vtotal reali. Verificare se il valore serve solo per UI o se deve distinguere modalità ad alta precisione (es. 59.94 Hz).
- La riconciliazione dei global Wayland è transazionale sul modello, ma va testato il comportamento reale dei client quando un output viene sostituito e il global precedente rimosso.
- I controlli di overflow e le geometrie negative sono coperti da test unitari; mancano prove multi-monitor fisiche e con hotplug ripetuto.

### 13.5 Registro audit — monitor/output

| ID | Priorità | Elemento | Classificazione | Azione |
|---|---|---|---|---|
| AUD-021 | Alta | Cambio mode/enable/transform aggiorna modello e Wayland ma non è dimostrato un modeset DRM/KMS runtime | Integrazione mancante | Progettare DisplayConfiguration transaction → DRM TEST_ONLY → commit → rollback → publish |
| AUD-022 | Media | I nuovi monitor vengono collocati sempre a destra | Limite UX | Implementare editor disposizione e persistenza layout, mantenendo coordinate globali |
| AUD-023 | Alta | Identità monitor dipende da EDID e sysfs validi; seriale può mancare o duplicarsi | Limite identità | Resolver con revisione manuale, duplicate detection e fallback non ambiguo |
| AUD-024 | Media | Frequenza mode ricavata da `vrefresh()` può perdere precisione | Precisione dati | Valutare refresh millihertz derivato dal timing DRM completo |
| AUD-025 | Alta | Nessun test fisico di hotplug, modeset, rotazione e fractional scale multi-monitor | Non verificato | Matrice 1–4 monitor, connector, dock/MST, risoluzioni e scale |

### 13.6 File di riferimento
- `components/aurea-compositor/src/drm/discovery.rs`
- `components/aurea-compositor/src/drm/outputs.rs`
- `components/aurea-compositor/src/drm/identity.rs`
- `components/aurea-compositor/src/drm/identity_registry.rs`
- `components/aurea-compositor/src/drm/connector_identity.rs`
- `components/aurea-compositor/src/output.rs`
- `components/aurea-compositor/src/output/manager.rs`
- `components/aurea-compositor/src/output/manager/configuration.rs`
- `components/aurea-compositor/src/output/manager/reconciliation.rs`
- `components/aurea-compositor/src/output/manager/geometry.rs`
- `components/aurea-compositor/src/output/coordinates.rs`
- `components/aurea-compositor/src/output/scale.rs`
- `components/aurea-compositor/src/output/wayland.rs`
- `components/aurea-compositor/src/compositor/output_configuration.rs`
- `components/aurea-compositor/src/compositor/output_reconciliation.rs`
- `components/aurea-compositor/src/compositor/surface_output_reconciliation.rs`

**Esito fase output:** discovery, identità transitoria/fisica, registry, geometria, fractional scale e pubblicazione Wayland sono presenti con test unitari. La configurazione grafica non è ancora collegata a un modeset DRM/KMS runtime verificato; layout persistente e prove fisiche multi-monitor restano aperti. Nessuna modifica al codice e nessuna build locale eseguita.

## 13. Audit display: modalità, frequenze e orientamento (25 novembre 2026)

### Requisito prodotto confermato
AureaOS non deve codificare risoluzioni, rapporti d'aspetto o frequenze predefiniti. Deve enumerare le modalità realmente esposte da ogni output DRM/KMS e consentire configurazioni indipendenti per ciascun monitor. Sono inclusi monitor standard, ultrawide, ad alta frequenza e monitor ruotati in verticale (90°/270°), oltre a configurazioni miste.

### Codice già presente e verificato
- `drm/discovery.rs`: enumera i connector DRM connessi, legge le modalità restituite dal kernel, sceglie la modalità marcata PREFERRED come modalità iniziale (fallback: prima modalità disponibile) e prova a leggere EDID. EDID assente o non valido non blocca la scoperta.
- `drm/output.rs`: mantiene separati descrittore logico e handle DRM nativi; converte le modalità nel modello del compositor.
- `output.rs`: conserva l'elenco delle modalità, la modalità corrente, scaling intero/frazionario e trasformazione per singolo output.
- `output/scale.rs`: usa fattori razionali basati su 120, evitando drift floating point nelle conversioni.
- `output/manager/geometry.rs`: scambia larghezza/altezza per trasformazioni a 90°/270° e calcola i limiti logici per output.
- `compositor/output_configuration.rs`: applica modifiche al modello e ripubblica lo stato Wayland, con rollback del modello se fallisce la pubblicazione.

### Lacuna critica confermata: configurazione fisica
`compositor/output_configuration.rs::configure_output` dichiara esplicitamente che aggiorna stato compositor/protocollo ma **non esegue il modeset DRM/KMS**. Quindi oggi una richiesta di cambio risoluzione/frequenza può aggiornare il modello e ciò che viene annunciato ai client senza dimostrare che il monitor sia stato realmente riconfigurato. Non presentare questa API come cambio fisico del display.

### Precisione delle frequenze: verifica necessaria
`drm/discovery.rs` converte `mode.vrefresh()` in millihertz moltiplicando il valore intero per 1000. Questa rappresentazione perde la distinzione tra frequenze vicine/non intere (ad esempio timing nominali 59,94 Hz e 60,00 Hz) se il valore restituito dal binding è intero. Prima di dichiarare supporto fedele a tutte le frequenze, il modello DRM deve conservare il timing nativo completo oppure calcolare millihertz dai parametri del mode DRM con gestione corretta di interlace, doublescan e vscan. Non correggere con una formula semplificata senza test e senza verificare l'API della versione drm nel Cargo.lock.

### Identità delle modalità
`OutputMode` identifica una modalità soltanto con larghezza, altezza e refresh in millihertz. Due timing DRM distinti con gli stessi tre valori possono quindi collassare nella stessa identità logica; `DrmOutputDescriptor::into_display_output` attualmente rifiuta duplicati. Valutare un identificatore di modalità che mantenga il riferimento/indice del timing DRM nativo, senza perdere la validazione del modello logico.

### Orientamento verticale
Il modello supporta `Transform` e la geometria logica considera le trasformazioni che scambiano gli assi. Resta da verificare end-to-end che la trasformazione venga applicata anche al KMS plane/CRTC e al rendering, non soltanto pubblicata in Wayland o riflessa nei limiti logici. I test devono coprire rotazioni 90°/270°, mirror, input/puntatore e layout multi-monitor.

### Piano di correzione (ordine obbligatorio)
1. Separare chiaramente **modalità rilevate**, **configurazione desiderata** e **configurazione effettivamente applicata**.
2. Introdurre un contratto backend DRM/KMS per testare atomicamente mode + connector/CRTC + plane + transform, con test di validazione/test-only quando supportati.
3. Applicare la configurazione fisica prima di pubblicare il nuovo stato ai client; in caso di errore mantenere/ripristinare la configurazione precedente e non mentire ai client.
4. Conservare timing DRM nativo e frequenza precisa, evitando identificazioni basate solo su width/height/refresh arrotondato.
5. Aggiungere test per risoluzioni non standard, ultrawide, 59,94/60 Hz, frequenze elevate, portrait, mixed-DPI, hotplug e rollback.
6. Solo dopo stabilizzare il backend, esporre le scelte nella UI Impostazioni > Schermi.

**Stato:** audit del modello e della scoperta completato a livello sorgente; applicazione KMS reale, precisione completa dei timing e trasformazione fisica end-to-end ancora da implementare/verificare. Nessuna build locale eseguita.


## 14. Audit approfondito — cancellazione touchscreen (25 novembre 2026)

### Problema corretto
Il precedente `handle_touch_cancel` rimuoveva gli slot del dispositivo e inviava un `TouchHandle::up` per ogni contatto. Un rilascio ordinario non equivale a `wl_touch.cancel`: i client potevano interpretare una cancellazione hardware come se l'utente avesse sollevato normalmente ogni dito.

### Vincolo Smithay 0.7
L'API pubblica `TouchHandle::cancel` cancella la sessione touch aggregata del seat e non accetta un filtro per dispositivo o slot. Non è quindi corretto chiamarla per un dispositivo e lasciare attivi nel registro gli slot degli altri touchscreen.

### Politica implementata
- Se il dispositivo che segnala `TouchCancel` non possiede contatti attivi, l'evento viene ignorato: un cancel tardivo non deve interrompere un altro touchscreen.
- Se possiede almeno un contatto attivo, AureaOS cancella la sessione touch dell'intero seat con `TouchHandle::cancel`.
- Contestualmente vengono azzerate tutte le associazioni fisiche → slot Wayland, comprese quelle degli altri dispositivi.
- I successivi eventi motion/up relativi ai vecchi contatti non trovano più uno slot e vengono ignorati. Un nuovo contatto deve iniziare con un nuovo down.
- La rimozione fisica di un dispositivo riutilizza lo stesso percorso, evitando slot orfani.

Questa scelta interrompe intenzionalmente anche i contatti simultanei degli altri touchscreen. È il compromesso coerente con l'API Smithay 0.7 e con il singolo seat Wayland condiviso; non fingiamo di offrire una cancellazione isolata che l'API non consente.

### File e modifiche
- `components/aurea-compositor/src/compositor/touchscreen.rs`: invia il vero evento di cancellazione Smithay.
- `components/aurea-compositor/src/input/pointing/touchscreen_slots.rs`: aggiunge `clear_all` e il test di reset globale.
- `docs/architecture/INPUT-EVENT-COVERAGE.md`: aggiorna semantica e stato della copertura input.
- `docs/development/DEVELOPMENT-LOG.md`: registra la modifica e i limiti di verifica.

### Stato di verifica
**Modifica sorgente e documentazione registrate su main; non ancora validate in compilazione o su hardware.** Non sono stati eseguiti build, Cargo check, rustfmt o test locali. Restano obbligatori nella prossima sessione di verifica raggruppata:
1. test del registro slot, inclusi due dispositivi con lo stesso slot fisico;
2. TouchCancel su un device con contatti simultanei su due touchscreen;
3. cancel tardivo da un device senza slot attivi;
4. rimozione dispositivo durante un contatto;
5. riconnessione e nuovo down dopo la cancellazione;
6. verifica del comportamento dei client Wayland che ricevono `wl_touch.cancel`.


## 15. Navigazione desktop da tastiera — 25 novembre 2026

**Scorciatoie predefinite registrate su `main`:**

| Combinazione | Azione | Percorso |
|---|---|---|
| `Super + Ctrl + Left` | Workspace precedente | Compositor `switch_workspace` |
| `Super + Ctrl + Right` | Workspace successivo | Compositor `switch_workspace` |
| `Super + Ctrl + Up` | Overview / Panorama | DesktopInteraction → Shell panel state |
| `Super + Ctrl + Down` | App Exposé | DesktopInteraction → Shell panel state |

La risoluzione richiede esattamente Super e Ctrl; Alt o Shift aggiuntivi fanno rifiutare la combinazione. Le sequenze riconosciute vengono consumate dal filtro tastiera, incluse ripetizioni e rilascio del tasto già intercettato. Le gesture del touchpad restano separate e vengono generate solo dai dispositivi che le supportano.

**Stato reale:** il resolver e il dispatch delle quattro scorciatoie sono implementati; il runtime shell riceve Overview e App Exposé e aggiorna `DesktopPanelState`. I test sorgente coprono la risoluzione delle quattro frecce e l'attivazione del pannello App Exposé. Non esiste ancora una UI grafica finale per Overview/App Exposé: il modello di stato non equivale a una superficie visibile.

**Verifica:** modifiche registrate su `main`; nessuna build, Cargo check, rustfmt o esecuzione test effettuata, in accordo con la verifica raggruppata. La CI e la verifica su hardware restano da eseguire.


## 16. Pointing-device lifecycle bookkeeping — 25 November 2026

- `PointerManager` owns the per-device pressed-button sets and pointer/touchpad preference overrides. The disconnect path first synthesizes releases for any still-pressed buttons, then removes the device's settings and remaining state.
- Outstanding button codes are now sorted before synthetic release delivery, making disconnect behavior deterministic when multiple buttons were held.
- Unit tests were added for button-state isolation across devices, stable release order, individual button release, per-device preference isolation/cleanup, and rejecting touchpad settings for a mouse.
- These are source-level tests only. They have not yet been compiled or executed; grouped validation and physical-device testing remain pending.


## 17. Device hot-unplug ordering and identity — 25 November 2026

- `DeviceRemoved` now clears gesture ownership and all compositor-owned state (touch cancellation, held pointer-button releases, interaction engine state, output routing, calibration, slots and per-device preferences) before deleting the device from the pointing-device inventory.
- The backend then removes the applied-touchpad-settings cache and retained libinput handle, followed by touchscreen-capability reconciliation.
- `InputDeviceId` is a deterministic hash of the current kernel input sysname. It is valid as a runtime/session key only; `/dev/input/eventN` numbering is not a persistent hardware identity and must not key durable user profiles.
- Source tests cover ID determinism for equal sysnames and separation for the selected test sysnames. PointerManager tests cover held-button and preference isolation/cleanup. The 64-bit hash is not collision-proof.
- No build, Cargo check, rustfmt, test execution or physical hot-unplug/reconnect test has been run.


## 18. PointerManager lifecycle boundary and reclassification — 25 November 2026

- Device discovery now enters through `Compositor::register_pointing_device`; disconnect enters through `Compositor::disconnect_pointing_device`. Registry mutation methods are crate-private, preventing server dispatch from bypassing manager policy.
- `PointerManager::connect` reconciles stored overrides whenever a device descriptor is refreshed. Touchpad-only settings are removed when a device is no longer a touchpad; pointer overrides are retained only for mouse/touchpad classes. A stale touchpad apply report is also removed on registry reclassification.
- `disconnect_pointing_device` clears active compositor/protocol state before `PointerManager::disconnect` removes per-device preferences, pressed-button bookkeeping and the inventory record. Server-side libinput handles and applied-setting cache are then removed, followed by touchscreen capability reconciliation.
- Added source-level tests for reclassification and disconnect cleanup. No build, Cargo check, rustfmt, test execution or hardware validation has been performed.

## 19. Unsupported pointing-device re-announcement

The `DeviceAdded` path now handles a descriptor transition from a supported pointing kind to an unsupported kind. If the sysname-derived ID is already registered, the compositor performs the same ordered disconnect used by `DeviceRemoved`, clears cached touchpad handles and applied settings, and recalculates touchscreen seat capability. This prevents stale inventory and runtime state when a device is re-announced with changed capabilities. A same-ID disconnect/reconnect regression test verifies that runtime buttons and preferences start clean. Validation remains pending; no build, test execution, or rustfmt was run.

## 20. Reject stale pointing events after hot-unplug

The libinput dispatch path now checks the pointing-device inventory before forwarding pointer motion, pointer buttons, pointer axes, touchpad gestures, and touchscreen contact events. Events delivered after the corresponding device has been removed are ignored, so stale queue entries cannot recreate per-device state or emit protocol input from a disconnected device. Keyboard and tablet-tool paths are intentionally unaffected because they use separate lifecycle paths. No build, tests, or rustfmt were run.

## 21. Audit approfondito — input, dispositivi e scorciatoie (25 novembre 2026)

### 21.1 Quadro generale

L'input non è più soltanto un puntatore dimostrativo: nel compositor esistono dispatch libinput, inventario dei dispositivi, stato per-device, gestione dei pulsanti, touch slot, gesti, routing touchscreen/tablet e filtro delle scorciatoie. La copertura è però disomogenea e non equivale ancora a un'esperienza desktop completa né a una validazione hardware.

### 21.2 Mouse e pointing-device lifecycle

- `server/input/dispatch.rs` inoltra movimento relativo/assoluto, pulsanti e assi verticali/orizzontali solo se il dispositivo è registrato.
- Il percorso relativo applica le preferenze di accelerazione; il percorso pulsanti conserva lo stato per dispositivo e il disconnect sintetizza i rilasci ancora pendenti in ordine deterministico.
- `PointerManager` centralizza inventario, preferenze, stato dei pulsanti e cleanup. La riconciliazione di un dispositivo che cambia classificazione elimina le preferenze non più valide.
- Gli eventi pointing/gesture/touch tardivi dopo la rimozione vengono filtrati tramite inventario.
- I Device ID derivano dall'attuale sysname kernel: sono chiavi runtime, non identità hardware persistenti. Non usarli per profili durevoli.
- DPI vendor-specifici, macro e controlli gaming non sono esposti in modo universale da libinput e non vanno simulati.

### 21.3 Touchpad e gesti

- Le preferenze libinput applicate quando supportate sono tap-to-click, tap-and-drag, scroll a due dita, natural scrolling e disable-while-typing. Ogni opzione restituisce uno stato Applied/Unsupported/Failed.
- I controlli sono capability-aware; un mouse generico o una tavoletta non deve ricevere impostazioni touchpad.
- Le gesture pinch/swipe continuano a essere inoltrate ai client tramite il protocollo Wayland pointer-gestures; alcune gesture sono inoltre interpretate come azioni desktop.
- Profilo predefinito osservato: swipe a tre dita sinistra/destra per workspace, swipe in alto per Overview, swipe in basso per App Exposé, pinch a quattro dita per Overview e hold a un dito per richiesta menu contestuale.
- Le preferenze per-device attuali sono stato runtime. La persistenza durevole legata a un'identità fisica stabile e una UI completa per sensibilità/gesture non sono dimostrate.
- Overview e App Exposé aggiornano lo stato del pannello nella Shell, ma la superficie grafica finale non è ancora dimostrata.

### 21.4 Touchscreen

- Gli slot fisici libinput vengono tradotti in slot univoci per il seat Wayland, evitando collisioni quando due touchscreen usano lo stesso numero di slot.
- Mapping automatico limitato a un solo output; su più monitor è necessaria un'assegnazione esplicita del dispositivo.
- Sono presenti trasformazione per rotazione output, calibrazione, down/motion/up e gestione di TouchCancel.
- Smithay 0.7 espone cancellazione touch a livello di seat, non per dispositivo. Se il dispositivo che cancella possiede contatti attivi, AureaOS cancella l'intero seat e azzera tutte le mappature; è una scelta esplicita, non una cancellazione isolata.
- I test sorgente coprono allocazione slot, collisioni tra dispositivi, rilascio e reset globale; non sono ancora stati eseguiti.

### 21.5 Pen tablet e tablet pad

- Il percorso tablet-tool gestisce proximity in/out, movimento, tip down/up, pulsanti, pressione, distanza, inclinazione, rotazione, slider e wheel quando gli eventi libinput li espongono.
- Mapping a output, calibrazione e contenimento della penna nei rettangoli reali degli output sono presenti a livello sorgente.
- I dispositivi tablet-pad sono rilevati nell'inventario, ma Smithay 0.7 non espone nel percorso InputEvent esaminato gli eventi dei pad. La presenza del dispositivo non equivale al supporto di ring, strip o tasti pad.
- **Lacuna lifecycle da correggere:** il dispatch dei quattro eventi TabletTool (Proximity, Axis, Tip, Button) non verifica che il dispositivo sia ancora presente nell'inventario prima di invocare il compositor. I rispettivi handler possono creare/recuperare tablet e tool dal seat; un evento in coda dopo DeviceRemoved potrebbe quindi ricreare stato per un dispositivo scollegato. Il filtro già applicato agli eventi pointing/touch deve essere esteso ai tablet-tool, usando un controllo coerente con l'identità del device/tool e senza bloccare eventi validi.
- La rimozione tablet attualmente è avviata in base alle capability ancora visibili su DeviceRemoved; verificare il comportamento quando libinput ha già ritirato metadata e il descriptor tablet non è più ricostruibile.

### 21.6 Tastiera e scorciatoie

- La tastiera Wayland viene inizializzata con XKB; il layout software validato può essere applicato al seat.
- Il filtro intercetta solo le combinazioni globali riconosciute; gli altri tasti sono inoltrati al client con focus. Press, repeat e release di una scorciatoia consumata sono gestiti per non esporre eventi parziali alle applicazioni.
- Attualmente il resolver implementa Super+Ctrl+Left/Right per workspace e Super+Ctrl+Up/Down per Overview/App Exposé.
- Super da solo per Launcher, Alt+Tab, Alt+F4, Super+E/I/S e le altre combinazioni della policy Windows restano pianificate e non devono essere dichiarate operative.
- Le azioni Overview/App Exposé raggiungono il modello Shell, ma la UI visibile è ancora da completare.
- Non esiste ancora una UI utente per personalizzare le scorciatoie.

### 21.7 Punti da verificare prima di dichiarare input completo

1. Verificare la semantica Smithay di `TouchHandle::motion`: `handle_touch_motion` passa nuovamente il risultato di `touchscreen_focus(location)`, mentre il commento dichiara che il contatto conserva il focus acquisito al down. Accertare se Smithay ignora il focus successivo per uno slot già attivo oppure se il codice può cambiare destinatario durante il contatto; correggere solo dopo aver verificato l'API 0.7 e aggiungere un test.
2. Aggiungere guardie lifecycle agli eventi TabletTool per ignorare code tardive dopo hot-unplug e test di regressione.
3. Verificare la rimozione tablet quando le capability del device sono già mutate/ritirate.
4. Verificare il routing absolute-pointer su singolo output e l'assegnazione esplicita multi-output; oggi non va assunto che una tavoletta/touchscreen si mappi automaticamente a più monitor.
5. Verificare persistenza e riapplicazione delle preferenze dopo riavvio, distinguendo i default globali dai profili per dispositivo e senza usare sysname/eventN come identità permanente.
6. Testare press/release, repeat, focus e cambio layout con più tastiere collegate; verificare che la rimozione di una tastiera non lasci tasti/modificatori logicamente premuti.
7. Eseguire test hardware su mouse, touchpad, touchscreen e tablet; i test unitari non dimostrano comportamento reale del firmware/libinput.

### 21.8 Registro audit — input

| ID | Priorità | Elemento | Classificazione | Azione |
|---|---|---|---|---|
| AUD-026 | Alta | TabletTool events non verificano l'inventario dopo DeviceRemoved e possono ricreare stato tablet/tool | Lifecycle/hot-unplug | Filtrare Proximity/Axis/Tip/Button e aggiungere test evento tardivo |
| AUD-027 | Media | Focus touchscreen durante motion va confrontato con la semantica TouchHandle Smithay 0.7 | Da verificare | Ispezionare API e testare contatto che attraversa superfici |
| AUD-028 | Media | Preferenze per-device e gesture non hanno ancora persistenza hardware durevole dimostrata | Integrazione mancante | Definire identità persistente, storage e riapplicazione senza sysname |
| AUD-029 | Alta | Tablet-pad inventariato ma senza eventi pad nel percorso Smithay 0.7 esaminato | Limite API | Non dichiarare ring/strip/pad buttons; valutare API futura o adapter separato |
| AUD-030 | Alta | Touchscreen/absolute pointer su multi-monitor richiede assegnazione esplicita | Funzionalità parziale | Contratto UI/API di assegnazione per-device e test 1–4 output |
| AUD-031 | Alta | Super/Launcher, Alt+Tab e scorciatoie desktop comuni sono ancora pianificate | Funzionalità mancante | Implementare quando Shell/window activation consumer sono pronti |
| AUD-032 | Alta | Overview/App Exposé aggiornano stato Shell ma non hanno superfici grafiche finali dimostrate | Integrazione UI mancante | Collegare stato pannello a superfici e test end-to-end |
| AUD-033 | Media | Multi-keyboard, key state dopo hot-unplug e focus non verificati su hardware | Non verificato | Test con due tastiere, press/release e rimozione durante tasto premuto |

### 21.9 File esaminati

- `components/aurea-compositor/src/server/input/dispatch.rs`
- `components/aurea-compositor/src/compositor/keyboard.rs`
- `components/aurea-compositor/src/compositor/keyboard_shortcuts.rs`
- `components/aurea-compositor/src/compositor/tablet.rs`
- `components/aurea-compositor/src/compositor/touchscreen.rs`
- `components/aurea-compositor/src/input/pointing/manager.rs`
- `components/aurea-compositor/src/input/pointing/touchpad_settings.rs`
- `components/aurea-compositor/src/input/pointing/touchscreen_slots.rs`
- `components/aurea-compositor/src/input/shortcuts/mod.rs`
- `components/aurea-compositor/src/input/shortcuts/action.rs`
- `components/aurea-compositor/src/input/shortcuts/workspace.rs`
- `components/aurea-compositor/src/input/interaction/profile.rs`
- `docs/architecture/INPUT-EVENT-COVERAGE.md`
- `docs/product/KEYBOARD-SHORTCUT-POLICY.md`

**Esito fase input:** mouse, touchpad, gesture, touchscreen, pen-tablet e scorciatoie workspace hanno percorsi sorgente reali e test mirati. È stata individuata una lacuna concreta nel filtro hot-unplug degli eventi tablet-tool; focus touchscreen durante motion richiede verifica della semantica Smithay. Tablet-pad, profili persistenti, shortcut desktop principali e superfici Overview/App Exposé restano incompleti. Nessuna modifica al codice, build, rustfmt o test eseguiti in questa fase.

## 22. Audit approfondito — Shell, finestre e desktop nativo (25 novembre 2026)

### 22.1 Avvio effettivo della sessione

- `aurea-desktop/src/main.rs` avvia la composition root, inizializza diagnostica e carica il layout tastiera.
- `aurea-shell/src/app.rs` crea ShellRuntime, canali display/input/interazioni e il thread proprietario del compositor.
- `aurea-desktop/src/compositor_factory.rs` crea il server Wayland e avvia `WelcomeClientProcess`.
- Il client Welcome è al momento il primo client grafico esplicito avviato dalla factory.
- Non è stata trovata nella factory una composizione/registrazione di Dock, Launcher, pannelli, Quick Settings o notification center come superfici native. Il fatto che esistano i modelli di stato Shell non prova che esista la relativa UI.

### 22.2 Shell: stato e canali

- `DesktopState` contiene DockState, LauncherState, WorkspaceManager, DesktopPanelState, tema, Night Light, stato Cestino e selezione desktop.
- `ShellRuntime` riceve le interazioni semantiche del compositor e aggiorna il modello per workspace, Overview, App Exposé, Context Menu e selezione.
- Gli eventi DesktopItemActivated vengono conservati nella coda semantica ma non avviano file o applicazioni: il consumer Files/Shell non è collegato.
- Il canale touchpad e il controller display sono collegati alla runtime production path. Il costruttore secondario `with_display_color` continua a creare e scartare il receiver touchpad; evitare di usarlo quando si richiede controllo touchpad.
- `run_event_loop` senza receiver esplicito crea un canale interazioni e scarta il sender: è un percorso di test/uso limitato. Il percorso di produzione usa `run_event_loop_with_interactions`; non sostituirlo accidentalmente.
- Il runtime conserva fino a 32 interazioni pendenti, eliminando la più vecchia quando la coda è piena. Il consumer di presentazione deve svuotare la coda e non trattarla come event log durevole.

### 22.3 Dock, Launcher e pannelli

- DockState implementa le policy Intelligent, AlwaysVisible e AutoHide; il comportamento AutoHide considera bordo, puntatore sopra il Dock e reveal da tastiera.
- Al momento `Intelligent` e `AlwaysVisible` restituiscono entrambi sempre visibile: non esiste ancora una policy intelligente basata su finestre/occlusione. Non presentare Intelligent come comportamento distinto finché non viene implementato.
- LauncherState conserva apertura e query; chiusura azzera la query. Il modello non costituisce una superficie, un catalogo applicazioni o un motore di ricerca.
- DesktopPanelState memorizza una sola richiesta attiva tra Overview, AppExpose e ContextMenu. Le superfici visuali non sono ancora collegate.
- Super da solo per aprire Launcher non è implementato nel resolver globale; il pulsante A del Dock non è dimostrato da una UI attiva.

### 22.4 Finestre Wayland e interazione

- WindowManager conserva toplevel XDG, stato mapped, posizione logica, workspace e stacking order.
- Sono presenti registrazione, mapping, cambio workspace, spostamento tra workspace, raise, posizione e richiesta resize con handshake XDG configure.
- Il click primario su un toplevel lo porta in cima e gli assegna keyboard focus; il click sul desktop vuoto passa al sistema di selezione desktop.
- La scena renderizza le superfici toplevel mappate e il puntatore; il renderer Wayland non equivale ancora a un window manager desktop completo.
- `XdgShellHandler::new_popup` ora conserva il `PositionerState`, calcola la geometria iniziale con `PositionerState::get_geometry()` e invia il primo configure; `reposition_request` aggiorna lo stato e risponde con `send_repositioned(token)`. È il minimo lifecycle protocol-level per permettere ai client di completare configure/commit, non ancora un popup manager completo.
- Il posizionamento corrente non applica ancora `constraint_adjustment` rispetto all'area utile dell'output: popup vicino ai bordi può uscire dallo schermo. `grab` resta intenzionalmente non implementato finché non viene introdotto un `PopupManager` per seat con pointer/keyboard grab coerenti. I popup inoltre non sono ancora raccolti nella scena Vulkan né inclusi nell'hit testing.
- Non sono dimostrati controlli grafici di move/resize, maximize, minimize, close, tiling, decorazioni e Alt+Tab. Le API isolate di resize o workspace non provano l'intero flusso interattivo.
- Non è dimostrata una policy completa di activation token/focus-stealing per nuove finestre; il codice evita che il semplice mapping rubi focus a un client già attivo.

### 22.5 Desktop items, Cestino e applicazioni

- Il compositor riceve uno snapshot semantico dei desktop item e può hit-testare File, Folder, Application e Trash.
- La selezione è propagata al modello Shell; il doppio clic genera un intento di attivazione.
- Il percorso Vulkan non disegna ancora le icone native desktop e non consuma il flag visuale di selezione: niente icone visibili o highlight può essere dichiarato completato.
- Il servizio Cestino legge lo stato iniziale e lo passa al compositor; watcher, aggiornamenti live, conferme e UI non sono collegati alla composition root.
- Non esistono ancora crate/applicazioni indipendenti Fides o Quaero nel workspace; l'avvio file/applicazioni dal desktop resta mancante.

### 22.6 Registro audit — Shell e desktop

| ID | Priorità | Elemento | Classificazione | Azione |
|---|---|---|---|---|
| AUD-034 | Critica | Factory avvia compositor e Welcome, ma non Dock/Launcher/Settings/Quick Settings/Notifications come superfici native | Integrazione desktop mancante | Definire primo client shell nativo e lifecycle delle superfici |
| AUD-035 | Alta | XDG popup: configure/reposition iniziali ora gestiti; output constraints, PopupManager/grab, dismiss e rendering/hit-test restano mancanti | Protocollo e presentazione parziali | Integrare PopupManager per seat, constrain su output, scene/render/hit-test e dismiss annidato |
| AUD-036 | Alta | DesktopItemActivated viene accodato ma non consumato; nessun launch service Fides/app | Integrazione servizio mancante | Collegare intent a un servizio applicazioni/file con policy e feedback |
| AUD-037 | Alta | Icone desktop e selezione non sono disegnate nel frame Vulkan | Rendering mancante | Collegare raster assets/cache/geometry al frame pass e test visuale |
| AUD-038 | Media | DockMode::Intelligent equivale attualmente ad AlwaysVisible | Policy incompleta | Definire comportamento intelligente o rinominare/limitare l'opzione |
| AUD-039 | Alta | WindowManager non dimostra move/resize interattivo, maximize/minimize/close e Alt+Tab end-to-end | Gestione finestre parziale | Definire request, hit regions, focus, stacking e azioni testabili |
| AUD-040 | Alta | Cestino ha solo stato iniziale; watcher e aggiornamenti live non sono collegati | Integrazione mancante | Collegare servizio notifiche filesystem → API semantica → Shell/compositor |
| AUD-041 | Media | `ShellRuntime::run_event_loop` crea un receiver interazioni con sender scartato | API d'uso ambiguo | Documentare come test-only o rimuovere/limitare il percorso alternativo |
| AUD-042 | Media | Nessun catalogo applicazioni/ricerca Launcher dimostrato | Servizio mancante | Definire registry desktop entry, ricerca e lancio processi |

### 22.7 File esaminati

- `components/aurea-desktop/src/main.rs`
- `components/aurea-desktop/src/compositor_factory.rs`
- `components/aurea-shell/src/app.rs`
- `components/aurea-shell/src/runtime.rs`
- `components/aurea-shell/src/desktop/mod.rs`
- `components/aurea-shell/src/desktop/dock.rs`
- `components/aurea-shell/src/desktop/launcher.rs`
- `components/aurea-shell/src/desktop/panels.rs`
- `components/aurea-compositor/src/window_manager.rs`
- `components/aurea-compositor/src/compositor/pointer_button.rs`
- `components/aurea-compositor/src/xdg_shell.rs`
- `components/aurea-compositor/src/render.rs`
- `docs/architecture/DESKTOP-RENDERING-AND-MEDIA-PIPELINE.md`

**Esito fase Shell/desktop:** i modelli Shell, il lifecycle del compositor, il rendering delle superfici Wayland, il focus/raise al click e una parte della gestione workspace sono presenti. Il prodotto desktop visibile resta però incompleto: la factory avvia Welcome, non una shell nativa completa; popup XDG sono ignorati; icone e attivazione desktop non arrivano a un servizio applicazioni; le principali operazioni di window management e le superfici Dock/Launcher/pannelli devono ancora essere integrate. Nessuna modifica al codice, build, rustfmt o test eseguiti in questa fase.

| AUD-043 | Alta | Popup XDG tracciati ma non ancora renderizzati/interattivi | Integrazione parziale | Aggiungere placement vincolato all'output, grab coordinati, scena e hit testing |

## 23. Avanzamento — primo lifecycle XDG popup (25 novembre 2026)

### 23.1 Modifica applicata

- `components/aurea-compositor/src/xdg_shell.rs` ora implementa il configure iniziale di un popup: conserva il `PositionerState`, calcola la geometria protocol-level con `get_geometry()` e invia `send_configure()`.
- Le richieste di reposition aggiornano sia `positioner` sia `geometry` nello stato pending e rispondono con `send_repositioned(token)`.
- Entrambi i percorsi richiedono un nuovo frame per gli output.
- Se l'invio del configure iniziale fallisce, viene registrato un warning e il callback termina senza fingere che il popup sia stato configurato.

### 23.2 Limiti mantenuti espliciti

- `PositionerState::get_geometry()` non applica il constraint adjustment. Serve ancora calcolare il target rectangle relativo al parent e usare `get_unconstrained_geometry()` con l'area utile dell'output.
- `grab` resta intenzionalmente non implementato: non va simulato con un pointer grab isolato. Occorre integrare `smithay::desktop::PopupManager`, la gestione dei grab annidati e il ripristino del focus di tastiera/puntatore.
- La scena `RenderScene`, il rendering SHM/Vulkan e l'hit testing continuano a includere soltanto i toplevel; i popup non sono ancora visibili né interattivi nel frame.
- Nessuna build, `cargo check`, rustfmt, test o prova hardware è stata eseguita, in accordo con la regola di accumulare le modifiche prima della verifica su Fedora.

### 23.3 Prossima sequenza tecnica

1. Aggiungere un piccolo modulo popup owner nel compositor che usa `PopupManager` di Smithay 0.7 per registrazione, relazioni parent/child e grab.
2. Calcolare il rettangolo di constraint nel sistema di coordinate del parent a partire dall'output che contiene il parent; gestire output mancanti senza inventare bounds.
3. Includere popup e relativi surface tree nella scena, nell'ordine corretto sopra il parent, con coordinate globali logiche.
4. Includere popup nell'hit testing e nel focus pointer, rispettando i grab e la chiusura protocol-level.
5. Solo dopo questi passaggi dichiarare completo il supporto popup XDG.

## 24. Avanzamento — registry dei popup XDG (25 novembre 2026)

- Aggiunto `smithay::desktop::PopupManager` allo stato del compositor.
- `new_popup` registra il popup come `PopupKind::Xdg`; il commit delle superfici aggiorna il tree parent/child e ripulisce le risorse non più vive.
- La gestione usa l'API di Smithay 0.7 e non introduce dipendenze aggiuntive.
- Questo passaggio non abilita ancora grab, rendering, hit testing o constraint rispetto all'output. La registrazione del tree è prerequisito, non completamento del menu.
- Nessuna build, rustfmt, cargo check o test eseguiti.
