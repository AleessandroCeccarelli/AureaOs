<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Diario di bordo dello sviluppo AureaOS

Questo documento è il registro cronologico dell’attività tecnica di AureaOS. Affianca, senza sostituirli, i documenti di architettura, gli audit di sicurezza, la roadmap e la cronologia Git.

## 18 novembre 2026 — Stato del repository e fondamenta della scala dei display

### Stato del repository

- Il ramo di sviluppo attivo è `main`.
- Il repository non presenta altri branch né Pull Request aperte al momento della verifica.
- Il workspace Rust contiene quattro crate: `aurea-desktop`, `aurea-shell`, `aurea-desktop-api` e `aurea-compositor`.
- Il repository dispone già di documentazione dedicata ad architettura, prodotto, sicurezza, test e interfacce umane. Questa sezione aggiunge una cronologia operativa, senza duplicare quei documenti.
- Durante questa revisione non sono stati rimossi file sorgente o branch: l’ispezione non ha dimostrato che elementi tracciati fossero obsoleti o eliminabili in sicurezza.

### Lavoro tecnico registrato

I commit recenti hanno stabilito le prime fondamenta per coordinate dei display e scala frazionaria:

- Rappresentazione esatta e razionale del fattore di scala e conversioni coordinate con controlli di sicurezza.
- Separazione tra coordinate logiche globali e pixel fisici locali all’uscita.
- Posizionamento logico delle uscite in base alle larghezze corrette per la scala.
- Trasporto del valore razionale della scala nei render target e nella geometria del puntatore.
- Registrazione del global Wayland fractional-scale manager.
- Impostazione conservativa della scala preferita iniziale a 1.0 per i nuovi oggetti fractional-scale.

Commit di riferimento:

- `9796c023` — documentazione delle fondamenta delle coordinate display e dello stato della migrazione.
- `26f47d60` — definizione del contratto di interoperabilità desktop Linux.
- `6bebd8f1` e `d77db70d` — posizionamento logico delle uscite e test della disposizione con scala.
- `8a0df018`, `9cce2778`, `2ae83868`, `d9ff4e36` e `91f2b214` — protocollo fractional-scale e stato del manager.
- `12cddaa8` — correzione dell’accesso allo stato fractional-scale del compositor e della formattazione.

### Verifica

L’esecuzione GitHub Actions [44290834407](https://github.com/AleessandroCeccarelli/AureaOs/actions/runs/44290834407), relativa al commit `12cddaa8fe35dff6d8b5e1eece5c5f70ce267424`, è terminata con successo il 18 novembre 2026.

Tutti e tre i job sono passati:

- Formattazione Rust.
- Compilazione e test del workspace.
- Audit di sicurezza delle dipendenze Rust.

Questa è una prova relativa ai controlli configurati nella CI. Non dimostra un avvio riuscito su hardware fisico, uno scanout DRM/KMS completo, un desktop grafico finito o un’immagine ISO di installazione avviabile.

### Limitazioni note e lavoro aperto

- La scala frazionaria non è integrata end-to-end. Restano da collegare aggiornamenti della scala preferita in base all’uscita, gestione di buffer e viewport delle superfici, hotspot del cursore, clipping, hit-testing e integrazione completa del renderer.
- Uscita DRM/KMS fisica e scanout devono ancora essere verificati su hardware supportato.
- Il flusso di bootstrap Debian e pubblicazione del rootfs richiede ancora una validazione completa end-to-end sulla macchina Fedora di build supportata.
- L’audit di sicurezza del 13 novembre 2026 registra attività di hardening ancora aperte. I relativi rilievi non possono essere considerati chiusi solo perché il job CI di audit delle dipendenze Rust è passato.
- L’avviso RustSec relativo alla dipendenza transitiva `cgmath 0.18.0` resta documentato nell’audit di sicurezza e richiede una soluzione consapevole dello stato upstream. Un job di audit terminato con successo non equivale a un rapporto dipendenze privo di avvisi.

### Prossime attività tecniche

1. Proseguire l’integrazione della scala frazionaria esaminando associazione tra superfici e uscite e API Smithay 0.7 pertinenti prima di modificare il comportamento.
2. Definire e testare una politica coerente di enter/leave delle uscite e scala preferita per toplevel, popup e superfici figlie.
3. Completare l’integrazione di buffer, viewport, cursore, clipping e hit-testing in moduli piccoli e verificabili separatamente.
4. Mantenere esplicita nella documentazione la distinzione tra verifica CI e verifica su hardware fisico.
5. Riprendere i rilievi aperti dell’audit di sicurezza e pianificare la validazione del rootfs/bootstrap su Fedora quando la macchina di build sarà disponibile.

---

## Modello per le nuove registrazioni

Per ogni nuova voce utilizzare questa struttura:

- **Data e ambito**
- **Riferimenti a repository e commit**
- **Modifiche effettivamente registrate**
- **Verifiche eseguite e risultati precisi**
- **Limitazioni note o regressioni**
- **Prossime attività**

Non registrare come completate attività che sono soltanto intenzioni.


## 22 novembre 2026 — Prima generazione ISO e primo avvio reale

### Contesto e decisione operativa

La giornata è stata dedicata alla prima generazione completa dell'immagine Live Debian Trixie e alla sua prova reale tramite Ventoy. L'utente ha confermato che il flusso abituale è copiare il file ISO nella partizione dati Ventoy e selezionarlo dal menu di avvio: non va proposta la preparazione di una nuova chiavetta.

La generazione della ISO ha richiesto numerosi interventi sul layout live-build, sul recupero del workspace e sulla gestione dei lock. L'esito positivo della build non è stato sufficiente a dimostrare che il sistema Live fosse avviabile.

### Interventi e riferimenti Git

Commit pubblicati su `main` nella sessione:
- `47fd5ed7` — sincronizzazione delle modifiche locali del repository.
- `6bfc72c7` — test d'integrazione per verificare che i gesti touchpad raggiungano la shell desktop.
- `ad524e94` — sistemazione della formattazione dei test d'integrazione dei gesti.
- `f38bd6ac` — serializzazione dell'accesso al workspace live-build persistente tramite lock.
- `63c4bed8` — recupero del lock live-build obsoleto durante la ripresa di una build interrotta.

File ISO principali coinvolti:
- `tools/iso/build-live-iso.sh`
- `tools/iso/build-live-iso-podman.sh`
- `tools/iso/hooks/010-build-aureaos.hook.chroot`
- `tools/iso/hooks/020-brand-boot-menu.hook.binary`
- `tools/iso/package-lists/aureaos.list.chroot`

Il workspace live-build era stato inizialmente creato con una struttura annidata non canonica (`config/config`). Lo script è stato aggiornato per migrare in modo prudente il vecchio workspace al layout Debian standard, preservando chroot, marker di fase e cache quando possibile. Il wrapper Podman ora serializza l'accesso al workspace con `flock`; lo script interno controlla e rimuove un lock residuo solo nel percorso di resume e solo se è un file regolare non symlink.

### Esito della generazione

Il log di build fornito dall'utente il 22 novembre riporta:
- recupero del problema `lb chroot_devpts`;
- kernel e initramfs generati;
- esecuzione riuscita dell'hook di personalizzazione GRUB;
- creazione dell'immagine tramite `xorriso`;
- messaggio finale `P: Build completed successfully`.

Artefatti indicati:
- `~/Documenti/AureaOS.iso`
- `~/Documenti/AureaOS.iso.sha256`
- SHA-256: `1d929c0f99b665f1307191d9502f5c094a25452e32dd28615309feb8350c0704`

Controlli eseguiti su Fedora:
- ISO di circa 350 MB.
- `sha256sum -c` ha restituito `AureaOS.iso: OK`.
- `file` ha riconosciuto ISO 9660 con settore di boot DOS/MBR e volume `AUREAOS_BLUE_TEST`.
- Dopo installazione di `xorriso`, il report El Torito ha rilevato GRUB BIOS (`/boot/grub/grub_eltorito`), immagine EFI (`/boot/grub/efi.img`), MBR protettivo e GPT.

Questi controlli provano la creazione e la presenza delle strutture di boot nell'immagine; non provano il corretto avvio del sistema Live.

### Primo avvio tramite Ventoy: problema bloccante

Sul portatile Lenovo, Ventoy ha caricato GRUB e il kernel Debian `6.12.117+deb13-amd64`. Il boot si è poi interrotto prima dell'avvio del desktop.

Messaggi fotografati:
```
init: line 249: can't open '/scripts/live': No such file or directory
Kernel panic - not syncing: Attempted to kill init: exitcode=0x00000200
```

Conclusione verificata: durante l'esecuzione di init manca il percorso `/scripts/live` nell'ambiente initramfs. Il kernel panic è conseguenza del fallimento del processo init. Non è stato dimostrato un problema GPU, Vulkan o Smithay: il sistema non ha ancora raggiunto quella fase. La causa precisa deve essere determinata ispezionando l'initramfs e il contenuto del chroot.

### Rilievi sullo script da correggere

L'ispezione dei file su `main` ha evidenziato un punto importante: `build-live-iso.sh` rimuove automaticamente l'intero `BUILD_ROOT` quando `lb build` termina con codice zero. Questo elimina anche workspace e cache dopo una build riuscita, vanificando il riutilizzo incrementale. La pulizia dovrà diventare esplicita e opzionale, non automatica.

Inoltre il controllo finale attuale verifica essenzialmente che esista una ISO non vuota e che live-build termini con successo. Non controlla che l'initramfs contenga davvero `/scripts/live`, gli hook di live-boot, il kernel previsto o parametri GRUB coerenti. Una build non dovrà più essere dichiarata valida solo in base all'esistenza del file ISO.

### Decisioni e priorità concordate

- Da lunedì 23 novembre il lavoro torna esclusivamente alla costruzione del desktop AureaOS su GitHub.
- Nessuna build ISO e nessun intervento sugli script ISO durante le sessioni desktop, salvo richiesta esplicita.
- Venerdì 27 novembre si riprende la verifica ISO su Fedora, prima ispezionando l'immagine già prodotta e l'initramfs, senza ricompilare alla cieca.
- La prova continua tramite Ventoy, copiando direttamente la ISO nella sua partizione dati.
- Le modifiche restano su `main`; non eseguire build/test Rust remoti. L'utente eseguirà `cargo fmt`, controlli e test su Fedora quando concordato.
- Mantenere modularità rigorosa: un file, una responsabilità; script di build suddivisi in moduli piccoli e coordinatore limitato.

### Prossime attività

**Lavoro desktop (da lunedì):** proseguire lo sviluppo del desktop e delle sue funzionalità, senza riaprire il filone ISO.

**Sessione ISO di venerdì:**
1. Montare la ISO esistente in sola lettura e individuare kernel/initramfs.
2. Ispezionare il contenuto initramfs per `scripts/live`, hook e moduli live-boot.
3. Se necessario, ispezionare chroot/workspace; non presumere che il workspace sia conservato, perché il cleanup attuale lo elimina dopo il successo.
4. Verificare pacchetti `live-boot`, `live-config`, hook initramfs-tools, parametri kernel `boot=live` e voci GRUB.
5. Solo dopo aver identificato la causa, correggere in modo mirato la pipeline.
6. Rendere persistente il workspace dopo il successo e introdurre pulizia esplicita.
7. Aggiungere validazione pre-ISO e post-ISO, inclusa verifica effettiva dell'initramfs, mantenendo log diagnostici completi.

Non registrare il desktop come completato né l'ISO come avviabile: il primo boot reale ha dimostrato il contrario.


## 23 novembre 2026 — Primo nucleo operativo del Desktop

Dopo aver definito l'architettura, è iniziata l'implementazione concreta del desktop. La priorità è costruire prima una superficie desktop reale; Dock, Launcher e lavoro sul puntatore restano temporaneamente esclusi.

### DesktopManager e superficie base

È stato introdotto in `components/aurea-compositor/src/desktop/` un primo sottosistema modulare:
- `manager.rs`: coordina tema e sorgente dello sfondo e produce uno snapshot immutabile per il frame;
- `theme.rs`: modalità Light/Dark con i token desktop AureaOS;
- `background.rs`: background derivato dal tema o colore uniforme personalizzato, con validazione;
- `surface.rs`: descrizione immutabile della superficie desktop;
- `snapshot.rs`: dati desktop congelati per il singolo frame;
- `mod.rs`: esportazioni del modulo.

Il compositor ora possiede il DesktopManager. La scena di rendering riceve lo snapshot e il backend Vulkan utilizza il colore selezionato in entrambe le principali situazioni: scena vuota e scena con superfici, incluse le varianti readback e rilascio delle immagini verso KMS. Il clear pass Vulkan rimane generico e non contiene logica di tema.

Il colore predefinito è il token Light `#edf4ff`; il token Dark è `#0d1320`. È inoltre predisposto un colore uniforme personalizzato, validato e sempre opaco.

### Stato delle verifiche

Sono stati aggiunti test unitari per tema, background e DesktopManager. Non sono stati eseguiti `cargo fmt`, `cargo check`, test o build: la verifica resta affidata a Fedora dell'utente. Il codice è quindi pubblicato ma non ancora dichiarato compilante o funzionante su hardware.

Wallpaper, decodifica immagini, upload texture Vulkan, scaling/crop e persistenza non sono ancora implementati. Il puntatore, Dock e Launcher restano rinviati.

Commit principali: `542f918`, `2826f2d`, `e0fb92e`.


## 23 novembre 2026 — Primo livello del WallpaperManager

È iniziata l'implementazione del wallpaper selezionato, `aureaos-midnight-flow.jpg`, senza limitarsi a documentare l'architettura.

### Codice aggiunto

Creato `components/aurea-compositor/src/desktop/wallpaper/` con moduli separati:
- `asset.rs`: incorpora l'asset JPEG nel binario e verifica i marker SOF e le dimensioni dell'immagine.
- `manager.rs`: possiede la selezione del wallpaper e produce snapshot immutabili.
- `snapshot.rs`: espone l'asset selezionato al singolo frame.
- `mod.rs`: esporta il sottosistema.

Il `DesktopManager` ora possiede il `WallpaperManager`; il `DesktopSnapshot` e la `RenderScene` trasportano il riferimento all'asset. Il fallback colore rimane disponibile quando non è selezionato un wallpaper.

### Stato reale

Questo passaggio incorpora e rende disponibile il JPEG alla scena, ma non lo mostra ancora sul monitor. Non sono ancora presenti:
- decodifica JPEG in RGBA8;
- texture Vulkan dedicata e upload;
- pass di disegno del wallpaper;
- scaling/crop e clipping per monitor.

Il Cargo.lock non è stato alterato e non è stata introdotta una dipendenza di decoding senza poterla risolvere e verificare su Fedora.

### Verifiche

Nessun `cargo fmt`, `cargo check`, test o build è stato eseguito. La verifica resta da fare su Fedora. Nessun lavoro ISO è stato eseguito.

### Prossimo passo

Verificare prima la compilazione su Fedora; poi scegliere una libreria JPEG, aggiornare Cargo.lock, implementare la decodifica isolata e collegare il buffer RGBA al sistema di upload Vulkan già esistente.


## 23 novembre 2026 — Audit del percorso texture Vulkan per il wallpaper

Proseguendo il lavoro sul wallpaper, è stato ispezionato il percorso Vulkan già usato per le superfici Wayland.

Risultati:
- Esiste già `VulkanShmTexture`, che carica pixel RGBA8 in una immagine Vulkan campionabile usando le primitive di upload condivise.
- Esiste una cache GPU per le superfici Wayland, ma non è direttamente adatta al wallpaper: usa ID di oggetti Wayland e revisioni delle superfici. Il wallpaper richiede una cache con ciclo di vita e chiave propri.
- `surface_compositor::render_frame` oggi pulisce il target con il colore desktop. Nel caso di scena vuota ritorna dopo il clear/readback o clear/release. Per visualizzare il wallpaper anche senza finestre, il draw del wallpaper deve avvenire prima di quel ritorno.
- Nelle scene con finestre, il draw del wallpaper deve essere inserito dopo il clear e prima delle superfici applicative; il cursore resta in cima.
- Il RenderTarget fornisce già geometria e scala dell'output: il fit/crop dovrà essere calcolato localmente per ogni monitor.

Non è stato aggiunto un decoder JPEG perché non risulta una dipendenza decoder già disponibile nel Cargo.lock. Non va lasciato Cargo.toml e Cargo.lock incoerenti né va scritto un decoder JPEG proprietario. La scelta della dipendenza e la rigenerazione del lockfile saranno fatte con Cargo su Fedora.

Nessun file Rust modificato in questa fase; nessun test o build eseguito. Il prossimo step è la verifica Fedora del codice wallpaper già pubblicato, poi decoder JPEG e cache/texture dedicate, quindi integrazione del draw pass in entrambe le condizioni: scena vuota e scena con finestre.


## 23 novembre 2026 — Layout per monitor e buffer RGBA8

La fase desktop prosegue con due moduli piccoli e indipendenti, preparatori alla visualizzazione reale del wallpaper.

### Layout wallpaper per output

Aggiunto `desktop/wallpaper/layout.rs`:
- modalità Fill (crop centrato), Fit (immagine intera con margini), Stretch e Center (dimensione nativa);
- rettangolo sorgente separato dal rettangolo destinazione;
- coordinate locali in pixel fisici dell'output;
- Center può oltrepassare i bordi del monitor: il renderer dovrà applicare lo scissor dell'output;
- test unitari per i quattro comportamenti e input a dimensione zero.

### Buffer immagine decodificata

Aggiunto `desktop/wallpaper/pixels.rs`:
- tipo posseduto `WallpaperPixels` per immagini RGBA8;
- verifica lunghezza esatta del buffer;
- controlli su dimensioni nulle e overflow;
- limite massimo di 256 MiB per il buffer decodificato;
- test unitari sui casi validi e non validi.

I due moduli sono esportati dal sottosistema wallpaper.

### Stato effettivo

Questi moduli non visualizzano ancora il JPEG. Il draw Vulkan resta sospeso fino a quando non saranno disponibili decoder JPEG e texture wallpaper campionabile. È intenzionale: il ramo scena vuota non deve essere modificato con una chiamata di disegno priva di una texture valida.

Commit: `7b15482`, `84b3f00`, `e1a1fdb`.

Nessun comando Cargo è stato eseguito: formattazione, compilazione e test saranno verificati su Fedora. Il prossimo blocco è decoder JPEG + upload/cache Vulkan dedicata, quindi draw pass sia con scena vuota sia con finestre.


## 23 novembre 2026 — Decisione: supporto agli sfondi animati e dinamici

È stata accolta la richiesta di aggiungere ad AureaOS una funzione paragonabile agli sfondi animati/dinamici disponibili su macOS, oltre alle modalità classiche di Windows.

Il wallpaper ufficiale già scelto, `aureaos-midnight-flow.jpg`, resta quello predefinito. La nuova funzione amplia il motore, non cambia la scelta grafica.

La specifica completa è in `docs/design/WALLPAPER-SYSTEM.md`.

### Funzioni previste

- video o immagini animate in riproduzione continua;
- raccolte dinamiche che cambiano in base all'orario (alba, giorno, tramonto, notte);
- sfondo indipendente per ciascun monitor, condiviso su tutti o panoramico su più schermi;
- audio disattivato per impostazione predefinita;
- pausa quando lo sfondo è coperto, lo schermo è disabilitato, la sessione è bloccata o il sistema è sospeso;
- limiti di memoria, risoluzione, frame rate e frame in coda;
- fallback statico e preferenza per ridurre il movimento/risparmiare batteria.

### Scelta architetturale

Il decoder animato sarà separato dal render thread. Il compositor Vulkan riceverà frame RGBA con timestamp, senza occuparsi di decodificare video. La cache animata avrà identità e ciclo di vita propri, distinti dalle superfici Wayland.

Non è ancora stato scelto il backend video: GStreamer e FFmpeg sono candidati da valutare per manutenzione, packaging, licenze/codec e supporto hardware/software. Nessuna dipendenza viene aggiunta prima di tale valutazione.

### Ordine di lavoro

Prima si completa il wallpaper statico JPEG con upload e draw Vulkan; poi si aggiungono PNG/BMP e Tile/Span; successivamente si realizza il contratto dei frame animati e si integra il backend video con le politiche di pausa/ripresa e risparmio energetico.

La funzione animata è una decisione di prodotto e una specifica documentata: non è ancora implementata. Nessuna build o test è stato eseguito in questa fase.


## 23 novembre 2026 — Primo codice per il supporto agli sfondi animati

La decisione sugli sfondi animati passa dalla sola specifica a un primo contratto Rust integrato nel sottosistema wallpaper.

### Codice aggiunto

- `animation.rs`: definisce `WallpaperPlaybackPolicy`, ambiente di riproduzione, motivi di pausa, decisione Playing/Paused, frame RGBA con indice e timestamp, errori e trait `WallpaperFrameProvider`.
- `WallpaperManager`: possiede la policy e la trasferisce agli snapshot.
- `WallpaperSnapshot`: trasporta insieme asset selezionato e policy di riproduzione.
- `wallpaper/mod.rs`: esporta le nuove API.

La policy predefinita limita l'animazione a 30 fps, mantiene l'audio disattivato e prevede la pausa quando il wallpaper è coperto, l'output è disabilitato, la sessione è bloccata, il sistema è sospeso o è attivo il risparmio energetico. La preferenza reduced-motion disabilita l'avanzamento dell'animazione. Il limite configurabile è 1–60 fps.

### Cosa non è ancora operativo

Non è ancora presente un decoder GIF o video e non viene ancora disegnato alcun frame animato. Il trait definisce il confine tra decoder e desktop: il provider dovrà lavorare su un worker dedicato, mai sul render thread Vulkan.

Prossimo ordine: test Fedora del codice appena pubblicato; completamento del percorso statico JPEG (decoder, upload, cache e draw Vulkan); quindi provider GIF/video, worker, coda limitata di frame e collegamento alle policy di pausa/ripresa.

Nessun Cargo fmt/check/test o build è stato eseguito in questa fase.


## 23 novembre 2026 — Modello degli elementi desktop e pile visuali opzionali

È stato aggiunto il primo modello dati condiviso per la scrivania, sotto components/aurea-desktop-api/src/desktop/:

- item.rs descrive identificatori stabili, file, cartelle, applicazioni, Cestino e posizioni logiche.
- preferences.rs gestisce la visibilità del Cestino, l'attivazione facoltativa delle pile e le esclusioni per singolo elemento.
- stacks.rs crea gruppi visuali deterministici per categoria o estensione; una pila viene generata solo quando contiene almeno due file idonei.
- manager.rs possiede la raccolta in memoria, rifiuta identificatori duplicati e impedisce la rimozione del Cestino di sistema.
- mod.rs esporta i contratti senza introdurre dipendenze da Smithay, Wayland o Vulkan.

Il desktop predefinito contiene esclusivamente il Cestino. Le pile sono disattivate inizialmente. Il raggruppamento è derivato e non modifica, sposta, rinomina o elimina i file originali.

La specifica architetturale è in docs/architecture/DESKTOP-ITEMS-AND-STACKS.md.

### Verifica e limiti

Sono stati aggiunti test unitari mirati, ma in questa sessione non sono stati eseguiti comandi Cargo. Non si dichiara quindi la compilazione verificata. Persistenza, scansione e monitoraggio del filesystem, integrazione MIME, interfaccia shell, drag-and-drop, menu contestuale, apertura/espansione delle pile e rendering compositor restano da implementare.

La CI precedente (run 45256450916 sul commit 99714264) aveva build/test workspace e audit dipendenze superati, ma il job di formattazione Rust fallito su file wallpaper/Vulkan preesistenti. Le differenze di formattazione individuate sono state corrette nello stesso aggiornamento; la nuova CI dovrà confermare il risultato.


## 23 novembre 2026 — Icone dedicate del Cestino desktop

### Ambito e scelta grafica

Le icone `light-trash.svg` e `dark-trash.svg` già presenti nella famiglia cartelle Fides v5 rappresentano una cartella/posizione Cestino, non un bidone autonomo. Sono state lasciate invariate. È stata creata una famiglia separata di simboli del Cestino desktop, coerente con il linguaggio dimensionale AureaOS: palette blu-grigia, riflessi morbidi, bordi e ombreggiatura.

### Asset sorgente aggiunti

Nella cartella `aurea-human-interfaces/icons/source/system/trash/`:

- `light/trash-empty.svg`
- `light/trash-half.svg`
- `light/trash-full.svg`
- `dark/trash-empty.svg`
- `dark/trash-half.svg`
- `dark/trash-full.svg`

Tutti e sei gli SVG hanno canvas trasparente 512 × 512 e condividono la stessa geometria del bidone. Il tema Light/Dark modifica la palette per mantenere il contrasto; lo stato Vuoto/Metà/Pieno modifica soltanto il contenuto visibile.

È stato aggiunto un README dedicato e aggiornato l'inventario della libreria icone. Lo sprite delle cartelle Fides e le sue icone Cestino esistenti non sono stati modificati.

### Verifiche e limiti

- Il markup SVG dei sei asset è stato creato e registrato come sorgente modificabile.
- In questa modifica non sono stati eseguiti rendering SVG locale, controllo visivo alle diverse dimensioni, integrazione nella shell o selezione runtime dello stato.
- Le icone non implementano comportamento del Cestino, conteggio file, ripristino, eliminazione definitiva o rendering desktop.
- La scelta tra Vuoto/Metà/Pieno dovrà dipendere dallo stato reale del Cestino, indipendentemente dal tema attivo.

### Prossime attività

1. Ispezionare i sei asset alle dimensioni logiche previste e rifinire l'allineamento ottico se necessario.
2. Collegare il tema attivo alla variante Light/Dark.
3. Collegare Vuoto/Metà/Pieno allo stato effettivo del Cestino.
4. Inserire l'elemento Cestino nella scena della shell e collegare separatamente apertura e menu contestuale.

## 23 novembre 2026 — Revisione grafica delle icone del Cestino

### Direzione visiva approvata

Le prime bozze del Cestino autonomo avevano un coperchio e non rendevano abbastanza evidenti gli stati del contenuto. Sono state sostituite con un bidone aperto, senza coperchio, ispirato alla metafora desktop familiare di Windows.

- Vuoto: apertura ben visibile e interno vuoto.
- A metà: cartaccia accartocciata visibile dentro l'apertura.
- Pieno: carta e rifiuti sporgono chiaramente oltre il bordo.
- Le versioni Light e Dark mantengono la stessa geometria e adottano palette adatte al tema.
- Tutti e sei gli SVG hanno sfondo trasparente.

### File aggiornati

Sostituiti tutti e sei i file in `aurea-human-interfaces/icons/source/system/trash/{light,dark}/`. Il README delle icone descrive ora il contratto grafico senza coperchio.

### Verifiche e limiti

Il sorgente SVG è stato aggiornato e registrato nel repository. Non è stato eseguito rendering grafico né controllo visivo alle diverse dimensioni; non si dichiara completata l'integrazione runtime nel desktop. La selezione dello stato dovrà ancora essere collegata al modello effettivo del Cestino.

 
## 23 novembre 2026 — Cestino desktop riprogettato in rete metallica

Su indicazione del fondatore e sulla base della fotografia fornita, il Cestino desktop passa dal bidone stilizzato a un cestino cilindrico realistico in rete metallica.

- Corpo leggermente rastremato, bordo superiore arrotondato e apertura libera senza coperchio.
- Trama metallica incrociata con trasparenza intenzionale: interno e contenuto devono intravedersi attraverso la rete, come nei riferimenti Windows/macOS.
- Riflessi metallici, ombreggiatura laterale e anello inferiore di rinforzo.
- Stessa geometria per Vuoto, A metà e Pieno; i tre stati differiscono per il contenuto.
- Palette Light/Dark distinta, canvas trasparente.

Aggiornati i sei SVG in aurea-human-interfaces/icons/source/system/trash/, il README dedicato e l'inventario generale. Le icone Cestino della famiglia cartelle Fides v5 restano intatte.

**Limiti:** non è stato eseguito un renderer SVG né un controllo visivo alle dimensioni desktop. Non è stata implementata l'integrazione nella shell o la selezione automatica di tema/stato. Nessun comando Cargo è stato eseguito.

**Prossimi passi:** rendering a 24, 32, 48, 64 e 128 px; controllo della leggibilità della rete e della visibilità dei fogli; collegamento al tema e allo stato reale del Cestino; integrazione nella scena desktop.

 
## 23 novembre 2026 — Direzione definitiva del Cestino: materiale satinato traslucido

Il fondatore ha fornito un riferimento visivo più preciso: due cestini bianchi, vuoto e pieno, su uno sfondo desktop. La richiesta corregge la precedente interpretazione in rete metallica: **la rete metallica viene scartata**.

### Specifica approvata
- Contenitore bianco satinato/opalino, con traslucenza morbida.
- Corpo cilindrico leggermente svasato, base arrotondata e bordo superiore sottile.
- Apertura superiore sempre libera, senza coperchio.
- Interno e oggetti visibili attraverso le pareti, ma diffusi e leggermente sfumati dal materiale.
- Nel caso pieno, fogli e piccoli oggetti devono emergere chiaramente dall'apertura.
- Stessa sagoma, proporzioni e illuminazione per Vuoto, A metà e Pieno.
- Light/Dark modifica solo il contrasto del materiale; gli stati modificano solo il contenuto.
- SVG con sfondo trasparente: il wallpaper non appartiene all'icona.

Aggiornati i sei SVG, il README della famiglia e l'inventario delle icone. Le icone cartella Fides v5 restano separate e inalterate.

**Verifica:** non è stato eseguito un renderer SVG né un controllo visivo alle dimensioni reali. La nuova direzione è stata tradotta nei sorgenti vettoriali, ma richiede ancora rendering e rifinitura ottica prima dell'approvazione finale. Nessuna integrazione runtime o selezione automatica dello stato è stata implementata.

**Prossimi passi:** renderizzare i sei SVG su wallpaper chiaro e scuro, verificare che la trasparenza sia percepibile senza perdere leggibilità, rifinire apertura e contenuto sporgente, quindi integrare tema e stato effettivo del Cestino nella shell.


## 23 novembre 2026 — Correzione SVG Cestino

Dopo il controllo del fondatore su GitHub, individuato un errore strutturale XML nei sei SVG del Cestino: un gruppo SVG risultava non chiuso correttamente e il gruppo di clipping era duplicato. GitHub mostrava quindi “Error rendering embedded code — Invalid image source”.

Rigenerati tutti e sei i file Light/Dark × Vuoto/A metà/Pieno con struttura SVG semplificata e gruppi bilanciati. Il contenuto è disposto dietro la parete frontale traslucida; gli elementi che sporgono restano sopra il bordo. Correzione pubblicata nel commit cf515c2.

**Verifica ancora necessaria:** riaprire la Preview GitHub e controllare visivamente i sei file su dispositivo reale. Non dichiarare concluso il controllo grafico finché la Preview non è stata verificata.

 
## 23 novembre 2026 — Integrazione risorse SVG del Cestino nella shell

I sei asset approvati (Light/Dark × Vuoto/A metà/Pieno) sono ora incorporati a compile-time in Aurea Shell tramite `include_bytes!`, senza dipendenze aggiuntive e senza copiare gli SVG in una seconda cartella.

### Nuovi moduli
- `components/aurea-shell/src/desktop/trash.rs`: enum semantico `TrashContentsState` con Vuoto, A metà e Pieno; stato predefinito Vuoto.
- `components/aurea-shell/src/desktop/trash_artwork.rs`: risoluzione esclusiva delle risorse SVG in base a `DesktopTheme` e `TrashContentsState`.
- `desktop/mod.rs`: `DesktopState` ora possiede lo stato visuale del Cestino e fornisce `trash_artwork_svg()` e `set_trash_contents()`.

Aggiunti test unitari per tutti e sei gli abbinamenti, validità UTF-8/struttura radice SVG, distinzione Light/Dark e selezione dello stato Pieno.

### Confini dichiarati
Questa modifica integra gli asset nel binario della shell e nel modello di presentazione, ma **non disegna ancora l'icona sul desktop**: nel repository esaminato non è ancora presente un widget/renderer desktop che consumi questi byte. Non è ancora collegato un servizio filesystem del Cestino; perciò lo stato parte da Vuoto e deve essere aggiornato dal futuro servizio. Soglie di riempimento, monitoraggio, ripristino e cancellazione definitiva restano da implementare.

**Verifica:** nessun comando Cargo, build o test è stato eseguito in questa fase. I test sono stati aggiunti ma restano da eseguire su Fedora nel ciclo concordato.

### Specifica visuale corretta
La rete metallica descritta in una voce storica precedente era un'interpretazione errata e viene superata: la specifica approvata è il cestino bianco satinato/opalino traslucido, aperto, con contenuto visibile attraverso le pareti. I sei SVG correnti, incluso il Pieno più voluminoso approvato dal fondatore, sono i master canonici.


## 23 novembre 2026 — Primo servizio filesystem del Cestino

Aggiunto `components/aurea-desktop/src/trash/` senza nuove dipendenze:
- `metadata.rs`: codifica/decodifica del percorso originale e formato .trashinfo.
- `service.rs`: servizio per il Cestino home in `~/.local/share/Trash`, con spostamento, elenco, ripristino, eliminazione definitiva esplicita, svuotamento e snapshot.
- `mod.rs`: API pubblica del servizio.

Il ripristino non sovrascrive mai una destinazione esistente. Le voci richiedono sia il file in `files` sia il relativo .trashinfo. Il cestino rifiuta percorsi relativi e percorsi già interni al cestino. La soglia visuale Pieno è configurabile e predefinita a 10 elementi.

Aggiunto in Aurea Shell `ShellRuntime::set_trash_contents` come punto di aggiornamento dello stato visuale ricevuto dal servizio.

**Non ancora collegato:** il servizio non è ancora istanziato dal composition root, non c'è watcher filesystem né conferma UI; il renderer Vulkan non disegna ancora l'icona. Implementazione limitata per ora al cestino home: spostamenti cross-filesystem restituiscono errore senza lasciare metadati orfani. Test scritti ma non eseguiti; nessun comando Cargo lanciato.


## 23 novembre 2026 — Geometria logica e hit-testing degli elementi desktop

La ripresa dello sviluppo si concentra sul passaggio dal modello dati a una superficie desktop realmente interattiva. L'ispezione di main ha confermato che DesktopManager conserva gli elementi e le loro posizioni, mentre RenderScene non contiene ancora elementi desktop nativi: al momento raccoglie superfici Wayland, cursore e snapshot dello sfondo.

### Modifica implementata

Aggiunto components/aurea-desktop-api/src/desktop/layout.rs, senza dipendenze esterne:
- DesktopPoint: punto in coordinate logiche globali.
- DesktopRect: area rettangolare con origine in alto a sinistra e dimensioni positive.
- DesktopHitTest: ricerca dell'elemento sotto il puntatore, usando le posizioni logiche già memorizzate.
- Le aree sono inclusive sul bordo superiore/sinistro ed esclusive su quello inferiore/destro.
- In caso di sovrapposizione, l'ultimo elemento nell'ordine visuale fornito prevale.
- Dimensioni nulle non producono hit.

Il modulo è esportato dall'API condivisa. Aggiornato docs/architecture/DESKTOP-ITEMS-AND-STACKS.md per fissare il contratto e mantenere separati coordinate logiche e pixel fisici dei monitor.

### Confini e verifiche

Questo è un primo contratto geometrico condiviso, non ancora il collegamento al puntatore reale: non sono stati modificati compositor, renderer Vulkan o gestione eventi. Non disegna icone e non implementa selezione, drag, doppio clic o attivazione.

Aggiunti test unitari per bordi, dimensioni nulle, sovrapposizione e coordinate logiche. I test non sono stati eseguiti: nessun comando Cargo, build o CI è stato avviato. La verifica resta programmata su Fedora secondo il flusso concordato.

### Prossimo passo

Introdurre uno snapshot desktop posseduto e immutabile, separato dall'attuale DesktopSnapshot Copy, che trasporti gli elementi dal modello condiviso al compositor senza trasferire responsabilità filesystem o dipendenze grafiche nell'API. Poi collegare hit-testing e input alla scena e soltanto dopo affrontare la rasterizzazione SVG e il rendering Vulkan delle icone.


## 24 novembre 2026 — Audit dei moduli Desktop e File Intelligence

### Ambito e riferimenti

- Branch attivo: `main`.
- Documento architetturale inglese: `docs/architecture/FILE-INTELLIGENCE-AND-METADATA.md`.
- Variante italiana: `docs/it/ARCHITETTURA-INTELLIGENZA-FILE-METADATI.md`.
- L'architettura definisce una capacità AureaOS originale, ispirata ai concetti utili dei moderni sistemi desktop, senza replicare database privati, interfacce o implementazioni di Windows, macOS, KDE o GNOME.

### Risultati dell'ispezione del codice esistente

- `aurea-desktop-api::desktop` contiene già contratti indipendenti dalla tecnologia grafica: DesktopItem, DesktopItemId, DesktopItemKind, DesktopPosition, DesktopPreferences, DesktopManager, raggruppamenti visuali e hit-testing.
- Il modello Desktop è attualmente in memoria. Non è ancora presente una persistenza delle posizioni né un servizio che scopra e aggiorni automaticamente i file visualizzati sul Desktop.
- `DesktopItemKind` distingue File, Folder, Application e Trash; i riferimenti a file/cartelle sono URI stringa. Il metodo `file_extension()` ricava l'estensione dal percorso, ma non equivale a identificazione MIME e non deve diventare la fonte autorevole del tipo di file.
- `DesktopItemsSnapshot` trasferisce una copia posseduta degli elementi al compositor. Il percorso Vulkan non disegna ancora le icone desktop native.
- `components/aurea-desktop/src/trash` implementa un servizio per il Cestino home compatibile con il formato FreeDesktop, ma il servizio non è ancora collegato al composition root, a un watcher o alla UI di conferma.
- Il crate `aurea-desktop` è il composition root/servizi condivisi; il workspace attuale non contiene ancora crate applicativi autonomi per Fides o Quaero.
- Non è stato individuato un catalogo generale dei file, un motore di ricerca globale o un servizio di estrazione metadati/miniature già implementato. Le relative responsabilità non devono essere duplicate nei moduli Desktop esistenti.

### Decisioni e direzione

- Aurea File Intelligence sarà il proprietario del catalogo locale condiviso; Fides gestirà operazioni e navigazione file, Quaero l'esperienza di ricerca, Desktop la disposizione degli elementi, Aurea Media l'estrazione limitata e il compositor soltanto rendering/input.
- Nessun file nascosto per-directory in stile Finder `.DS_Store` come formato canonico AureaOS.
- Separazione tra fatti filesystem, metadati nativi dei formati, metadati estratti, metadati scritti dall'utente, stato di presentazione e cache delle miniature.
- SQLite è il candidato iniziale per il catalogo, ma non è ancora una dipendenza approvata: verificare FTS5, binding Rust, MSRV, packaging Debian/Fedora, WAL e recupero.
- Notifiche filesystem come suggerimenti a bassa latenza; scanner e riconciliazione restano la fonte autorevole.
- Nessun demone permanente, fanotify privilegiato, indicizzazione completa del disco, database vettoriale o runtime AI nella prima fase.
- Tutte le nuove specifiche Markdown devono essere pubblicate in inglese e nella corrispondente variante italiana sotto `docs/it/`.

### Verifiche e limiti

- Ispezionati i contratti Desktop API, DesktopManager, DesktopItemsSnapshot, il servizio Trash, il composition root e i documenti architetturali correlati.
- Nessun file sorgente Rust è stato modificato.
- Nessun comando Cargo, formattazione, build, test o CI è stato eseguito.
- L'audit è statico sul repository; non dimostra comportamento runtime né prestazioni su Fedora.

### Prossime attività

1. Verificare la disponibilità di SQLite/FTS5 e i pacchetti Debian 13/Fedora.
2. Confrontare in modo mirato notify e inotify nativo, includendo overflow, limiti watch, filesystem di rete e riconciliazione.
3. Definire identità stabile, provenienza, freschezza e permessi dei metadati.
4. Disegnare i contratti pubblici senza aggiungere ancora dipendenze o duplicare Desktop API.
5. Proseguire con moduli piccoli e documentazione bilingue (inglese/italiano).


## 24 novembre 2026 — Fondamenta della piattaforma AureaOS

### Direzione confermata
- Base tecnologica: Debian 13 Trixie, Btrfs, Rust, Wayland, Smithay e Vulkan.
- Quaero è il punto di accesso conversazionale e di ricerca a livello di sistema, con esperienza simile a ChatGPT integrata tramite Aurea Intelligence.
- Aurea Intelligence gestisce orchestrazione, policy del contesto locale, confini provider e autorizzazioni. ChatGPT è un provider remoto opzionale ed esplicito; ricerca essenziale e funzioni desktop restano locali e utilizzabili offline.
- Aurea Passkey è una capacità di autenticazione di sistema per login/sblocco, autorizzazione privilegi e API documentate per le applicazioni, con FIDO2/CTAP, integrazione PAM e TPM 2.0 opzionale.
- Layout Btrfs, transazioni aggiornamento, snapshot e rollback vanno progettati insieme. Gli snapshot non sono backup; i sottovolumi annidati sono confini per gli snapshot.
- Le credenziali restano fuori dal compositor e dalle normali configurazioni applicative. Servono recupero e fallback; nessun protocollo crittografico proprietario.
- Restano validi i confini di responsabilità e la modularità di File Intelligence.

### Ricerca tecnologica
- Debian 13 fornisce pacchetti SQLite runtime e sviluppo; i metadati Fedora indicano FTS5 abilitato. Sulla build Debian di destinazione serve una verifica esplicita della capacità FTS5.
- notify 9 documenta supporto Linux tramite inotify e licenza CC0, ma richiede attualmente Rust 1.91 come MSRV. I manifest esaminati usano edition 2024 e non dichiarano rust-version. Non aggiungere notify 9 finché la policy MSRV non è definita; valutare anche inotify nativo.
- Le crate Rust passkey hanno ambiti differenti. passkey-rs offre componenti client WebAuthn e autenticatore CTAP2; le librerie FIDO Linux hanno compromessi distinti su trasporti e licenze. Nessuna dipendenza è stata selezionata.
- Gli snapshot Btrfs sono basati su sottovolumi, non attraversano ricorsivamente sottovolumi annidati e non sono backup.

### Repository e verifiche
- Branch: main. HEAD prima di questo aggiornamento documentale: 9332e26cdb936695c7dc85e6dee3f347820eb129.
- Aggiunti documenti architetturali bilingue.
- Nessun sorgente Rust, dipendenza, manifest Cargo o comportamento runtime è stato modificato.
- Nessun Cargo fmt/check/test/build o CI è stato eseguito.

### Prossime attività
1. Progettare layout sottovolumi Btrfs e modello rollback.
2. Definire threat model Aurea Passkey e confini broker di sistema/PAM.
3. Definire provider Aurea Intelligence e contratti di autorizzazione azioni.
4. Completare verifiche SQLite FTS5 e compatibilità watcher prima di aggiungere dipendenze.


## 24 novembre 2026 — Progettazione Btrfs e recupero
- Aggiunti documenti bilingue di proposta per layout Btrfs e ripristino.
- Proposti sottovolumi fratelli e piatti per rendere espliciti i confini snapshot ed evitare omissioni dovute a sottovolumi annidati.
- Mantenuto /var/lib nell'unità di rollback affinché dpkg e stato servizi restino coerenti con i file installati; proposto di preservare /home e log diagnostici.
- Definiti ciclo di vita obiettivo delle transazioni aggiornamento e requisiti di recupero. Bootloader, cifratura, conservazione, policy cache e swap/ibernazione restano aperti.
- Nessun installer, gestore snapshot, transazione aggiornamento o codice rollback implementato. Nessun comando Cargo o test eseguito.


## 24 novembre 2026 — Decisione architetturale di avvio: systemd-boot

### Decisioni approvate
- systemd-boot è il boot manager principale di AureaOS Roma 1.0.
- UEFI è obbligatorio; BIOS Legacy/CSM non è supportato. Il target iniziale è costituito da PC amd64 moderni, indicativamente degli ultimi dieci anni, subordinatamente alla qualificazione effettiva di firmware e hardware.
- Secure Boot è disabilitato in Roma 1.0. Firma/certificazione Microsoft non è un prerequisito della prima release. Il supporto Secure Boot resta un obiettivo futuro, non una garanzia attuale.
- GRUB 2 resta soltanto una possibile alternativa futura, se emergerà una necessità dimostrata; non viene installato né configurato come boot manager predefinito.
- UKI è la direzione scelta per gli artefatti di avvio, subordinata alla verifica dell'integrazione kernel Debian 13/initramfs.
- L'ESP ospita systemd-boot e le UKI. Il layout resta una ESP più una partizione Btrfs per l'intero sistema installato; non è prevista una partizione XBOOTLDR.
- 1 GiB per l'ESP è soltanto un obiettivo provvisorio. La dimensione finale richiede misurazioni che includano UKI corrente, precedente e Aurea Recovery.

### Ricerca tecnica
- Debian 13 Trixie fornisce systemd-boot 257, gli strumenti systemd-boot e il pacchetto systemd-ukify.
- systemd-boot supporta soltanto UEFI, carica UKI da ESP o XBOOTLDR e implementa il conteggio dei tentativi con comportamento di fallback.
- Il boot manager non può stabilire autonomamente se l'intero desktop/sessione sia sano. AureaOS richiede un segnale esplicito userspace e un ciclo di vita coordinato con gli aggiornamenti.

### Documentazione e ambito
- Aggiunti documenti architetturali systemd-boot/UKI in inglese e italiano.
- Sostituita la precedente proposta speculativa Btrfs basata su sottovolumi/snapshot con la direzione approvata: una sola partizione Btrfs e Aurea Time come unico sistema di backup.
- Aggiornati i riferimenti nelle fondamenta della piattaforma, README, Technology e Vision nelle rispettive versioni.
- Nessun sorgente Rust, manifest pacchetti, installer, configurazione di avvio o comportamento runtime è stato modificato.
- Nessun comando Cargo, build, test o CI è stato eseguito.

### Prossimo lavoro tecnico
1. Verificare nel bootstrap Debian e nei manifest pacchetti eventuali assunzioni sul boot.
2. Prototipare la creazione UKI dai pacchetti kernel Debian e initramfs-tools.
3. Misurare l'occupazione ESP e validare la capacità provvisoria.
4. Specificare segnale di avvio riuscito, conteggio tentativi, ritorno al precedente artefatto e responsabilità di pulizia.
5. Progettare l'immagine di avvio indipendente Aurea Recovery e il percorso tramite supporto esterno.
6. Mantenere Secure Boot fuori dai criteri di accettazione di Roma 1.0; valutare firma e gestione chiavi separatamente per una release futura.

### Proposta Btrfs superata
La precedente bozza del 24 novembre che proponeva sottovolumi fratelli, snapshot, transazioni di aggiornamento basate su snapshot e rollback non era una decisione approvata. È superata. AureaOS usa Btrfs come filesystem scelto, con una sola partizione Btrfs per il sistema installato. Aurea Time è l'unico sistema pianificato per backup/ripristino; gli snapshot Btrfs nativi non fanno parte del flusso previsto.


## 24 novembre 2026 — Predisposizione futura Secure Boot

- Confermato che Roma 1.0 mantiene Secure Boot disabilitato, ma la pipeline UKI deve essere strutturata per un futuro profilo firmato.
- La ricerca ha verificato che Debian 13 Trixie fornisce shim firmato da una Microsoft UEFI CA, helper shim firmati da Debian, systemd-boot firmato da Debian e systemd-ukify per generazione/firma UKI.
- Un futuro percorso di registrazione da parte del proprietario tramite DB firmware o MOK di shim potrebbe evitare che AureaOS debba ottenere un proprio rapporto di firma Microsoft; tuttavia la catena shim → systemd-boot → UKI deve essere dimostrata su firmware reali. Non promettere compatibilità universale.
- Documentate fasi separate di build/firma, separazione chiavi pubbliche/private, custodia protetta delle chiavi di produzione, verifica firme, rotazione/revoca, attenzione a SBAT/DBX e requisiti di ripristino.
- Non sono state generate chiavi; non sono stati modificati stato firmware o manifest pacchetti. Nessun build, test o CI eseguito.
- Aggiornati i documenti architetturali systemd-boot/UKI in inglese e italiano.



## 24 novembre 2026 — Regole Debian-first per le dipendenze

### Decisione

Creato il registro obbligatorio delle dipendenze in `docs/development/DEPENDENCY-REGISTER.md`, con versione italiana in `docs/it/REGISTRO-DIPENDENZE.md`.

Debian GNU/Linux 13 (Trixie), amd64, è esplicitamente la piattaforma target e l'autorità di compatibilità. Fedora resta esclusivamente l'ambiente di sviluppo. Una dipendenza non può essere accettata solo perché comoda o disponibile su Fedora.

### Contenuto del registro

- Definiti gli stati: Adottata, Approvata, Candidata, Condizionata, Respinta e Da riesaminare.
- Resi obbligatori: pacchetto/pacchetto sorgente Debian e versioni esatte, pacchetti Fedora di sviluppo, crate Rust/MSRV, ABI/API e linking, supporto amd64, licenza SPDX, manutenzione upstream, advisory di sicurezza, footprint transitivo/runtime, fonti e validazioni realmente eseguite.
- Registrate le tecnologie architetturali attuali senza dichiarare completata l'integrazione dei pacchetti o la validazione hardware.
- Aggiunte schede specifiche per TurboJPEG, librsvg, SQLite, notifiche filesystem, passkey/FIDO2 e criticità RustSec di cgmath transitiva da Smithay.
- TurboJPEG resta Candidata in attesa di verifica API/ABI Debian, binding Rust e pacchetto Fedora. librsvg è Respinta per l'attuale runtime core; gli SVG master restano gli originali canonici e la direzione prevista è la generazione di asset raster.
- SQLite e notifiche filesystem non sono approvati; nessuna dipendenza passkey è stata selezionata.
- Aggiunte checklist obbligatoria e procedura di modifica che richiede l'aggiornamento di entrambe le lingue e dei documenti/diari pertinenti.

### Ambito e limitazioni

È una modifica di governance e documentazione. Non aggiunge, rimuove o aggiorna dipendenze software, non modifica manifest Cargo o Cargo.lock e non dichiara testati tutti i pacchetti elencati. I dati non ancora supportati da evidenze sono indicati esplicitamente come non verificati.

Non sono stati eseguiti Cargo formatting, build, check, test o CI.

### Regola operativa

Prima di proporre o aggiungere una dipendenza, consultare e aggiornare il registro. La compatibilità Debian prevale sulla comodità dell'ambiente di sviluppo. Non dedurre mai la compatibilità Debian da una compilazione Fedora o dalla CI Ubuntu.



## 24 novembre 2026 — Verifica compatibilità decoder JPEG Debian

### Risultati

- Verificati i pacchetti Debian 13 Trixie amd64 `libturbojpeg0` e `libturbojpeg0-dev`, versione `1:2.1.5-4`.
- Verificati i nomi dei pacchetti di sviluppo Fedora `libjpeg-turbo-devel` e `turbojpeg-devel`; i flussi Fedora consultati usano libjpeg-turbo 3.1.x, quindi gli header nativi Fedora non rappresentano il target Debian 2.1.
- Verificato che `turbojpeg-sys` 1.1.1 e 1.2.0 espongono le API TurboJPEG 3.0 `tj3*`. Il crate di alto livello `turbojpeg` attuale dipende da questa famiglia di binding. Non è una corrispondenza valida da presumere con la libreria TurboJPEG 2.1 di Debian 13.
- Valutato `zune-jpeg` 0.5.15 come alternativa candidata: Rust puro, una dipendenza normale dichiarata (`zune-core ^0.5.1`), MSRV 1.75, licenza `MIT OR Apache-2.0 OR Zlib`.

### Aggiornamento registro

- Aggiornate entrambe le versioni linguistiche del registro.
- I binding Rust TurboJPEG non sono accettati per il percorso con libreria di sistema Debian, salvo verifica di un binding 2.1 o definizione di un adapter FFI AureaOS mantenuto esplicitamente.
- `zune-jpeg` è registrato solo come Candidato. Restano aperte la revisione completa delle dipendenze/sicurezza, i limiti e la gestione degli errori.

### Ambito

Nessuna dipendenza è stata aggiunta ai manifest Cargo o ai lockfile. Nessuna implementazione del decoder è stata scritta. Non sono stati eseguiti build, format, check, test o CI.


### Ulteriore verifica decoder — 24 novembre 2026

- Individuato il problema upstream #496: zune-jpeg 0.5.15 può decodificare JPEG baseline validi con colori errati; upstream indica una correzione nella 0.5.16-rc3.
- La 0.5.16-rc3 è una pre-release e non viene accettata come dipendenza di produzione.
- Aggiornati entrambi i registri: zune-jpeg 0.5.15 è esplicitamente bloccato; attendere una release stabile corretta e ripetere la revisione di sicurezza, compatibilità e limiti del decoder.
- Nessuna dipendenza o codice decoder aggiunto. Nessuna build o test eseguiti.



## 24 novembre 2026 — Valutazione adapter nativo libjpeg-turbo

### Risultati della ricerca
- Confermata la disponibilità su Debian 13 Trixie amd64 di `libturbojpeg0` e `libturbojpeg0-dev` versione `1:2.1.5-4`.
- Confermati su Fedora i pacchetti di sviluppo `libjpeg-turbo-devel` e `turbojpeg-devel`; i flussi esaminati riportano la serie 3.1.x.
- Le note di rilascio upstream della serie 3.x descrivono un wrapper di compatibilità TurboJPEG 2.x. È quindi plausibile usare una piccola interfaccia FFI AureaOS basata sull'API legacy comune, senza dipendere dai binding Rust che espongono l'API 3.0 `tj3*`.
- La compatibilità esatta non è ancora dimostrata: occorre verificare simboli, header, ABI e linking contro Debian 2.1 e Fedora 3.1 reali.
- `jpeg-decoder` 0.3.2 è un'alternativa Rust pura, ma upstream la dichiara in maintenance mode; Debian Trixie fornisce 0.3.0.
- `zune-jpeg` 0.5.15 resta bloccato per la regressione cromatica; la correzione è stata individuata soltanto in una pre-release 0.5.16-rc3.

### Decisione provvisoria
- Registrare libjpeg-turbo con adapter FFI AureaOS come Candidato principale, non ancora Approvato.
- Tenere l'FFI unsafe confinato in un modulo piccolo; il resto del progetto deve ricevere soltanto un risultato sicuro `WallpaperPixels`.
- Decodifica su worker limitato, mai nel render thread.
- Prima dell'implementazione verificare ABI/simboli su entrambe le distribuzioni, runtime package nella base Debian, licenza/advisory, gestione errori e limiti memoria.

### Stato e verifiche
- Aggiornati i registri dipendenze EN/IT con la valutazione e i controlli ancora aperti.
- Nessuna dipendenza aggiunta a Cargo.toml o Cargo.lock; nessun codice Rust modificato.
- Nessun pacchetto installato; nessun build, format, check, test o CI eseguito.
- HEAD di partenza della sessione: `66a0b6ce0657a8be35baa169ae1cb6d3fa5420eb` su `main`.

### Prossima azione
Verificare gli header e i simboli TurboJPEG legacy effettivamente forniti dai pacchetti Debian 13 e Fedora, quindi definire il contratto dell'adapter senza ancora collegarlo al renderer Vulkan.


### Preparazione della verifica ABI locale

- Aggiunto `tools/dependencies/jpeg/turbojpeg-abi-probe.c`, un piccolo programma diagnostico C che usa l'API TurboJPEG legacy: inizializzazione, lettura header, decodifica RGBA e rilascio.
- Aggiunto `tools/dependencies/jpeg/README.md` con i comandi di compilazione per Fedora e Debian 13.
- Il probe limita input compresso a 64 MiB, dimensione massima per asse a 16384 pixel e buffer RGBA8 a 256 MiB; rifiuta esplicitamente CMYK/YCCK nel primo perimetro e verifica alpha opaco.
- Il sorgente è stato revisionato staticamente e corretto per inizializzare il colorspace con il valore legacy `TJCS_DEFAULT`.
- Il probe NON è stato compilato né eseguito; non costituisce ancora prova di compatibilità ABI. Nessun pacchetto installato e nessuna dipendenza Rust aggiunta.
- Prossimo passo: quando l'utente torna su Fedora, compilare ed eseguire il probe sul wallpaper incluso; in seguito ripetere la stessa prova in un ambiente Debian 13 Trixie prima di approvare l'adapter.


## 24 novembre 2026 — Decisione architetturale Aurea Media Service

### Decisione
- AureaOS deve possedere un servizio media proprio: API, limiti, validazione, gestione errori, scheduling, dispatch e integrazione.
- Non riscrivere da zero il codec JPEG. Usare inizialmente un backend libjpeg-turbo/TurboJPEG dietro un confine FFI piccolo e privato, se la verifica su Fedora e Debian ha esito positivo.
- Prima fase: crate Rust `aurea-media` in-process, non daemon separato. IPC e helper process verranno valutati quando esisteranno più client o l'esposizione a immagini non fidate lo renderà necessario.
- Il servizio restituisce pixel RGBA8 CPU-owned. Nessun riferimento a Vulkan, texture o scene desktop.
- Decodifica su worker in background, mai nel render thread.
- Specifica: `docs/architecture/AUREA-MEDIA-SERVICE.md`.

### Verifica e correzione probe
- Consultato direttamente l'header TurboJPEG Debian 2.1.5: non definisce `TJCS_DEFAULT` né `TJCS_UNKNOWN` nel vecchio enum colorspace.
- Corretto il probe C inizializzando il parametro di output colorspace a `-1`; la funzione header lo valorizza in caso di successo.
- Confermato che `TJFLAG_LIMITSCANS` è disponibile nell'API 2.1.5.
- Il probe non è stato compilato o eseguito. Nessuna compatibilità ABI è ancora dichiarata.

## 24 novembre 2026 — Direzione prodotto: desktop universale, multimediale e produttivo

### Conclusione
- AureaOS deve servire bambini, anziani, principianti, studenti, creativi, professionisti, amministratori e sviluppatori attraverso un unico desktop coerente.
- Principio: semplice per impostazione predefinita, potente quando serve, accessibile fin dall'inizio.
- Accessibilità, multimedialità e produttività sono requisiti architetturali e di prodotto, non rifiniture successive.
- Non creare edizioni diverse per età o presunte capacità. Usare progressive disclosure, impostazioni comprensibili, controlli leggibili e grandi, navigazione da tastiera, tecnologie assistive, undo e recupero.

### Conclusioni tecnologiche
- Debian stable resta l'autorità di compatibilità; Fedora resta ambiente di sviluppo rapido e primo rilevatore di problemi.
- Confermata la direzione d'integrazione PipeWire + WirePlumber per trasporto/routing e policy audio-video.
- GStreamer 1.x è il candidato framework multimediale per playback, recording, encoding, muxing e streaming. Su Debian Trixie è stata verificata la disponibilità della famiglia 1.26.2. Plugin e codec saranno selezionati per funzione, con verifica licenze e disponibilità.
- XDG Desktop Portal è il confine d'interoperabilità e permessi per Flatpak; AureaOS dovrà sviluppare il proprio backend per le interfacce realmente supportate, senza dipendere da sessioni GNOME/KDE.
- AT-SPI2 è il target d'interoperabilità accessibile; Orca sarà usato per prove reali. AccessKit è candidato come astrazione per widget Rust, non sostituisce AT-SPI2.
- Parley e cosmic-text sono candidati per il testo, non toolkit completi. Il toolkit nativo resta non deciso finché un prototipo non dimostra Wayland-only, Vulkan, accessibilità, IME, testo complesso, touch, scaling e packaging Debian.
- Le tecnologie nuove/acerbe sono benvenute in prototipi isolati, non come dipendenze obbligatorie del boot o del sistema senza prove, benefici misurabili e rollback.

### Confini
- Aurea Media, Aurea Capture, PipeWire, GStreamer e compositor mantengono responsabilità separate.
- Il compositor non deve codificare media, mixare audio, fare muxing o scrivere file; non deve bloccare il render thread.
- Nessun daemon multimediale monolitico nella prima fase.

### Documentazione aggiornata
- Nuovo docs/architecture/UNIVERSAL-DESKTOP-TECHNOLOGY-STRATEGY.md.
- Allineati VISION.md, docs/it/VISIONE.md, TECHNOLOGY.md e docs/it/PRINCIPI-TECNOLOGICI.md.
- Aggiunti collegamenti ai README EN/IT.
- Aggiornati i registri dipendenze EN/IT con le valutazioni e gli stati non approvati.

### Verifiche
- Nessun manifest Cargo, Cargo.lock, sorgente Rust o pacchetto di sistema modificato.
- Nessun build, format, check, test o CI eseguito.

## 24 novembre 2026 — Condivisione tra dispositivi: Aurea Link e Aurea Share

### Direzione prodotto
- Aggiungere la continuità tra dispositivi alla visione: AureaOS deve scambiare file, foto, audio, video, documenti, link e testo con Android, iOS/iPadOS, macOS e altri desktop.
- Separare **Aurea Link** (pairing, discovery peer, trasferimento locale sicuro e ciclo di ricezione) da **Aurea Share** (azione Condividi del sistema e registro destinazioni).
- Distinguere il passaggio locale tra dispositivi dall'invio a una persona tramite Telegram/WhatsApp. AureaOS può aprire un'app installata supportata con contenuti selezionati; l'utente conferma destinatario/azione nell'app. Non automatizzare API private di messaggistica.
- Primo obiettivo: trasferimento locale di file/link/testo, non streaming media live, relay cloud o sincronizzazione messaggi/notifiche.

### Conclusione tecnologica
- LocalSend Protocol v2.x (esaminata la specifica v2.2 corrente) è il primo candidato d'interoperabilità; non inventare un protocollo proprietario per il primo traguardo.
- Preferire HTTPS app-to-app autenticato, approvazione esplicita, verifica integrità e streaming limitato. Discovery non equivale ad autenticazione. Prevedere fallback IP/QR manuale.
- I client LocalSend esistenti sono riferimento multipiattaforma Linux, macOS, Windows, Android e iOS; non sono UI AureaOS né app obbligatoria da includere.
- KDE Connect è ecosistema secondario opzionale. AirDrop/AWDL/OWL resta ricerca sperimentale, senza promesse o requisiti.
- Futura companion mobile: Android Sharesheet e iOS Share Extensions. Companion macOS valutabile più avanti.

### Sicurezza e architettura
- Pairing e consenso espliciti; mittenti sconosciuti richiedono approvazione.
- Mostrare mittente, file, dimensione e destinazione; niente sovrascrittura silenziosa o apertura automatica.
- Validare percorsi/metadati, imporre limiti dimensione/sessione/frequenza/spazio, scrivere temporaneamente, verificare hash e pubblicare atomicamente.
- Nessun downgrade silenzioso a HTTP non autenticato; nessun file server LAN illimitato sempre attivo.
- Iniziare con piccole librerie Rust e listener limitati alle sessioni, non un daemon monolitico.
- Tenere rete/trasferimento fuori da compositor, render thread, decoder Aurea Media e pipeline GStreamer.

### Documentazione
- Creati docs/architecture/AUREA-LINK-SHARE.md e docs/it/ARCHITETTURA-AUREA-LINK-SHARE.md.
- Aggiornati strategia tecnologica universale, registri dipendenze EN/IT, catalogo applicazioni e README.
- Nessuna dipendenza Cargo, manifest, lockfile o sorgente Rust modificati. Nessun build, format, check, test o CI eseguito.

## 24 novembre 2026 — Aurea Share Online: integrazioni dirette con i servizi

- Chiarito l'obiettivo: condividere una foto/PDF/video/documento visualizzato in Fides o in un viewer proprietario verso servizi online senza aprire manualmente l'app di destinazione e riselezionare/allegare il file.
- Percorso distinto dal trasferimento dispositivo-dispositivo Aurea Link. Aurea Share include ora un percorso esplicito di adapter per servizi online.
- Ricercate le API ufficiali: Telegram Bot API rispetto a Telegram API/TDLib; ambito business di WhatsApp Cloud API; pubblicazione Facebook Pages; Instagram Professional; OAuth/accesso e uso commerciale Reddit; upload sul canale autenticato YouTube.
- Decisione: usare API ufficiali solo per azioni/tipi account consentiti; altrimenti share/deep link ufficiale o handoff ad app desktop registrata. Mai endpoint privati, scraping o automazione silenziosa.
- Definiti descrittori provider neutrali (tipi contenuto, azioni, account, scope OAuth, limiti MIME/dimensione, quote, revisione app, vincoli commerciali, conferma).
- Creata architettura integrazioni online EN/IT e collegata a strategia, documenti Aurea Link/Share e README. Aggiornati registri dipendenze.
- Nessun codice Rust/Cargo, dipendenza, registrazione OAuth o build/test modificati. Le approvazioni API restano da verificare prima dell'implementazione.

## 24 novembre 2026 — Modello account Aurea Share

- Chiarito che AureaOS deve servire utenti comuni, bambini, anziani, creator, professionisti e aziende senza presumere una categoria.
- Collegamento account facoltativo e avviato dall'utente; nessun account online obbligatorio durante l'installazione.
- Previsto un gestore account con account multipli quando consentiti dal provider, tipologia, permessi, capacità disponibili, disconnessione e revoca.
- Le operazioni disponibili dipendono da account, scope, revisione provider, regione, contenuto e limiti correnti.
- Il selettore Condividi resta utilizzabile anche senza account collegati, tramite handoff desktop supportati.
- Aggiornate le architetture Aurea Share online in italiano e inglese. Nessun codice o dipendenza aggiunti.


## 24 novembre 2026 — Riduzione allocazioni nel movimento del puntatore

- Verificato il percorso input mouse/touchpad esistente prima di intervenire.
- Rimosso il vettore intermedio delle geometrie monitor dal movimento del puntatore; il calcolo dei limiti ora attraversa gli output abilitati una sola volta.
- La funzione di vincolo non raccoglie più l'iteratore in un vettore temporaneo: restituisce subito la posizione se già dentro un monitor, altrimenti conserva soltanto il punto valido più vicino.
- Aggiunti test per rifiutare coordinate del puntatore non finite. Restano i test esistenti su spazi tra monitor, bordi, origini negative e scale differenti.
- Nessuna modifica intenzionale alla semantica del puntatore, al routing input, al layout monitor, alle dipendenze o all'API pubblica.
- Modifiche pubblicate su main. Non eseguiti Cargo fmt/check/test/build né CI: resta necessaria la verifica locale su Fedora. Nessuna ISO generata.


## 24 novembre 2026 — Validazione metadati JPEG SOF

- Rafforzato il parser dei metadati JPEG dello sfondo incorporato: il segmento Start Of Frame deve dichiarare un numero di componenti diverso da zero e una lunghezza esattamente coerente con la struttura JPEG SOF (8 byte più 3 byte per componente).
- Aggiunti test unitari per un frame minimo valido a un componente, una tabella componenti troncata e un conteggio componenti nullo.
- È soltanto validazione dei metadati: non decodifica i pixel e non approva il backend FFI TurboJPEG. La prova ABI su Fedora e Debian 13 resta obbligatoria prima di aggiungere il decoder Rust.
- Nessuna dipendenza o manifest modificati. Non eseguiti build, format, check, test, CI o ISO.


## 24 novembre 2026 — Prima risorsa texture Vulkan per lo sfondo

- Aggiunta `VulkanWallpaperTexture` in `components/aurea-compositor/src/vulkan/wallpaper_texture.rs`. Consuma il contratto RGBA8 validato `WallpaperPixels`, alloca un'immagine Vulkan ottimizzata e campionabile, carica i pixel tramite lo staging batch già esistente e transita l'immagine da UNDEFINED → TRANSFER_DST → SHADER_READ.
- La texture possiede un'immagine Vulkan condivisa e una image view: la durata della risorsa non dipende da una variabile locale di uno stack frame. La risorsa è esportata dal modulo Vulkan.
- Esportato `WallpaperPixels` dal modulo `desktop` per permettere alla risorsa GPU di consumare il contratto senza collegare il decoder media a Vulkan.
- Questo completa soltanto il primo livello di upload/risorsa GPU. La texture non è ancora memorizzata in cache per identità dello sfondo, inserita nella scena di rendering o disegnata sotto le superfici desktop. Il costruttore attuale attende il completamento del trasferimento: deve essere chiamato da un percorso di preparazione risorse/background, mai dal callback di rendering sensibile alla latenza.
- Nessun decoder JPEG o dipendenza aggiunti. Non eseguiti Cargo fmt/check/test/build né CI; nessuna ISO generata.


## 24 novembre 2026 — Upload Vulkan asincrono dello sfondo

- Rifattorizzato `VulkanWallpaperTexture` per offrire il percorso `submit_pixels`: invia il trasferimento tramite staging e restituisce un `VulkanPendingUpload<Arc<VulkanWallpaperTexture>>` protetto da fence, senza attendere il completamento GPU.
- Mantenuto `from_pixels` come wrapper bloccante per percorsi controllati di preparazione risorse; ora delega all'implementazione asincrona e attende soltanto in quel punto.
- L'upload pendente conserva staging buffer e texture fino al completamento della fence, seguendo il modello di ownership già usato per le superfici SHM. Cache e integrazione nella scena non sono ancora state aggiunte.
- Nessuna dipendenza o modifica ai manifest. Non eseguiti Cargo fmt/check/test/build né CI; nessuna ISO generata.


## 24 novembre 2026 — Cache limitata delle texture Vulkan dello sfondo

- Aggiunta `VulkanWallpaperTextureCache`, identificata da ID dell'asset wallpaper e numero esplicito di revisione, mai dagli ID delle superfici Wayland.
- La cache conserva una texture selezionata pronta e fino a due upload pendenti protetti da fence, con un budget aggregato di 256 MiB per gli upload in corso. Gli upload vengono controllati senza bloccare; soltanto quello completato e corrispondente alla selezione corrente diventa visibile.
- Integrata la cache nel ciclo di vita della `VulkanSurfacePipelineCache` per dispositivo e aggiunto il polling durante la preparazione del frame. Predisposti i punti di ingresso per preparazione, accesso e rimozione, destinati al decoder e al futuro draw pass.
- Il decoder non produce ancora `WallpaperPixels` dall'asset selezionato: per ora la cache è infrastruttura, non contiene texture effettive e lo sfondo non viene ancora disegnato. Restano da completare draw pass e integrazione nella scena.
- Nessuna dipendenza o modifica ai manifest. Non eseguiti Cargo fmt/check/test/build né CI; nessuna ISO generata.


## GEOMETRIA DI DISEGNO WALLPAPER PER OUTPUT — 2026-11-24

- Aggiunto `desktop/wallpaper/draw_geometry.rs` con `WallpaperDrawGeometry` e `WallpaperUvRect`.
- Converte il layout Fill/Fit/Stretch/Center in una destinazione ritagliata sui limiti del singolo output e nel corrispondente intervallo UV normalizzato. Center corregge le UV dopo il clipping, evitando scissor Vulkan con coordinate negative; Fill mantiene il crop centrale della sorgente.
- Aggiunti test unitari per le quattro modalità e per output nullo/non intersezione.
- Aggiunto `WallpaperPixelRect::new` con validazione delle dimensioni e pubblicati i nuovi tipi tramite l'API desktop.
- Commit: `a5b33ccd` (geometria), `93284f76` (costruttore rettangolo), `804f9e67` (export modulo wallpaper), `96c49b75` (export desktop).
- È solo geometria CPU: non è ancora integrata nel draw pass Vulkan e non visualizza il wallpaper. Nessun decoder aggiunto. Non sono stati eseguiti Cargo fmt/check/test/build/CI né ISO; serve la verifica locale su Fedora.


## 24 novembre 2026 — Shader e pipeline Vulkan dedicati al wallpaper

- Aggiunti gli shader separati `wallpaper_texture.vert` e `wallpaper_texture.frag`. Il vertex shader applica un rettangolo UV normalizzato ricevuto tramite quattro push constant float; il fragment shader campiona la texture dello sfondo.
- Aggiornato `build.rs` per monitorare e compilare i due shader nelle build DRM e pubblicare i percorsi SPIR-V generati.
- Aggiunta una cache di pipeline wallpaper per formato, separata dalla pipeline delle superfici. Riutilizza il descriptor layout per texture campionate, ma dichiara un proprio intervallo push constant da quattro float. L'ABI esistente delle superfici (`u32` per la trasformazione) resta invariata.
- È ancora soltanto infrastruttura della pipeline: non viene ancora associato un descriptor né registrato un draw wallpaper. Decoder e popolamento della texture selezionata restano da completare.
- Commit: `e0823ebe` (vertex shader), `fa3d28e4` (fragment shader), `41b24c1c` (build shader), `7a742fe6` (cache pipeline dedicata).
- Non eseguiti Cargo fmt/check/test/build, CI o ISO. Serve la verifica locale su Fedora; Debian 13 resta il target.

 
## 24 novembre 2026 — Push constant Vulkan generiche per draw testurizzati

## 24 novembre 2026 — Codifica UV wallpaper per push constant

## 24 novembre 2026 — Integrazione del draw wallpaper nel frame Vulkan

## 24 novembre 2026 — Protezione identità asset nella cache wallpaper

- Aggiunto `VulkanWallpaperTextureCache::texture_for_asset` e il relativo accesso nella cache del compositor.
- Il compositor confronta ora l'ID dell'asset selezionato nello `DesktopSnapshot` con la chiave della texture GPU pronta prima di creare il binding o disegnare. Una vecchia texture non può quindi comparire dopo un cambio di sfondo mentre il nuovo upload è ancora in attesa.
- Commit: `e5faff5a`, `337b403a`, `4bbde807`, `2c2e0e3e`.
- Decoder e produttore dei pixel ancora assenti. Nessun Cargo fmt/check/test/build, CI o ISO eseguito.

- Integrato in `surface_compositor::render_frame` il consumo della texture wallpaper selezionata e con upload completato.
- Il frame crea il descriptor per lo sfondo, usa la pipeline wallpaper dedicata, calcola la geometria `Fill` in coordinate fisiche locali all'output usando le dimensioni della texture caricata e registra il disegno subito dopo il clear, prima delle superfici SHM e del cursore.
- Lo sfondo pronto viene disegnato anche quando la scena non contiene finestre né cursore; il percorso rapido di sola cancellazione resta attivo se non esiste una texture pronta.
- Quando la selezione wallpaper viene svuotata, viene svuotata anche la cache Vulkan, evitando che resti visibile una vecchia immagine.
- Commit: `30582074` (integrazione nel frame), `a347fde8` (pulizia alla rimozione della selezione).
- Limite importante: non esistono ancora decoder JPEG né produttore che fornisca pixel decodificati a `prepare_wallpaper_texture`. È stato integrato il percorso di consumo Vulkan, non è ancora dimostrata la visualizzazione dello sfondo. Nessun Cargo fmt/check/test/build, CI o ISO eseguito.

- Aggiunto `WallpaperUvRect::to_push_constant_bytes`, che serializza il rettangolo UV in quattro valori `f32` nell'ordine ABI richiesto dallo shader: minimo X, minimo Y, massimo X, massimo Y.
- Aggiunto un test unitario che ricostruisce i quattro valori dai 16 byte e ne verifica l'ordine.
- Commit: `f474ddfd`.
- È solo preparazione CPU del payload: binding descriptor e draw dello sfondo non sono ancora integrati. Test scritto ma non eseguito; nessun Cargo fmt/check/test/build o CI eseguito.

- Esteso `VulkanDrawPass` con `record_textured_region_with_push_constants`, così le pipeline testurizzate dedicate possono fornire il proprio payload senza modificare l'ABI della trasformazione delle superfici.
- Le superfici continuano a usare il valore `u32` da quattro byte. Lo shader wallpaper può usare il proprio intervallo indipendente da 16 byte per il rettangolo UV.
- Aggiunta validazione prima della registrazione dei comandi: payload non vuoto, allineato a quattro byte e massimo 128 byte, oltre a test unitari per il payload wallpaper e le dimensioni non valide.
- Commit: `7b9b4b37` (API draw pass), `d8a31412` (validazione e test).
- È ancora infrastruttura: binding e disegno effettivo dello sfondo non sono integrati. I test sono stati scritti ma non eseguiti. Nessun Cargo fmt/check/test/build, CI o ISO eseguito.


## 24 novembre 2026 — Prima verifica ABI TurboJPEG su GitHub

- Aggiunto `.github/workflows/turbojpeg-abi.yml`: esecuzione isolata in container Debian 13 Trixie e Fedora, installazione dei pacchetti di sviluppo, compilazione con warning trattati come errori, controllo del runtime linkato e decodifica del wallpaper incluso.
- La prima esecuzione ha individuato due problemi reali nel controllo: `TJSAMP_UNKNOWN` non è definito nell'header TurboJPEG 2.1.5 di Debian; inoltre il filtro del nome SONAME nel workflow era sovra-escapato e falliva anche quando il collegamento era corretto.
- Corretto il valore iniziale dell'output subsampling a `-1`, che viene sostituito dalla funzione di lettura header in caso di successo; corretto il filtro SONAME usando una classe di carattere.
- Debian 13 ha installato effettivamente `libturbojpeg0-dev` e `libturbojpeg0` versione `1:2.1.5-4`. Fedora ha installato `libjpeg-turbo-devel` e `turbojpeg-devel` versione `3.1.4.1-2.fc45`; la compilazione Fedora è riuscita nella prima esecuzione e `ldd` ha mostrato `libturbojpeg.so.0`.
- La prima esecuzione complessiva è fallita per i due problemi sopra indicati. La seconda esecuzione, GitHub Actions run `45356390011`, è terminata con successo su entrambi gli ambienti: Debian 13 Trixie e Fedora. In entrambi i job compilazione, controllo SONAME e decodifica sono passati.
- Risultato identico nei due ambienti: `TurboJPEG legacy API: OK`; 3840 × 2160; 33.177.600 byte RGBA8; alpha opaco. Debian ha caricato `libturbojpeg.so.0` da `/lib/x86_64-linux-gnu/`, Fedora da `/lib64/`.
- La prova convalida il percorso API/ABI esercitato dal probe nei container CI, non la totalità degli input JPEG né il funzionamento del futuro wrapper Rust. La verifica sulla Fedora locale dell'utente e la revisione di sicurezza/packaging restano aperte.
- Nessuna dipendenza Cargo, manifest, lockfile o codice di produzione modificato. Nessuna ISO generata. Il decoder JPEG resta candidato e il crate Aurea Media non viene ancora implementato.


- Aggiornati `docs/architecture/AUREA-MEDIA-SERVICE.md` e `docs/development/DEPENDENCY-REGISTER.md` con le versioni effettivamente testate e la distinzione tra prova CI, verifica host e approvazione del decoder.


## 24 novembre 2026 — Primo nucleo Aurea Media

- Creato `components/aurea-media` e aggiunto al workspace Rust. Il servizio è indipendente da compositor, Smithay e Vulkan.
- API pubblica iniziale: `decode`, `ImageFormat`, `DecodeLimits`, `DecodedImage`, `PixelFormat` e `DecodeError`. Per ora il solo formato implementato è JPEG.
- Limiti predefiniti espliciti: input compresso 64 MiB, dimensione massima per asse 16.384 pixel, 64 milioni di pixel e 256 MiB RGBA8. L'allocazione usa aritmetica controllata e `try_reserve_exact`.
- Isolato il confine nativo: shim C Aurea compilato contro `turbojpeg.h`, con FFI Rust privata limitata a due funzioni Aurea. Costanti, handle e dichiarazioni TurboJPEG restano in C. Sono attivi `TJFLAG_STOPONWARNING` e `TJFLAG_LIMITSCANS`; CMYK/YCCK vengono rifiutati.
- Il servizio è sincrono e non crea thread: i consumer dovranno chiamarlo da un worker limitato, mai dal thread di rendering.
- Aggiornati workspace, lockfile e Rust CI per installare gli header TurboJPEG. La pipeline è stata avviata; compilazione e test Rust sono ancora da verificare.
- Nessuna integrazione nel compositor ancora: prima si deve ottenere una CI verde e completare i test con JPEG validi, progressivi, grayscale, troncati e malformati. Nessuna ISO generata.

- Aggiunti test unitari per limiti, input vuoto/eccessivo, dimensioni, pixel e memoria; aggiunti test nativi che decodificano il wallpaper reale, verificano RGBA8/alpha e rifiutano JPEG troncati. La CI Rust è ancora in coda e non considero i test superati fino al risultato del runner.


## Aurea Media — worker di decodifica

- Aggiunto `components/aurea-media/src/worker.rs`, con un singolo thread dedicato alla decodifica.
- La coda accetta al massimo una richiesta in attesa oltre a quella eventualmente in elaborazione.
- `try_submit` non attende spazio nella coda: segnala il back-pressure quando è piena.
- I byte compressi vengono controllati rispetto al limite configurato prima dell'inserimento in coda.
- Ogni richiesta accettata restituisce un ricevitore one-shot per il risultato.
- Il worker non dipende da compositor, Smithay o Vulkan.
- Il compositor non è ancora collegato al worker: il percorso JPEG → WallpaperPixels → cache texture Vulkan resta il prossimo obiettivo. Nessuna visualizzazione dello sfondo viene ancora dichiarata.
- I test del nuovo worker sono stati aggiunti, ma attendono una CI completata sul commit aggiornato.


## Collegamento iniziale del servizio di decodifica wallpaper

- Aggiunta la dipendenza locale `aurea-media` al compositor.
- Creato `desktop/wallpaper/decode_service.rs`: un file dedicato esclusivamente al ciclo di vita della decodifica dello sfondo.
- Il servizio invia i byte compressi al worker limitato, interroga il risultato senza bloccare e convalida il buffer RGBA8 tramite `WallpaperPixels`.
- Ogni risultato valido porta asset ID e revisione; la selezione di un altro sfondo invalida i risultati precedenti.
- Se la coda è momentaneamente piena, la richiesta può essere ritentata senza cambiare revisione.
- Aggiunti test per decodifica, annullamento logico di una selezione e idempotenza della stessa selezione.
- Il servizio non è ancora collegato a `RenderScene` né alla cache texture Vulkan. Questo è il prossimo passaggio, da fare con un handoff piccolo e senza bloccare il thread di rendering.


## Collegamento decoder JPEG alla cache Vulkan

- Collegato `WallpaperDecodeService` alla preparazione delle risorse del backend Vulkan, senza aggiungere responsabilità a `AureaCompositor`.
- Il backend sincronizza la selezione dell'asset, interroga il worker senza attese e trasferisce il buffer RGBA8 completato alla cache texture senza duplicare l'intera immagine.
- La cache usa la coppia asset/revisione e rende disponibile la texture solo dopo il completamento del fence GPU; una selezione precedente non può diventare la texture attiva.
- La saturazione della coda di decodifica è transitoria: il tentativo viene ripetuto nei frame successivi. La rimozione dello sfondo invalida la selezione.
- Il test di decodifica del worker è stato reso meno dipendente dalla schedulazione della CPU: ora attende con intervalli brevi e un limite massimo invece di affidarsi a un numero fisso di `yield_now`.
- La CI del 24 novembre ha evidenziato una precedente instabilità in quel test e un controllo rustfmt già non conforme su numerosi file del workspace. La nuova esecuzione deve confermare il risultato del fix e la compilazione dell'integrazione; non consideriamo ancora verificata la build.

## 24 novembre 2026 — Stabilizzazione del ciclo di decodifica wallpaper

- Spostata la creazione di WallpaperDecodeService nell'inizializzazione della cache persistente delle pipeline Vulkan. Il thread di decodifica non viene più creato pigramente durante il callback di composizione del frame.
- Aggiunto uno stato di fallimento associato alla revisione corrente: un JPEG non valido, pixel non validi o un worker disconnesso non causano un nuovo tentativo a ogni frame. La selezione di un altro asset azzera lo stato e avvia una nuova revisione.
- La saturazione temporanea della coda resta distinta da un errore definitivo: viene ritentata nei frame successivi.
- Aggiunto un test unitario per verificare che una selezione già fallita non venga reinserita nella coda.
- Aggiornata l'architettura Aurea Media in inglese.
- Commit su main: 1c6fe1a0, 70ad0c7c, b8cb721c, 98c66183.
- Nessuna build o batteria di test avviata per queste correzioni: le verifiche saranno raggruppate dopo il completamento del blocco di lavoro. La formattazione Rust resta affidata a Fedora.
- Verifica visiva su Fedora ancora da eseguire; non viene dichiarato il funzionamento grafico verificato.

## 24 novembre 2026 — Ownership dei pixel durante l'upload wallpaper

- Modificato il confine tra decoder e texture Vulkan: `submit_pixels` riceve ora un riferimento a `WallpaperPixels` e copia i byte nel buffer di staging senza consumare in anticipo il buffer CPU.
- La pipeline chiama `take_ready()` soltanto dopo che la cache conferma l'accettazione dell'upload. Se allocazione immagine, staging, setup comandi o submission falliscono, il risultato decodificato resta quindi disponibile nel servizio.
- La cache memorizza la revisione il cui upload è fallito e non ripete l'operazione a ogni frame. Una nuova revisione/asset azzera il blocco; anche la rimozione dello sfondo pulisce lo stato.
- Nessun clone completo del buffer RGBA è stato introdotto: resta soltanto la copia necessaria verso lo staging Vulkan.
- Commit su main: `130975a6`, `3ebe773e`, `d5d69149`.
- La conservazione dei pixel non equivale a un retry automatico: per riprovare la stessa selezione dopo un errore temporaneo servirà una politica esplicita. Nessuna build, test, rustfmt o CI avviata; verifiche raggruppate a fine blocco su Fedora/CI.


## 24 novembre 2026 — Invalidazione immediata della vecchia texture

- Individuato un caso limite: se la preparazione del nuovo wallpaper falliva prima della submission, la cache poteva conservare la vecchia texture come selezione attiva.
- Ora il cambio di chiave asset/revisione invalida subito la texture pronta, prima delle operazioni Vulkan che possono fallire.
- Gli upload già inviati non vengono cancellati né liberati anticipatamente: restano posseduti dalla cache fino al completamento verificato della fence; se appartengono a una revisione non più selezionata, vengono poi scartati.
- La rimozione dello sfondo continua a togliere immediatamente la texture dalla vista senza compromettere la durata delle risorse GPU pendenti.
- Commit: `58e37b8d`.
- Nessuna build, test, rustfmt o CI avviata. La verifica resta raggruppata al termine del blocco.


## 24 novembre 2026 — Gestione fatale di Vulkan DEVICE_LOST

- La perdita del device durante il polling delle fence wallpaper viene ora propagata come errore tipizzato, senza appiattirla in un generico errore texture.
- L'errore scanout conserva un indicatore esplicito di device loss fino al dispatcher di rendering.
- Se il percorso Vulkan diretto segnala device loss, il dispatcher interrompe il rendering dell'output: non tenta il fallback CPU, perché usa lo stesso logical device Vulkan e non può recuperare un dispositivo perso.
- Lo scanout evita inoltre cancel, reacquire e retire del framebuffer quando il completamento GPU è incerto. Queste operazioni potrebbero invocare altro lavoro Vulkan o liberare risorse ancora referenziate dalla GPU.
- È una politica fail-stop, non una ricreazione del device. La ricostruzione richiederà un lifecycle superiore capace di ricreare device, cache, frame pool e binding degli output.
- Commit: `80f7a7b1`, `5dc7503e`, `ea505973`, `caf159cf`, `5048c861`, `5cc97b6c`.
- Nessuna build, test, rustfmt o CI avviata.


## 24 novembre 2026 — Propagazione estesa di Vulkan DEVICE_LOST

- Estesa la gestione al submit dell'upload GPU del wallpaper: se la creazione/submission dell'upload segnala `ERROR_DEVICE_LOST`, la cache memorizza il guasto di sincronizzazione e restituisce un errore tipizzato.
- La pipeline cache conserva il tipo di errore invece di trasformarlo subito in una stringa.
- Composizione, clear/release e readback conservano la classificazione completa di `VulkanQueueSubmitError`.
- Vengono controllati sia l'errore del submit sia quelli di attesa fence e fallback `device_wait_idle`: una perdita segnalata dal secondo controllo non viene nascosta dal primo.
- Il percorso scanout può così interrompersi senza tentare fallback CPU o retirement non sicuro.
- La ricreazione del device non è ancora implementata: la politica rimane fail-stop.
- Commit aggiuntivi: `c5bb65c2`, `b9f11e2e`, `7861acf1`, `d5618002`, `b0b755db`, `53a6525f`, `de00b7fa`, `3a5bcf7d`, `87d69682`, `633ce8b5`, `b0459eaf`, `bb9cba1d`, `1f957360`.


## 24 novembre 2026 — Propagazione device loss nelle texture SHM

- L'audit della catena Vulkan ha rilevato un riferimento obsoleto a `from_vk_result` nel percorso `clear_and_readback`, dopo la rimozione del convertitore. Corretto con `submit_and_wait_with` e la classificazione centralizzata di `VulkanQueueSubmitError`.
- Commit: `76af94e5`.
- Estesa la classificazione fatale anche agli errori di submission e completamento delle upload delle texture SHM Wayland. `ERROR_DEVICE_LOST` non viene più trasformato in un generico errore texture nei percorsi `prepare_surface_textures` e `complete_surface_texture_uploads`.
- Aggiunto un test unitario per verificare che la perdita del device nella cache SHM diventi `VulkanSurfaceCompositionError::DeviceLost`.
- Commit: `8feb7509`, `822e1da3`.
- Nessun nuovo pacchetto o crate; nessuna modifica a Cargo.toml/Cargo.lock.
- I test sono stati aggiunti ma non eseguiti. Nessuna build, rustfmt, cargo check o CI avviata; la verifica resta programmata su Fedora secondo la procedura concordata.
- Prossimo audit: controllare gli altri ingressi Vulkan che possono ancora appiattire `ERROR_DEVICE_LOST`, in particolare la creazione della texture del cursore e i percorsi diagnostici/offscreen. Mantenere la politica fail-stop e non introdurre recovery del device finché non esiste un lifecycle completo di ricreazione.


### Follow-up — errore tipizzato della texture cursore

- Estesa la stessa classificazione al caricamento iniziale della texture del cursore nativo: il livello cache non converte più `VulkanShmTextureError` in `String` prima che il compositor possa riconoscere `ERROR_DEVICE_LOST`.
- Aggiunto un test unitario per la conversione diretta dell'errore SHM fatale; il test della cache resta presente.
- Commit: `c6e2737c`, `3d5fba86`, `d3ff57c4`.
- Nessuna verifica eseguita. Il caricamento iniziale del cursore usa ancora il percorso sincrono `from_snapshot`: valutare una futura conversione a upload asincrono senza introdurre attese nel frame callback.


### Follow-up — compatibilità del convertitore Vulkan legacy

- L'audit ha verificato anche i chiamanti che usano ancora `submit_and_wait()` e ricevono un `vk::Result` invece del tipo completo `VulkanQueueSubmitError`.
- Il convertitore legacy restituiva sempre `wait_error` per `CompletionUncertain`: se la fence riportava `TIMEOUT` e il fallback `device_wait_idle` riportava `ERROR_DEVICE_LOST`, il guasto fatale veniva nascosto.
- Corretto `VulkanQueueSubmitError::result()` per dare precedenza a `ERROR_DEVICE_LOST` in entrambi i campi, mantenendo `wait_error` per gli errori ordinari. Aggiunti due test unitari mirati.
- Commit: `4a5ac3d2`.
- Nessun test/build/format/CI eseguito.

- Follow-up statico: mantenuta la funzione `result()` `const` usando pattern matching invece di confronti tramite `PartialEq`, così da non introdurre chiamate non-const nel percorso di compilazione. Commit: `2acb358a`.


## 24 novembre 2026 — Upload asincrono della texture del cursore

- Eliminato il caricamento sincrono iniziale della freccia nativa nel compositor Vulkan.
- La cache pipeline ora avvia l'upload tramite `VulkanShmTexture::submit_snapshots` e conserva un `VulkanPendingUpload` con staging, command pool, command buffer e fence fino al completamento.
- Ogni frame controlla la fence in modalità non bloccante. La texture viene pubblicata per il rendering soltanto dopo il completamento; durante i frame intermedi il cursore viene temporaneamente omesso, senza bloccare il frame callback e senza campionare una texture incompleta.
- La texture pronta mantiene ownership condivisa tramite `Arc`, coerente con il modello di risorse della cache.
- Gli errori Vulkan restano tipizzati e `ERROR_DEVICE_LOST` continua a raggiungere la classificazione fatale del compositor.
- Nessuna dipendenza aggiunta e nessuna modifica a Cargo.toml/Cargo.lock.
- Commit: `bb8ae90e`, `fd1f8a80`, `0e35bd1f`.
- Nessuna build, rustfmt, cargo check, test o CI eseguiti. Verifica rimandata al blocco concordato su Fedora.


### Follow-up — audit statico del cursore e correzione ownership upload

- La revisione del costruttore di `VulkanSurfacePipelineCache` ha individuato che il nuovo campo `pending_cursor_upload` non veniva inizializzato. Corretto con `None` nel costruttore; senza questa inizializzazione il workspace non avrebbe compilato.
- Commit correttivo: `45757f16`.
- Prima verifica statica del percorso puntatore: il cursore nativo usa una bitmap RGBA8 interna, hotspot logico esplicito, coordinate logiche e trasformazione/scala dell'output; l'input fisico passa da libinput tramite Smithay.
- Le impostazioni touchpad vengono applicate solo quando libinput espone le capacità corrispondenti; le funzioni non supportate vengono riportate come tali, senza forzare driver o permessi aggiuntivi.
- Nessuna nuova dipendenza, nessun accesso root aggiunto e nessuna modifica a Debian base. Nessuna build, test, rustfmt o CI eseguiti.
- Vincolo permanente ribadito: ogni nuova dipendenza e integrazione hardware/codec/servizio deve essere valutata per Debian 13 Trixie, permessi minimi, licenza e disponibilità nei repository prima dell'adozione.
