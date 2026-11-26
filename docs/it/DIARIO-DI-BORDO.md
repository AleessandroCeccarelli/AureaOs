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



### Follow-up — compatibilità Debian degli strumenti shader

- L'audit del percorso di build ha rilevato un messaggio d'errore specifico Fedora in `components/aurea-compositor/build.rs`, non adatto alla piattaforma obiettivo Debian. Reso neutro rispetto alla distribuzione, indicando `glslang-tools` per Debian e `glslang` per Fedora. Commit: `6be56e11`.
- Aggiornato anche `tools/shaders/compile-vulkan-shaders.sh` per mostrare entrambi i comandi di installazione. Commit: `4e30598b`.
- Verificata la disponibilità di `glslang-tools` su Debian 13 Trixie amd64 (versione `15.1.0+1.4.309.0-1`). Lo strumento è classificato build-time e non va aggiunto automaticamente all'immagine runtime.
- Registrate entrambe le lingue le dipendenze native candidate per Smithay/DRM/input. I pacchetti di sviluppo non sono ancora approvati in blocco: i requisiti diretti devono essere confermati dal grafo Cargo e da una futura build Debian.
- Commit registro inglese: `39a20444`; registro italiano: `f0542b28`.
- Nessun pacchetto installato o aggiunto ai manifest runtime. Nessuna build, test, rustfmt o CI eseguiti.


### Follow-up — stato del percorso input puntatore

- Verificato staticamente che gli eventi libinput di movimento relativo, pulsanti, assi di scroll e gesture touchpad vengono inoltrati al compositor tramite Smithay.
- Il movimento aggiorna la posizione globale logica, la limita all'unione degli output attivi, ricalcola la superficie Wayland sotto il puntatore e richiede un nuovo frame su tutti gli output.
- Il pulsante sinistro esegue già focus/raise della finestra colpita; gli eventi vengono poi inoltrati alla superficie Wayland. La semantica di doppio clic e le interazioni con le icone desktop non risultano ancora implementate in questo percorso.
- Il cursore client Wayland e la freccia nativa AureaOS hanno percorsi distinti; il primo usa la superficie client e il relativo hotspot, il secondo la bitmap interna e la geometria output-aware.
- Nessun permesso globale aggiuntivo o driver vendor proprietario è richiesto dal percorso esaminato: la gestione input è mediata da libinput nella sessione libseat.
- Audit statico soltanto; nessuna build/test/format eseguiti. Prossimo passo: verificare lifecycle della posizione/cursor state tra output, hotplug e session pause/resume, poi collegare hit testing e interazioni del desktop senza duplicare la logica Wayland.


## 24 novembre 2026 — Allineamento della pipeline desktop e audit lifecycle puntatore

### Stato reale della pipeline wallpaper

La revisione del codice su `main` ha mostrato che la pipeline è più avanzata di quanto riportasse la precedente versione del documento architetturale:

- `aurea-media` è già un crate separato, con shim C TurboJPEG, API Rust e worker limitato.
- `WallpaperDecodeService` è collegato alla cache Vulkan e gestisce revisioni, risultati obsoleti, errori e back-pressure senza bloccare il frame.
- `VulkanWallpaperTextureCache` mantiene upload asincroni e risorse pendenti fino al completamento delle fence.
- `surface_compositor::render_frame` disegna la texture wallpaper pronta dopo il clear e prima delle superfici Wayland; il cursore resta sopra la scena.
- La geometria Fill viene calcolata per ciascun output. La cache wallpaper è separata dalla cache delle superfici SHM.

Questa è integrazione presente nel sorgente, non una verifica visiva: non abbiamo ancora eseguito la prova sul laptop Fedora e non dichiariamo il wallpaper funzionante su hardware.

### Audit statico output/sessione/puntatore

- La riconciliazione degli output aggiorna il focus del puntatore e richiede nuovi frame.
- Quando viene rimosso un intero dispositivo DRM, i relativi output vengono ritirati e il compositor tenta di ricentrare il puntatore sul primo output ancora attivo.
- Durante `PauseSession`, gli eventi input in coda vengono ignorati; le superfici KMS vengono disabilitate prima del rilascio delle risorse GBM/Vulkan.
- Durante `ActivateSession`, il compositor ripristina GBM, Vulkan, superfici KMS e output differiti; poi ricentra il puntatore su un output attivo.
- Il ricentramento non avviene mentre un client possiede un pointer grab. È una protezione intenzionale per non interrompere un'interazione Wayland. Resta da verificare il passaggio di fine grab se l'output su cui si trovava il puntatore viene rimosso durante il grab.
- La geometria del cursore nativo è calcolata per output e considera scala, fractional scale e trasformazione del target.

L'audit è statico: nessuna prova di hotplug o cambio VT è stata eseguita sul dispositivo fisico.

### Desktop items: punto di partenza già disponibile

- L'API condivisa contiene già `DesktopRect` e `DesktopHitTest`, con coordinate logiche, bordi sinistro/superiore inclusivi, destro/inferiore esclusivi e risoluzione deterministica delle sovrapposizioni.
- `DesktopItemsSnapshot` è già incluso in `RenderScene`.
- Il rendering Vulkan non disegna ancora le icone native e il percorso del click primario non consulta ancora `DesktopHitTest`: oggi il click si occupa del focus/raise delle finestre Wayland.
- La cartella `icons/generated` contiene al momento solo il README; le icone SVG sorgenti restano canoniche.

### Documentazione

- Aggiornato `docs/architecture/DESKTOP-RENDERING-AND-MEDIA-PIPELINE.md` per riflettere il codice realmente presente, separare implementazione e validazione hardware e correggere la sequenza delle attività successive.
- Commit: `33e8ebae`.

### Prossimo blocco

1. Definire una pipeline riproducibile SVG → raster con output versionati e senza dipendenza SVG runtime.
2. Aggiungere asset desktop e cache texture Vulkan dedicate, senza riutilizzare la cache SHM.
3. Disegnare le icone tra wallpaper e superfici Wayland, con clipping e coordinate logiche/output coerenti.
4. Solo dopo collegare hit-testing, selezione, doppio clic e menu contestuale; apertura file e avvio applicazioni restano responsabilità dei servizi Files/Shell.
5. Riprendere l'audit del termine dei pointer grab in caso di hot-unplug.

Nessuna build, test, rustfmt o CI avviati. Nessuna dipendenza runtime aggiunta. Debian 13 Trixie resta la piattaforma obiettivo; Fedora resta l'ambiente di sviluppo.


## 24 novembre 2026 — Prima pipeline riproducibile per le icone Trash

- Aggiunto `tools/assets/render-trash-icons.sh`, uno script Bash indipendente dalla directory corrente che genera gli export PNG delle sei SVG Trash approvate (Light/Dark × Empty/Half/Full).
- Sono previste otto dimensioni quadrate: 16, 24, 32, 48, 64, 96, 128 e 256 pixel; totale 48 file.
- Il renderer è fissato a `resvg 0.45.1`. Lo script rifiuta versioni diverse e verifica presenza file, firma PNG, dimensioni esatte e formato 8-bit RGBA.
- Verificata la disponibilità Debian 13 Trixie del pacchetto `resvg 0.45.1-1` (amd64). Licenza upstream Apache-2.0 OR MIT. Fedora non ha un pacchetto ufficiale confermato; è documentata la possibilità di installare la stessa versione con Cargo, da valutare per costo di compilazione sul laptop.
- `resvg` è esclusivamente un tool host per generare asset statici. Non viene aggiunto al runtime AureaOS, a Cargo.toml o al rootfs.
- Aggiornati README delle icone e registro dipendenze IT/EN. Il tool resta candidato finché gli export non saranno generati e ispezionati visivamente.
- Commit principali: `be04164b` (script), `70409c7d` (validazione PNG), `8c17ffc1`, `cdac2ab4`, `33ed98bd`, `67e2278b` (documentazione).
- Non eseguiti lo script, build, test, rustfmt o CI. Nessun PNG generato e nessuna dipendenza runtime aggiunta.


### Follow-up — riuso del renderer SVG già presente

- La prima bozza della pipeline aveva introdotto `resvg` come tool separato. Prima di considerare la scelta definitiva, l'ispezione di `tools/assets/build_cursor_theme.py` e `cursor_rasterizer.py` ha evidenziato che AureaOS usa già `rsvg-convert` per generare il tema cursore.
- Per evitare un secondo renderer e una nuova procedura di installazione, `render-trash-icons.sh` è stato corretto per riutilizzare `rsvg-convert`.
- Pacchetti verificati: Debian 13 `librsvg2-bin 2.60.0+dfsg-1+deb13u1`; Fedora 45 `librsvg2-tools 2.63.2-2.fc45`. Entrambi forniscono il comando `rsvg-convert`.
- La versione effettiva del renderer viene registrata in `EXPORT-MANIFEST.txt`; il controllo PNG verifica firma, dimensioni quadrate e formato RGBA8.
- La voce resvg è stata sostituita nel registro dipendenze. Nessun nuovo strumento o pacchetto runtime è richiesto oltre al renderer già usato dal flusso cursori.
- La pipeline Trash non è stata ancora eseguita; nessun PNG è stato generato o validato visivamente.
- Commit di correzione: `e0228650`; documentazione aggiornata nei commit `65705341`, `eb28e253`, `4aa8a60e`, `e9819c41`, `bc95da4b`.


## 24 novembre 2026 — Geometria per-output delle icone desktop e pipeline RGBA8

### Geometria delle icone

- Aggiunto `components/aurea-compositor/src/desktop/item_geometry.rs`.
- Le posizioni salvate restano coordinate logiche globali; per ogni output il modulo calcola la destinazione fisica.
- Gestisce origine del monitor, scala intera e fractional scale, trasformazione del display e clipping ai limiti fisici dell'output.
- Prima dimensione di rendering fissata a 64 × 64 pixel logici; il renderer userà varianti raster corrispondenti a 1x, 1.5x, 2x e 3x.
- Gli elementi fuori dal monitor o con geometria non valida vengono saltati individualmente, senza impedire il disegno degli altri.
- Aggiunti quattro test sorgente: monitor secondario con origine globale, icona fuori output, scala 2x e clipping parziale.
- Il modulo è preparatorio: non è ancora collegato al draw Vulkan e non dichiariamo icone visibili sul desktop. Nessun test è stato eseguito.

### Asset statici senza decoder PNG nel runtime

- Aggiunto `tools/assets/png_to_rgba.py`, basato esclusivamente sulla libreria standard Python.
- Il convertitore accetta solo PNG non interlacciati RGBA8 fino a 256 × 256; controlla CRC dei chunk, limiti di decompressione, dimensione esatta dei pixel e filtri PNG 0–4.
- `render-trash-icons.sh` ora genera 24 file RGBA8 raw (64, 96, 128 e 192 px) dalle sei SVG canoniche, usando `rsvg-convert` già previsto dal flusso cursori.
- I file raw sono tightly packed, row-major, senza padding: dimensione esatta = larghezza × altezza × 4. I PNG sono temporanei e rimossi al termine.
- Nessun decoder PNG Rust, crate Cargo o pacchetto runtime aggiunto. Python 3 e rsvg-convert restano strumenti host per la generazione degli asset.
- Il nuovo script non è stato eseguito; i 24 output non sono stati generati o ispezionati visivamente.

### Stato e prossimi passaggi

1. Eseguire lo script sul Fedora host e ispezionare trasparenza, alpha edges, contrasto Light/Dark e allineamento.
2. Implementare cache asset desktop dedicata e upload Vulkan asincrono con limiti espliciti di numero/byte; non riutilizzare la cache SHM.
3. Collegare la geometria per-output al renderer e disegnare le icone tra wallpaper e superfici Wayland.
4. Solo dopo implementare hit-test, selezione e attivazione.

Nessuna build, test, rustfmt o CI avviati. Nessuna dipendenza runtime aggiunta. Debian 13 Trixie resta il target.


## 24 novembre 2026 — Revisione geometria e pulizia del modulo desktop

### Riferimenti
- Branch: `main`.
- Commit `10f7e7ce607118555e09666cab23c33fb0b48de5`: rimozione delle dichiarazioni duplicate del modulo `item_geometry` e dei relativi re-export in `components/aurea-compositor/src/desktop/mod.rs`.

### Revisione tecnica
- Verificata la disponibilità di `ShmBufferSnapshot::from_rgba8` per dati pixel generati dal compositor e di `VulkanShmTexture::submit_snapshots` per upload asincroni con fence.
- Confermato che la cache SHM è legata agli `ObjectId` Wayland e non deve essere riutilizzata per le icone native.
- Confermato che la geometria desktop per-output gestisce coordinate globali, scala intera/frazionaria, trasformazione dell'output e clipping; resta preparatoria e non è ancora collegata al draw Vulkan.
- Il rendering attuale disegna wallpaper, superfici Wayland e cursore. Le icone desktop non sono ancora incluse nel pass.

### Verifiche e limiti
- Ispezione dei sorgenti su `main`; nessuna build, test, rustfmt o CI eseguiti.
- Gli asset RGBA8 non sono ancora stati generati né sottoposti a ispezione visiva su Fedora.
- Non viene dichiarata la compilabilità della modifica in assenza della verifica Cargo concordata.

### Prossimi passi
1. Generare e controllare su Fedora le 24 varianti Trash RGBA8 prima di integrarle come asset runtime.
2. Implementare una cache Vulkan nativa con chiavi asset, budget memoria e upload asincrono, separata dalla cache SHM e dalla cache wallpaper.
3. Collegare le texture pronte alla geometria per-output e inserirle nel pass tra wallpaper e superfici Wayland.
4. Collegare successivamente hit-testing e interazioni desktop, preservando la precedenza del focus Wayland.


## 24 novembre 2026 — Cache Vulkan dedicata alle icone native

### Modifiche pubblicate
- Aggiunto `components/aurea-compositor/src/vulkan/desktop_icon_texture_cache.rs`.
- La cache usa una chiave nativa composta da ID asset approvato, revisione e dimensione raster; non usa `ObjectId` Wayland.
- La cache è separata sia dalla cache SHM delle superfici client sia dalla cache wallpaper.
- Limiti espliciti: massimo 32 texture e 16 MiB residenti; massimo 2 batch di upload e 8 MiB di staging in-flight.
- Upload tramite `VulkanShmTexture::submit_snapshots`, polling delle fence senza attesa nel normale percorso frame e completamento controllato prima di esporre le texture al renderer.
- Validazione della dimensione quadrata rispetto alla chiave, con limite massimo di 256 pixel; gli upload pendenti mantengono le risorse vive fino al completamento.
- Registrato il modulo in `vulkan.rs` ed esposti i metodi di preparazione, polling, completamento e accesso alla texture da `VulkanSurfacePipelineCache`.

### Riferimenti
- `02feb39e` — prima implementazione della cache.
- `80b8cc3f` — deduplicazione delle richieste.
- `4fed642e` — validazione identità/dimensioni e lifetime degli upload pendenti.
- `574fece0` — registrazione del modulo Vulkan.
- `750e9d6d` — collegamento infrastrutturale alla cache pipeline.

### Stato e limiti
- La cache è infrastruttura preparatoria: non è ancora alimentata dagli asset generati e non è collegata al draw pass né alla geometria delle icone.
- Le 24 varianti RGBA8 non sono ancora state generate o ispezionate su Fedora; nessun binario non verificato è stato aggiunto al repository.
- Nessuna build, test, rustfmt o CI eseguiti. La compilabilità non è dichiarata.
- Nessuna dipendenza Cargo o runtime aggiunta.

### Prossimi passi
1. Generare e ispezionare gli asset RGBA8 sul Fedora host.
2. Aggiungere il mapping tra item desktop, tema/stato e asset approvato.
3. Collegare cache, geometria per-output e draw Vulkan, mantenendo l'ordine wallpaper → icone → superfici Wayland → cursore.
4. Solo dopo passare a hit-testing, selezione e attivazione.


## 24 novembre 2026 — Risoluzione degli asset delle icone Desktop

### Modifiche registrate

- Aggiunto `components/aurea-compositor/src/desktop/icon_assets.rs`, con responsabilità limitata alla risoluzione degli asset grafici approvati per gli elementi Desktop.
- Il modello `DesktopItemKind::Trash` viene associato alle sei varianti ufficiali: Light/Dark × Empty/PartiallyFull/Full.
- Sono dichiarate le quattro densità raster già previste dalla pipeline: 64, 96, 128 e 192 pixel per un'icona da 64 pixel logici.
- Ogni variante produce un identificatore stabile (asset ID, revisione del contenuto e dimensione raster) e il percorso relativo del file RGBA8 generato.
- Il descrittore converte i byte RGBA8 in uno `ShmBufferSnapshot` usando `ShmBufferSnapshot::from_rgba8`, che controlla dimensioni e lunghezza esatta dei dati prima dell'upload.
- Il tipo `DesktopIconTextureKey` è stato spostato nel confine Desktop, fuori dal modulo Vulkan. In questo modo la descrizione degli asset non dipende dalla feature opzionale `vulkan`.
- File, cartelle e applicazioni non ricevono ancora un'icona predefinita: i rispettivi asset canonici e i servizi di identificazione non sono ancora stati definiti.

### Limiti espliciti

- Lo stato Empty/PartiallyFull/Full è un input del resolver; non viene dedotto dal nome o dall'URI e non è ancora collegato al servizio Cestino.
- I 24 file RGBA8 non sono stati generati, caricati o ispezionati visivamente su Fedora.
- Il resolver non è ancora collegato al frame renderer; la cache Vulkan resta infrastruttura preparatoria e nessuna icona viene ancora disegnata.
- Nessuna dipendenza Cargo o runtime aggiunta.

### Riferimenti

- `0f06f637` — primo modulo descrittore degli asset Desktop.
- `fbd82001` — identità della texture spostata fuori dalla cache Vulkan.
- `38d5749b` — registrazione del resolver nel modulo Desktop.
- `0d1e135e` — prima esportazione della chiave dal modulo Vulkan, poi rimossa.
- `9c0b9a26`, `6ca60a59`, `f0353135` — correzione architetturale: chiave definita nel modulo Desktop, indipendente dalla feature Vulkan.
- Documentazione architetturale aggiornata nello stesso blocco.

### Verifica

Revisione statica del sorgente e dei contratti esistenti. Nessuna build, test, rustfmt, cargo check o CI eseguiti, secondo la procedura concordata. Non viene dichiarata la compilazione.

### Prossime attività

1. Eseguire su Fedora la generazione dei 24 asset e verificarne dimensioni, trasparenza, bordi alpha e resa su sfondi chiari/scuri.
2. Collegare il servizio Cestino al valore `DesktopTrashContents`, senza inventare uno stato di default.
3. Caricare i byte generati e costruire le richieste per la cache Vulkan.
4. Collegare cache e geometria per-output al draw pass, mantenendo l'ordine wallpaper → icone → superfici Wayland → puntatore.


### Follow-up — Stato Cestino nel contratto Desktop condiviso

- Aggiunto `DesktopTrashState` a `aurea-desktop-api`: Empty, PartiallyFull e Full sono ora un unico tipo semantico condiviso, indipendente da Shell e Vulkan.
- `DesktopManager` possiede lo stato e offre getter/setter espliciti; il valore predefinito è Empty.
- `DesktopItemsSnapshot` copia lo stato insieme agli elementi, così ogni frame riceve un valore immutabile e coerente.
- Aurea Shell riutilizza il tipo condiviso come `TrashContentsState`; il servizio `aurea-desktop::TrashVisualState` può essere convertito esplicitamente nel contratto condiviso.
- Il resolver delle icone ora usa lo stesso `DesktopTrashState`, senza mantenere un secondo enum locale.
- L'accesso al filesystem resta nel servizio Trash. Il compositor non interroga directory, non calcola soglie e non avvia watcher.

Riferimenti: `1807c23d`, `544036fb`, `5c35a382`, `17e7062e`, `5a08ca66`, `6048a1cc`, `5a3c05bf`, `e776dfba`.

**Limite ancora presente:** la conversione è disponibile, ma il composition root non ha ancora collegato automaticamente lo snapshot del servizio al modello Desktop del compositor. Non esiste ancora un watcher o un aggiornamento automatico dello stato.

Nessuna dipendenza aggiunta. Nessuna build, test, rustfmt, cargo check o CI eseguiti.


### Follow-up — Selezione densità raster

- Il resolver sceglie automaticamente la più piccola variante approvata che copre la scala effettiva dell'uscita: 64 px fino a 1x, 96 px fino a 1,5x, 128 px fino a 2x e 192 px oltre 2x.
- Un valore di scala non finito usa prudentemente la variante 1x.
- La scala deve provenire dal `RenderTarget::effective_scale()`; la selezione non si basa soltanto sul vecchio fattore intero dell'output.
- Il resolver resta non collegato al frame renderer finché gli asset RGBA8 non saranno generati e controllati.

Commit: `4bb33193`, `1ca67908`. Nessuna verifica Cargo o CI eseguita.


## 24 novembre 2026 — Riduzione responsabilità del servizio Cestino

- `components/aurea-desktop/src/trash/service.rs` è stato ridotto da 399 a circa 280 righe.
- La formattazione della data freedesktop è stata spostata in `trash/timestamp.rs`, con test mirati per epoch Unix, anno bisestile e cambio anno.
- I test del contratto pubblico del servizio sono stati spostati in `components/aurea-desktop/tests/trash_service.rs`; il file di produzione non contiene più una lunga sezione test.
- La lettura/scrittura dei metadati resta in `trash/metadata.rs`; le operazioni su file e directory restano in `trash/service.rs`.
- Nessuna nuova dipendenza. Nessun cambiamento al comportamento pubblico previsto.

Commit: `4769658c`, `f00e1d9e`, `78e05c62`, `40a95532`, `a3f6e9b2`.

Verifica soltanto statica dei percorsi e delle dipendenze. Nessuna build, test, rustfmt, cargo check o CI eseguiti.


### Follow-up — Isolamento del ciclo di vita del puntatore Vulkan

- Il file `vulkan/surface_pipeline_cache.rs` è sceso da 398 a 353 righe.
- Il caricamento asincrono della freccia nativa, la fence pendente, la promozione della texture pronta e i relativi errori sono stati spostati in `vulkan/native_cursor_texture.rs` (99 righe).
- Il coordinatore delle pipeline ora delega il ciclo di vita del puntatore a un componente dedicato; non conserva più direttamente lo stato dell'upload del cursore.
- Nessuna modifica alla semantica attesa: il frame non blocca l'upload iniziale e il puntatore non viene esposto finché la texture non è pronta.
- Nessuna dipendenza aggiunta.

Commit: `d0ea103b`, `bd5a6e28`, `bf415144`. Solo revisione statica; nessuna build, test, rustfmt, cargo check o CI.


## 24 novembre 2026 — Correzione del percorso errori del puntatore Vulkan

- Sostituito l'errore Vulkan sintetico `ERROR_UNKNOWN` usato quando un upload completato non contiene texture con la variante esplicita `EmptyUpload`. Un risultato vuoto è un errore del contratto di upload, non un errore restituito dal driver.
- Allineato il compositor al nome attuale della variante `NativeCursor`, introdotta durante l'estrazione del ciclo di vita del puntatore.
- Reso visibile al livello di composizione Vulkan il tipo di errore della cache pipeline, così il pass può distinguere gli errori texture e preservare la classificazione `ERROR_DEVICE_LOST`.
- Nessuna dipendenza aggiunta e nessuna modifica al comportamento del normale percorso frame.

Commit: `30397783`, `04c3aed5`, `552a342b`.

Verifica statica dei percorsi e delle corrispondenze tra tipi. Nessuna build, test, rustfmt, cargo check o CI eseguiti. La compilabilità resta da verificare nel blocco concordato su Fedora.


## 24 novembre 2026 — Primo collegamento del Cestino al Desktop

- Aggiunto `trash/session_state.rs` in Aurea Desktop: legge lo snapshot iniziale del servizio Trash dell’utente e lo converte nel contratto condiviso `DesktopTrashState`.
- Il composition root `aurea-desktop` acquisisce questo valore prima di creare il compositor. Se il servizio non è disponibile, registra un avviso e lascia il desktop avviarsi senza bloccare la sessione.
- `WaylandCompositorFactory` accetta il valore semantico iniziale; `WaylandServer` lo applica al modello desktop condiviso prima dell’avvio dell’event loop.
- Il compositor continua a non accedere al filesystem: riceve soltanto Empty/PartiallyFull/Full già calcolato dal servizio.
- Non sono state aggiunte dipendenze. Il servizio crea/usa la Trash home standard sotto `$HOME/.local/share/Trash`.

**Limite esplicito:** questo è un aggiornamento iniziale all’avvio. Non è ancora presente un watcher o un meccanismo di refresh durante la sessione; il contenuto può diventare obsoleto dopo nuove operazioni di eliminazione/ripristino.

Commit: `46d2c1d6`, `1d3f8fa4`, `611720ee`, `cc7b860e`, `5930c025`.

Solo controllo statico delle interfacce e dei punti di passaggio. Nessuna build, test, rustfmt, cargo check o CI.


## 24 novembre 2026 — Selezione degli elementi sul desktop

- Aggiunti gli eventi semantici `DesktopItemSelected { id }` e `DesktopSelectionCleared` al contratto condiviso `aurea-desktop-api`. L’enum non è più `Copy` perché la selezione trasporta un identificatore posseduto.
- Creato `compositor/desktop_selection.rs`: converte il puntatore globale in coordinate logiche intere, applica `DesktopHitTest` agli elementi del modello con area 64×64 logical px e invia il risultato alla Shell.
- Il percorso del click primario continua a dare precedenza alle superfici Wayland: soltanto quando non viene colpita una finestra il click viene interpretato come selezione di un elemento del desktop.
- `ShellRuntime` aggiorna `DesktopState.selected_item_id` e gestisce anche la cancellazione della selezione. Aggiunti test per selezione e deselezione nel runtime e nello stato desktop.
- Separata la policy di hit-testing dal gestore generale dei pulsanti, mantenendo piccolo `pointer_button.rs`.

**Limite esplicito:** la selezione è ora calcolata e conservata nello stato Shell, ma il renderer non disegna ancora il bordo/evidenziazione di selezione; il doppio clic e l’attivazione/apertura degli elementi restano da implementare. Nessuna icona è ancora visibile finché gli asset RGBA8 generati non vengono prodotti e collegati al pass Vulkan.

Commit principali: `edb084ba`, `57174278`, `281eeb56`, `66259fdc`, `141bb3d2`, `23eefbc5`, `52eb356e`, `1aab344d`, `738a7dca`.

Inoltre corretto il dichiaratore di visibilità duplicato e non valido in `VulkanSurfacePipelineCacheError` (`2169ed40`). Nessuna build, test, rustfmt, cargo check o CI eseguiti.


## 24 novembre 2026 — Riconoscimento del doppio clic sul desktop

### Modifiche pubblicate su main

- Aggiunto `DesktopClickTracker` in `components/aurea-compositor/src/compositor/desktop_click_tracker.rs`, separato dal gestore generale dei pulsanti.
- Il tracker riconosce due clic sullo stesso identificatore entro 400 ms e una distanza massima di 4 pixel logici.
- Usa `std::time::Instant`, quindi tempo monotono; non crea timer, thread o cicli di polling.
- Un clic su una finestra o sul desktop vuoto annulla la sequenza pendente. Dopo un doppio clic il tracker si azzera, così un terzo clic inizia una nuova sequenza.
- Aggiunto l'evento condiviso `DesktopItemActivated { id }`. Il compositor invia prima la selezione e poi l'attivazione riconosciuta.
- `ShellRuntime` conserva l'intento di attivazione nella coda semantica, senza avviare direttamente file o applicazioni.

### Test sorgente aggiunti

Il tracker contiene test per doppio clic valido, elementi differenti, intervallo scaduto, distanza eccessiva, reset e triplo clic. Aggiunto anche un test Shell per la conservazione dell'evento di attivazione.

### Limiti espliciti

- Nessun lancio di file o applicazioni: servirà il servizio Files/Shell per risolvere Desktop Entry, URI, permessi e policy di apertura.
- La selezione non è ancora disegnata dal renderer.
- Le icone native non sono ancora collegate al draw pass Vulkan.
- Il blocco è stato revisionato staticamente; non sono stati eseguiti build, test, rustfmt, cargo check o CI.


## 24 novembre 2026 — Propagazione selezione Shell → frame Vulkan

- Aggiunto `DesktopSelectionController` al contratto condiviso `aurea-desktop-api`, basato su `Arc<RwLock<Option<String>>>`. Non espone guardie di lock: il compositor legge una copia posseduta dell'identificatore.
- Il canale di interazione pubblica la selezione solo dopo l'invio riuscito dell'evento; la pubblicazione immediata evita che il frame richiesto dal click venga preparato prima che il thread Shell abbia consumato il messaggio.
- `ShellRuntime` collega il controller ricevuto dal canale e mantiene sincronizzati il controller e `DesktopState` quando applica selezione/deselezione.
- `AureaCompositor` conserva una copia del controller e `render_scene` campiona la selezione nel frame.
- `DesktopItemsSnapshot` include l'identificatore selezionato come dato immutabile; `DesktopItemDrawGeometry` espone `is_selected` per l'elemento corrispondente su ogni output.
- Aggiunti test sorgente per condivisione del controller, pubblicazione immediata, snapshot immutabile e propagazione del flag geometrico selezionato/non selezionato.

### Limite

Il flag arriva fino alla preparazione della geometria per-output, ma il draw pass Vulkan non lo consuma ancora. Non c'è ancora evidenziazione visibile e non è stato dichiarato completato il rendering delle icone desktop. Nessuna build, test, rustfmt, cargo check o CI è stata eseguita.

## 24 novembre 2026 — Contratto di responsabilità per gli elementi del Desktop

- Chiarita la responsabilità: Desktop Shell governa l'esperienza e le regole di presentazione; il Compositor si occupa di geometria per output, risorse GPU, disegno Vulkan e presentazione.
- aurea-desktop-api resta il confine neutrale per elementi, identificatori, posizioni e intenti di interazione.
- Filesystem/Fides forniscono scoperta, metadati autorevoli, MIME, miniature e operazioni; Fides non disegna le icone del Desktop.
- Trash Service mantiene stato e operazioni del cestino e comunica dati semantici; non disegna la propria icona.
- Icon/Theme risolve gli asset approvati; non decide posizioni e non esegue operazioni sui file.
- La pipeline grafica comune deve supportare File, Folder, Application e Trash. Per i file il tipo autorevole sarà MIME, non la sola estensione; per le applicazioni l'identità deriva dai Desktop Entry.
- Il renderer non deve leggere URI per dedurre tipi, scandire directory, contare elementi del cestino, lanciare applicazioni o contenere policy filesystem.
- Prima dell'integrazione va definito il percorso runtime installato degli asset: i percorsi relativi al repository servono al workflow di generazione e non devono dipendere dalla working directory.
- Aggiornato docs/architecture/DESKTOP-RENDERING-AND-MEDIA-PIPELINE.md con il contratto e la sequenza implementativa.
- Nessun codice runtime modificato in questo blocco. Nessuna build, test, rustfmt o CI eseguiti.

### Prossimo blocco

1. Definire asset root/runtime packaging e loader RGBA8 fuori dal frame loop.
2. Completare la mappatura asset per tutte le categorie.
3. Collegare geometria, cache Vulkan e pass di disegno in moduli dedicati.
4. Implementare evidenziazione della selezione e poi collegare le azioni Shell/Files.


## 24 novembre 2026 — Perimetro del primo desktop utilizzabile

È stato ristretto esplicitamente il prossimo traguardo: non realizziamo ancora il desktop completo. La prima sessione deve mostrare soltanto sfondo, puntatore funzionante e una singola icona Cestino in alto a sinistra, con evidenziazione quando viene selezionata.

- Il modello contiene già un solo elemento Trash predefinito, identificato da `system:trash`, in posizione logica globale `(32, 32)`. Non va duplicato né riposizionato.
- Movimento, pulsanti, doppio clic e rotellina appartengono al percorso input già esistente; la verifica reale sul portatile resta da fare.
- Il flag di selezione raggiunge già la geometria per-output, ma manca ancora il disegno visibile.
- Resolver degli asset, geometria e cache Vulkan sono infrastruttura preparatoria: non sono ancora collegati al draw pass.
- I 24 file RGBA8 del Cestino non sono ancora stati generati né ispezionati. Prima dell'integrazione vanno esportati su Fedora e verificati.
- Non si implementano ora icone File/Cartella/Applicazione, menu contestuali, drag-and-drop o apertura di elementi.
- Il loader dovrà ricevere una root runtime esplicita e caricare gli asset fuori dal ciclo dei frame; niente dipendenza dalla directory corrente e niente letture del filesystem a ogni frame.
- L'icona va disegnata una sola volta nel percorso di composizione condiviso da Vulkan diretto e compatibilità CPU-upload, tra wallpaper e superfici Wayland.

Nessuna build, test, rustfmt o CI eseguiti. La verifica resta raggruppata su Fedora, dopo aver completato l'integrazione e generato gli asset. Non si dichiara ancora visibile o validato il Cestino.


### Loader degli asset desktop — preparazione

- Aggiunto `desktop/icon_asset_loader.rs`, con una responsabilità sola: leggere un asset RGBA8 già generato da una directory radice esplicitamente fornita dal chiamante.
- Il loader rifiuta percorsi assoluti e componenti non normali (incluso `..`), legge il file senza dipendere dalla working directory e riutilizza la validazione RGBA8 di `ShmBufferSnapshot`.
- La root runtime non è ancora scelta dal composition root e il loader non è ancora chiamato dal renderer: il modulo resta deliberatamente preparatorio.
- Nessuna lettura su disco è stata aggiunta al ciclo dei frame; nessuna dipendenza aggiunta.
- Nessuna build, test, rustfmt o CI eseguiti.


## 24 novembre 2026 — Audit formati wallpaper Windows/macOS e direzione Aurea Media

### Decisione prodotto

- Nessun video, nessun codec video e nessuna dipendenza GStreamer/PipeWire per gli sfondi.
- Gli sfondi utente saranno immagini statiche. Un futuro wallpaper dinamico potrà essere costruito con varianti statiche e pianificazione temporale, senza introdurre riproduzione video.
- Confrontate le fonti ufficiali Microsoft e Apple: Windows documenta JPEG, BMP/DIB, TIFF/TIF, PNG e GIF nei file tema, mentre alcune interfacce gestite moderne restringono l'impostazione a JPG/JPEG/PNG. macOS permette di scegliere file, cartelle, album e foto; l'ecosistema Apple documenta JPEG, PNG, GIF, TIFF, HEIC e altri formati, e Preview esporta anche JPEG 2000 e OpenEXR.
- AureaOS adotterà una selezione ampia: JPEG, PNG, BMP/DIB, GIF (solo primo frame), TIFF (solo prima pagina), WebP, HEIC/HEIF e AVIF. JPEG 2000 resta candidato condizionato a backend Debian e revisione licenze. OpenEXR, PDF, SVG e RAW non fanno parte del primo import wallpaper.

### Audit del codice corrente

- aurea-media/src/decoder.rs espone attualmente soltanto ImageFormat::Jpeg.
- aurea-media/src/jpeg.rs usa il TurboJPEG ABI tramite shim C e restituisce RGBA8.
- aurea-media/build.rs compila e collega sempre il solo shim TurboJPEG.
- WallpaperAsset incorpora un JPEG nel binario e usa identificatore/byte slice statici: non può rappresentare direttamente un file scelto dall'utente.
- WallpaperDecodeService accetta un solo WallpaperFormat::Jpeg e passa i byte compressi al worker.
- Quindi la limitazione non riguarda soltanto i decoder: va resa dinamica e posseduta anche l'identità dell'asset wallpaper, senza introdurre accessi filesystem nel frame loop.

### Direzione implementativa

- Mantenere aurea-media come unica API pubblica, decoder privati e output RGBA8.
- Valutare crate Rust image con feature esplicite e disabilitando i default per PNG/BMP/GIF/TIFF/WebP.
- Integrare HEIC/HEIF e AVIF tramite libheif solo dopo verifica ABI, plugin decoder, licenze e pacchetti Debian 13; HEIC richiede il plugin libde265, AVIF un plugin AV1 come dav1d o aomdec.
- Mantenere i limiti di input/dimensioni/pixel/output e verificarli prima dell'allocazione; decodifica sempre nel worker bounded, mai nel render thread.
- GIF e TIFF restano wallpaper statici: primo frame/prima pagina.
- Rivedere WallpaperAsset e WallpaperDecodeKey per file utente con identità posseduta e revisione, mantenendo il caricamento fuori dal frame loop.
- Nessuna build, test, rustfmt, cargo check o CI eseguiti. Nessuna dipendenza o pacchetto è stato aggiunto in questo blocco.


## 24 novembre 2026 — Primo blocco decoder raster Aurea Media

- Aggiunta la dipendenza image 0.25.10 ad aurea-media con default-features disabilitate e sole feature bmp, gif, png, tiff e webp.
- Esteso ImageFormat con JPEG, PNG, BMP, GIF, TIFF e WebP.
- JPEG continua a usare il decoder TurboJPEG già presente; i cinque formati raster nuovi passano da un adapter privato raster.rs.
- Il decoder imposta i limiti massimi sugli assi prima della decodifica e controlla successivamente pixel, byte RGBA8 e lunghezza esatta del buffer.
- GIF e TIFF sono trattati come immagini statiche (primo frame/prima pagina); nessuna animazione o riproduzione video è stata aggiunta.
- Nessuna modifica al compositor in questo blocco.
- Non sono stati eseguiti build, test, rustfmt o cargo check. Cargo.lock non è stato rigenerato e dovrà essere aggiornato su Fedora prima del prossimo build locked.
- La documentazione upstream chiarisce che max_alloc è best-effort/non stretto per alcuni decoder: i limiti applicati non equivalgono a un sandbox di memoria di processo. La revisione sicurezza e i test reali restano obbligatori.
- Prossimo blocco: rendere WallpaperAsset capace di rappresentare file utente con identità/byte posseduti e rilevamento del formato; poi integrare HEIC/HEIF e AVIF tramite backend Debian verificato.

## 24 novembre 2026 — Asset wallpaper posseduti e caricamento file utente

- WallpaperAsset ora possiede ID e byte compressi tramite storage condiviso immutabile, non è più limitato a byte JPEG statici incorporati.
- Aggiunto from_bytes con controlli su ID, input vuoto, limite input e rilevamento firma per JPEG, PNG, BMP, GIF, TIFF e WebP.
- Aggiunto from_file con controllo dimensione preventiva e lettura limitata. Deve essere chiamato da un task impostazioni/media-I/O, mai dal frame/render callback.
- WallpaperManager e WallpaperSnapshot sono stati adattati al nuovo asset non-Copy; le chiavi decode e texture usano identità possedute.
- Il compositor passa una copia posseduta dei byte al worker limitato; il renderer continua soltanto a interrogare risultati e gestire upload Vulkan.
- HEIC/HEIF e AVIF non sono ancora accettati: saranno abilitati insieme ai rispettivi decoder.
- Nessuna build, test, rustfmt o cargo check eseguiti. L'intera modifica resta da verificare su Fedora.

## 24 novembre 2026 — Verifica pacchetti HEIF/AVIF Debian

- Confermata la disponibilità su Debian 13 Trixie di libheif 1.23.6-2~deb13u1 nei metadati security aggiornati.
- HEIC/HEIF richiede il plugin immagini HEVC libheif-plugin-libde265 e libde265-0.
- AVIF richiede un decoder AV1: scelta candidata libheif-plugin-dav1d con libdav1d7.
- Evitare libheif-plugins-all: installare soltanto i decoder effettivamente necessari.
- Restano da verificare licenze upstream complete, ABI, caricamento plugin e pacchetti Fedora. Nessun backend nativo è stato ancora collegato.


## 24 novembre 2026 — Ambito wallpaper statico e verifica formati

- Chiarito e fissato l'ambito: il wallpaper service gestisce immagini raster statiche. Nessun video wallpaper, playback GIF animato, audio, GStreamer, FFmpeg playback o PipeWire.
- Una futura raccolta dinamica potrà cambiare tra immagini statiche in base all'orario o a un intervallo; non userà decodifica video continua.
- Verificata la documentazione Microsoft: i file tema Windows elencano BMP, GIF, JPEG, PNG e TIFF; il formato tema include anche DIB. Alcune interfacce di configurazione gestita restringono i formati a JPEG/PNG/BMP.
- Verificata la documentazione Apple: le impostazioni macOS permettono di scegliere file immagine, foto o raccolte; non viene dichiarato un elenco universale di estensioni nella pagina di supporto, quindi non attribuiamo ad Apple una whitelist non documentata.
- La policy AureaOS resta il superset raster già definito in `AUREA-MEDIA-SERVICE.md`: JPEG, PNG, BMP/DIB, GIF statico (primo frame), TIFF statico (prima pagina), WebP, HEIF/HEIC e AVIF. JPEG 2000 resta condizionale; OpenEXR, PDF, SVG e RAW sono rinviati.
- Verificato che Debian 13 Trixie pubblica `libheif1`, `libheif-dev`, `libheif-plugin-libde265` e `libheif-plugin-dav1d`; il pacchetto principale dichiara dipendenze runtime separate per i decoder. Fedora 45 pubblica libheif con plugin dav1d.
- Il copyright Debian del sorgente libheif indica LGPL-3+ per la libreria principale. Restano da verificare in modo puntuale licenze/notice dei plugin e dei decoder libde265/dav1d, oltre al caricamento runtime dei plugin su entrambe le distribuzioni.
- Aggiornato `docs/design/WALLPAPER-SYSTEM.md` per eliminare il vecchio ambito video/animazioni e allineare architettura, formati e sequenza di implementazione alla decisione statica.
- Nessuna build, test, rustfmt o cargo check eseguiti. Nessuna modifica a Cargo.lock. Prossimo blocco tecnico: completare l'adapter libheif per HEIC/HEIF e AVIF, con controlli dimensioni prima dell'allocazione, output RGBA8 e test su Debian/Fedora; solo dopo abilitare le firme HEIF/AVIF nell'importer.
 

## 24 novembre 2026 — GIF escluse dagli sfondi; TIFF mantenuto come immagine statica

### Decisione prodotto

- GIF rimosse completamente dai formati wallpaper supportati: non vengono accettate neppure come immagine statica del primo fotogramma.
- Nessuna riproduzione animata per GIF, APNG, WebP animato o video. Il wallpaper service gestisce esclusivamente immagini statiche.
- TIFF resta supportato: è un formato raster professionale usato in fotografia, scansione, prestampa e archiviazione. Può essere lossless e supportare profondità colore elevate, ma i file possono essere molto grandi.
- TIFF multipagina viene trattato come immagine statica usando la prima pagina; non esiste una sequenza o riproduzione delle pagine.

### Modifiche sorgente pubblicate su main

- Rimosso `Gif` da `aurea_media::ImageFormat` e dal dispatch decoder.
- Rimossa la feature `gif` dalla dipendenza `image`; restano solo bmp, png, tiff e webp.
- Rimossi `WallpaperFormat::Gif`, il riconoscimento delle firme GIF e il mapping verso il decoder.
- Rimosso il contratto di animazione wallpaper `animation.rs`, inclusi provider frame, policy di riproduzione e relativo stato in manager/snapshot.
- Il media decoder ora descrive e tratta esplicitamente il proprio output come still-image.

### Documentazione

- Aggiornati `docs/design/WALLPAPER-SYSTEM.md` e `docs/architecture/AUREA-MEDIA-SERVICE.md`.
- Questa decisione sostituisce le precedenti note del diario che proponevano GIF o provider GIF/video per gli sfondi.
- Le GIF potranno eventualmente essere trattate da applicazioni social o strumenti dedicati, ma non dal wallpaper manager.

### Verifica

Nessuna build, cargo check, rustfmt o test eseguiti. Cargo.lock non è stato rigenerato. Le modifiche sorgente vanno verificate insieme al prossimo blocco concordato su Fedora.


## 24 novembre 2026 — Registro ufficiale dei formati multimediali

### Decisione di prodotto

Creato un registro unico e vincolante per i formati multimediali che AureaOS decide di supportare. Il registro nasce con la sezione immagini per gli sfondi desktop e sarà esteso in futuro con le decisioni sui codec e contenitori audio/musicali e su eventuali altre categorie multimediali.

Formati immagine ufficialmente approvati per gli sfondi, versione 1.0:
- JPEG: `.jpg`, `.jpeg`, `.jpe`
- PNG: `.png` (solo contenuti statici; APNG escluso)
- BMP: `.bmp`
- TIFF: `.tif`, `.tiff` (se multipagina, viene utilizzata soltanto la prima pagina)
- WebP: `.webp` (solo contenuti statici)
- HEIF/HEIC: `.heif`, `.heic` (solo immagini statiche)
- AVIF: `.avif` (solo immagini statiche)

GIF è escluso anche come primo fotogramma statico. Sono esclusi anche WebP animato, video, SVG, PDF, RAW, JPEG 2000 e OpenEXR dalla prima politica wallpaper. Non aggiungere estensioni o codec senza una decisione esplicita e una revisione Debian, licenze, dipendenze, limiti e test.

### Documenti ufficiali creati

- `docs/architecture/MEDIA-FORMATS.md` — registro principale in inglese.
- `docs/architecture/MEDIA-FORMATS-it.md` — variante italiana.

Il registro distingue approvazione del formato e stato reale dell'implementazione: HEIF/HEIC e AVIF sono approvati come obiettivo di prodotto, ma i rispettivi adapter di decodifica devono ancora essere realizzati e verificati. Il codice dei decoder raster già presente richiede ancora la verifica concordata su Fedora. Nessun formato audio o musicale è ancora approvato: la sezione è predisposta per decisioni future.

Aggiornati anche `docs/architecture/AUREA-MEDIA-SERVICE.md` e `docs/design/WALLPAPER-SYSTEM.md` per rimandare al registro centrale e rimuovere le precedenti indicazioni contraddittorie su GIF, JPEG 2000 e OpenEXR.

### Verifica

Documentazione aggiornata su `main`. Nessuna build, cargo check, rustfmt o test eseguiti. Nessuna modifica a Cargo.lock. La verifica dei decoder resta da eseguire in un unico blocco su Fedora, mantenendo Debian 13 come riferimento per le dipendenze.


## 24 novembre 2026 — Modularizzazione del backend di input

### Obiettivo

Proseguire la costruzione del desktop rendendo più manutenibile il percorso di input nativo. Il file `server/input.rs` conteneva sia la registrazione del backend libinput e la gestione delle preferenze touchpad, sia il dispatch di tutte le famiglie di eventi.

### Modifiche pubblicate su main

- Ridotto `components/aurea-compositor/src/server/input.rs` a 135 righe: registrazione del backend Calloop/libinput, conversione timestamp e applicazione delle impostazioni touchpad.
- Estratto il dispatch degli eventi in `components/aurea-compositor/src/server/input/dispatch.rs` (335 righe).
- Conservati i percorsi esistenti per tastiera, puntatore, assi, gesti swipe/pinch/hold, touchscreen e tavoletta grafica.
- Il nuovo modulo richiama gli stessi handler del compositor; questa modifica è una riorganizzazione strutturale, non una dichiarazione di nuove funzionalità input.

### Vincoli e verifica

- Nessuna build, cargo check, rustfmt o test eseguiti, in linea con la procedura concordata.
- Nessuna modifica a Cargo.toml o Cargo.lock.
- La compilazione e i test dovranno essere eseguiti in blocco su Fedora dopo il pull.
- Da verificare in seguito, con le API effettive di Smithay 0.7, il percorso dei tablet pad e la copertura completa degli eventi libinput; non viene dichiarato supporto completo dei pulsanti/controlli aggiuntivi delle tavolette.

### Prossimo passo

Proseguire l'audit dei percorsi input e dei moduli di interazione, verificando che eventi e dispositivi siano instradati correttamente e che ogni file mantenga una responsabilità singola.



## 24 novembre 2026 — Audit degli eventi Smithay 0.7

### Modifiche al codice

- In `server/input/dispatch.rs` aggiunto un ramo esplicito per `PointerMotionAbsolute`: registra dispositivo e coordinate a livello debug, senza spostare il puntatore. Manca ancora una policy di assegnazione del dispositivo a un output e di conversione in coordinate logiche globali.
- Aggiunto un ramo esplicito per `SwitchToggle`: registra dispositivo, tipo di switch e stato. La policy per chiusura coperchio e modalità tablet resta al livello sessione/risparmio energetico, non nel router input del compositor.
- Gli eventi assoluti e gli switch noti non vengono più assorbiti silenziosamente dal ramo wildcard.

### Risultato dell'audit API

- Smithay 0.7 espone eventi per motion assoluto/relativo, pulsanti/assi, gesti, touchscreen, tablet tool e switch.
- La capability `TabletPad` è presente, ma l'enum `InputEvent` di Smithay 0.7 non espone eventi TabletPad né un relativo trait per pulsanti, ring o strip. Non inventiamo varianti né dichiariamo supporto che l'API non consegna.
- I pulsanti del tablet tool già instradati non equivalgono ai controlli del tablet pad.

### Documentazione

Creato `docs/architecture/INPUT-EVENT-COVERAGE.md` con matrice di copertura, limiti e sequenza di implementazione.

### Verifica

Solo revisione statica del sorgente AureaOS e dell'API Smithay 0.7. Nessuna build, cargo check, rustfmt, test o prova hardware. Verifica raggruppata prevista venerdì su Fedora; Debian 13 resta il riferimento per le dipendenze.


### Correzione lifecycle tablet pad — 24 novembre 2026

- Durante la revisione della rimozione dispositivi è emerso che il cleanup della tablet seat veniva invocato soltanto per dispositivi con capability `TabletTool`.
- Estesa la condizione anche a `TabletPad`: i dispositivi pad-only vengono ora inclusi nel lifecycle cleanup, pur senza dichiarare la disponibilità degli eventi pad in Smithay 0.7.
- La pulizia delle mappe e delle calibrazioni per device continua a passare dal cleanup generico del pointing-device service.
- Nessuna build, cargo check, rustfmt o test eseguiti.

### Correzione instradamento touchscreen dopo disconnessione monitor — 24 novembre 2026

- Individuato un caso limite nella mappa touchscreen: la rimozione di un output cancellava l'assegnazione esplicita del dispositivo.
- Su una configurazione rimasta con un solo monitor, il touchscreen poteva quindi essere instradato automaticamente verso un display diverso da quello a cui era stato assegnato.
- La mappa ora conserva il fatto che l'assegnazione precedente è diventata non disponibile; il dispositivo resta senza destinazione finché non viene riassegnato o esplicitamente riportato alla modalità automatica.
- La rimozione del dispositivo elimina anche questo stato; una nuova assegnazione lo riattiva.
- Nessuna build, cargo check, rustfmt o test eseguiti. Verifica raggruppata su Fedora ancora in sospeso.


### Distinzione inventario tavoletta penna / tablet pad — 24 novembre 2026

- L'audit ha rilevato che un dispositivo con la sola capability `TabletPad` veniva classificato come `Tablet`.
- Questa classificazione poteva renderlo selezionabile nelle API di calibrazione penna e assegnazione monitor, pur non avendo eventi TabletTool.
- Aggiunto `InputDeviceKind::TabletPad` nell'API desktop e separata la rilevazione: `Tablet` richiede `TabletTool`; `TabletPad` identifica i dispositivi pad-only. Se entrambe le capability sono presenti, prevale `Tablet` e le capability pad restano nel descrittore.
- Il registro continua a mostrare i pad-only, ma le API specifiche per penna non li accettano.
- Non è stato aggiunto supporto agli eventi pad: il limite Smithay 0.7 resta invariato.
- Nessuna build, cargo check, rustfmt o test eseguiti; verifica raggruppata su Fedora ancora in sospeso.

### Protezione mapping tavoletta grafica dopo disconnessione monitor — 24 novembre 2026

- Individuata un'incoerenza rispetto alla correzione touchscreen: la mappa delle tavolette eliminava l'assegnazione esplicita quando il monitor veniva rimosso.
- In una configurazione con un solo monitor rimasto, la tavoletta poteva quindi passare automaticamente dal monitor assegnato all'intero desktop.
- La mappa ora conserva l'associazione come non disponibile e blocca la risoluzione finché l'utente non assegna esplicitamente un nuovo output o rimuove l'assegnazione.
- La rimozione fisica della tavoletta elimina sia l'associazione sia lo stato non disponibile. Le assegnazioni degli altri dispositivi restano indipendenti.
- Aggiunti test sorgente per blocco del rerouting, riassegnazione, disassegnazione e isolamento tra dispositivi.
- Nessuna build, cargo check, rustfmt o test eseguiti; verifica raggruppata su Fedora ancora in sospeso.

### Coordinate tavoletta vincolate ai monitor effettivi — 24 novembre 2026

- Esaminato il percorso delle coordinate della penna con più monitor e disposizione non contigua.
- La calibrazione della tavoletta usa un rettangolo complessivo: con monitor sfalsati o separati da uno spazio vuoto, il rettangolo può includere coordinate che non appartengono ad alcun display.
- Dopo la calibrazione, la posizione della penna viene ora vincolata alle geometrie logiche reali degli output abilitati. Se cade in uno spazio vuoto, viene riportata al punto valido più vicino.
- Quando la tavoletta è assegnata a un singolo monitor, il vincolo considera esclusivamente quel monitor: la penna non può sconfinare sugli altri display.
- Riutilizzata la policy geometrica già impiegata dal puntatore relativo; nessuna modifica al percorso touchscreen o agli eventi assoluti generici.
- Nessuna build, cargo check, rustfmt o test eseguiti; verifica raggruppata su Fedora ancora in sospeso.


### Primo instradamento puntatore assoluto — 24 novembre 2026

- Collegato PointerMotionAbsolute al compositor: con un solo output abilitato, le coordinate assolute vengono trasformate nello spazio logico globale del monitor e inviate alla normale seat Wayland del puntatore.
- Il percorso riutilizza il vincolo geometrico degli output e aggiorna il focus sotto il cursore, la frame del puntatore e il rendering.
- Su desktop multi-monitor l'evento viene deliberatamente ignorato: senza un'assegnazione esplicita del dispositivo non indoviniamo il monitor di destinazione.
- Touchscreen e penne continuano a usare i rispettivi percorsi dedicati; non vengono mescolati con il puntatore assoluto generico.
- È un primo supporto parziale, non ancora una gestione completa dei dispositivi assoluti su più monitor.
- Nessuna build, cargo check, rustfmt o test eseguiti. Verifica raggruppata su Fedora ancora in sospeso; nessuna dipendenza o modifica ai manifest Cargo.


### Decodifica immagini statiche HEIC/AVIF opt-in — 24 novembre 2026

- Aggiunto il rilevamento del box ISO-BMFF `ftyp` per i brand statici AVIF (`avif`) e HEIC (`heic`/`heix`). I brand di sequenza (`avis`, `msf1`) e il solo `mif1` generico vengono rifiutati; test unitari coprono box estesi, malformati e troncati.
- Integrata la dipendenza opzionale `libheif-rs 3.0.0` con feature API esplicita `v1_20`. La build predefinita resta indipendente da libheif nativo; il compositor inoltra una feature opt-in `heif`.
- L'adapter applica i limiti di sicurezza libheif prima del parsing, limita i thread di decodifica, rifiuta sequenze e contenitori multi-immagine, verifica dimensioni/stride/lunghezza buffer e restituisce RGBA8 posseduto e compatto. Il formato richiesto deve corrispondere al brand rilevato.
- Rilevamento asset wallpaper e dispatch decoder riconoscono ora HEIC/AVIF, mantenendo le API codec fuori dal renderer compositor. Nessun supporto video o immagini animate è stato introdotto.
- Aggiunto un job CI Debian 13 per installare i soli plugin decoder selezionati e compilare/testare la feature reale Aurea Media. Il primo tentativo si è fermato per un re-export Rust privato e perché Rust 1.85 Debian è troppo vecchio per `image 0.25.10` (richiede Rust 1.88). Entrambi i problemi sono stati corretti; la prossima esecuzione raggruppata deve confermare il risultato.
- Rust formatting resta rinviato alla sessione Fedora raggruppata. Nessuna build/test locale Fedora eseguiti. Restano da completare revisione sicurezza, fixture HEIC/AVIF reali, parità pacchetti Fedora e manifest runtime.


### 2026-11-25 — Abilitazione HEIC/AVIF nell'immagine desktop di produzione

- La revisione del sorgente ha confermato che worker media, servizio di decodifica wallpaper, cache texture Vulkan e renderer sono già collegati. Il collegamento mancante era nella build dell'ISO: il hook compilava `aurea-desktop` senza attivare la feature HEIF opzionale e l'immagine target non installava i plugin runtime libheif.
- Aggiunta la feature opt-in `heif` ad `aurea-desktop`, inoltrata a `aurea-compositor/heif`. La build workspace predefinita resta indipendente da libheif nativo.
- Il hook Debian Live compila ora il desktop con `--features heif`, installa soltanto `libheif1`, `libheif-plugin-libde265` e `libheif-plugin-dav1d` per immagini statiche HEVC/AV1 e rimuove `libheif-dev` dopo la compilazione.
- Aggiunto un controllo CI Debian 13 che compila l'intera catena di dipendenze di `aurea-desktop` con HEIF attivo, oltre ai test del crate media.
- Nessuna build, cargo check, test o rustfmt locale Fedora eseguiti. La prossima CI deve verificare la feature di produzione e l'integrazione dei pacchetti. Revisione sicurezza/licenze e fixture HEIC/AVIF reali restano da completare.


### Audit semantica TouchCancel — 25 novembre 2026

- Revisionato il dispatcher completo degli eventi libinput e il percorso touchscreen.
- Individuato che `InputEvent::TouchCancel` richiama `handle_touch_cancel`, che libera gli slot del dispositivo e invia normali eventi `TouchHandle::up`. Questo comunica al client un normale sollevamento delle dita, non la cancellazione della sessione touch prevista da Wayland.
- Verificato che Smithay 0.7 espone `TouchHandle::cancel`, ma l'API cancella la sessione touch attiva del seat e non accetta un filtro per dispositivo o slot. Usarla per un solo dispositivo può quindi interrompere anche i contatti di un secondo touchscreen.
- Nessuna modifica al comportamento in questo passaggio: prima serve una policy esplicita per la cancellazione seat-wide e il reset coerente della mappa slot. Nessuna build o test eseguiti.


### 25 novembre 2026 — Robustezza del mouse durante il cambio sessione

- Riesaminato il percorso nativo libinput → Calloop → compositor: movimento relativo, pulsanti, rotellina verticale/orizzontale, registrazione del dispositivo e inizializzazione del puntatore Wayland.
- Individuato un caso concreto di stato bloccato: se il mouse tiene premuto un pulsante durante un cambio VT, il rilascio fisico può non arrivare mentre la sessione è sospesa.
- Aggiunta la liberazione di tutti i pulsanti logici prima che libseat sospenda la sessione. La logica aggregata evita doppi rilasci quando più mouse tengono premuto lo stesso pulsante.
- Non dichiaro ancora il mouse completato: mancano build/test raggruppati su Fedora e prova fisica di movimento, clic, rotellina, scollegamento/ricollegamento e cambio VT.


### 25 novembre 2026 — Menu contestuale con tasto destro del mouse

- Aggiunto il comportamento standard del clic destro sul desktop: seleziona l'elemento sotto il puntatore (oppure deseleziona se si clicca sullo spazio vuoto) e invia alla Shell la richiesta di apertura del menu contestuale.
- Sulle finestre applicative il clic destro continua a essere consegnato normalmente al client Wayland, che mantiene il proprio menu contestuale.
- Il clic destro non altera il riconoscimento del doppio clic sinistro.
- Modifica sorgente e documentazione; nessuna build, cargo check, rustfmt, test o prova con mouse fisico eseguiti.

### 25 novembre 2026 — Limiti di sicurezza per il caricamento delle icone desktop

- Rafforzato `desktop/icon_asset_loader.rs`: accetta solo percorsi relativi normalizzati, risolve il percorso reale e rifiuta file/symlink che escano dalla root esplicita degli asset.
- Verifica la dimensione RGBA8 attesa dal descrittore prima di allocare e leggere il contenuto, con limite pixel già vincolato a 256 × 256.
- Aggiunti test sorgente per caricamento valido, traversal `..`, dimensione non conforme e symlink esterno.
- Commit `6cbc2669`. Il loader resta preparatorio: la root runtime non è ancora fornita dalla composition root e il rendering delle icone non è collegato.
- Nessun `cargo fmt`, `cargo check`, test, build o prova hardware eseguiti. Verifica raggruppata su Fedora ancora in sospeso.

### 25 novembre 2026 — Collegamento dei clic alla selezione desktop

- Collegato il click sinistro sull'area desktop libera a `select_desktop_item_at`: seleziona l'elemento sotto il puntatore, riconosce il doppio clic già previsto e deseleziona quando si clicca sullo spazio vuoto.
- Collegato anche il click destro: aggiorna prima la selezione (o la cancella sullo spazio vuoto), poi invia `ShowContextMenu` con la posizione logica del puntatore.
- I clic sopra superfici Wayland continuano a rispettare la precedenza delle finestre; il click destro resta consegnato all'applicazione.
- Commit `80483c23`. Il modello Shell conserva già pannello e posizione del menu, ma non esiste ancora una superficie grafica che lo disegni: il menu non è quindi ancora visibile.
- Nessuna build, cargo check, test, rustfmt o prova hardware eseguiti; verifica raggruppata su Fedora.


### 25 novembre 2026 — Primo configure dei popup XDG

- In `aurea-compositor/src/xdg_shell.rs` il callback `new_popup` ora salva il `PositionerState`, calcola la geometria iniziale con `get_geometry()` e invia `send_configure()`.
- `reposition_request` aggiorna lo stato pending e risponde con `send_repositioned(token)`, richiedendo un nuovo frame.
- Questo consente ai client di completare il primo ciclo configure/commit e di ricevere una risposta alle richieste di riposizionamento.
- Non è ancora un menu contestuale grafico: mancano constraint rispetto ai bordi degli output, PopupManager e grab di tastiera/puntatore, integrazione nella scena Vulkan e hit testing dei popup.
- `grab` resta intenzionalmente non implementato: non viene simulato un grab parziale che violerebbe il comportamento Wayland.
- Nessuna build, cargo check, rustfmt, test o prova hardware eseguiti. Verifica raggruppata su Fedora, come concordato.


### 25 novembre 2026 — Registry e albero dei popup XDG

- Integrato `smithay::desktop::PopupManager` nello stato del compositor.
- I popup XDG vengono registrati alla creazione; a ogni commit il manager aggiorna le relazioni parent/child e ripulisce le risorse distrutte.
- È il primo collegamento del lifecycle popup al compositor, non ancora la visualizzazione del menu: constraint per monitor, grab coordinati, rendering Vulkan, hit testing e dismiss restano da implementare.
- Nessuna build, cargo check, rustfmt o test eseguiti: continuiamo ad accumulare modifiche per la verifica raggruppata su Fedora.


### 25 novembre 2026 — Popup XDG nella scena Vulkan

- `RenderScene` raccoglie ora anche i surface tree dei popup registrati in `PopupManager`.
- Le coordinate vengono convertite in posizione globale logica; i popup entrano nel normale percorso snapshot SHM e quindi nella composizione Vulkan, con clipping per output.
- Corretto l'ordine di composizione: finestra genitrice prima, popup dopo, così il popup resta visivamente sopra la finestra. Gli ID dei popup sono inclusi nella gestione della cache texture.
- L'hit testing controlla i popup prima del toplevel, rispettando la sovrapposizione e instradando il puntatore sulla superficie popup.
- Restano constraint ai bordi del monitor, grab coordinati di mouse/tastiera, dismiss e prove visive multi-monitor. Nessuna build o test eseguiti.

### 25 novembre 2026 — Grab coordinati per popup XDG

- Collegata la richiesta `xdg_popup.grab` al seat effettivamente associato alla risorsa `wl_seat` ricevuta, usando `Seat::from_resource` e rifiutando il caso in cui manchino le capability di puntatore o tastiera.
- Il compositor individua la superficie radice del popup e chiede a Smithay `PopupManager::grab_popup` di validare serial e gerarchia.
- Installati `PopupPointerGrab` e `PopupKeyboardGrab` sullo stesso serial. La gestione standard Smithay copre focus nel popup tree, dismiss fuori dal grab e ripristino del focus radice.
- Modifiche sorgente su `main`; nessuna build, formattazione o test anticipato. Verifica raggruppata su Fedora ancora da eseguire.
- Restano aperti: vincoli geometrici rispetto all'output, dismiss esplicito da tastiera e verifica runtime dei menu annidati.

### 25 novembre 2026 — Posizionamento popup vincolato ai monitor

- Creato `compositor/popup_placement.rs` come modulo autonomo per la geometria dei popup.
- Il posizionamento risolve il toplevel radice e il parent reale del popup, inclusi i parent che sono a loro volta popup annidati.
- Il monitor di riferimento viene scelto usando il punto di ancoraggio XDG in coordinate globali; se il punto cade nello spazio tra due monitor, si ripiega sul monitor che contiene il centro della finestra genitrice. Nessun fallback cieco al primo output.
- I bounds dell'output sono convertiti nel sistema locale della window geometry del parent e passati a `PositionerState::get_unconstrained_geometry` di Smithay 0.7, che applica flip, slide e resize secondo i flag richiesti dal client.
- La logica viene usata sia nel configure iniziale sia nelle richieste `reposition`. Il token viene confermato dopo l'aggiornamento della geometria.
- Aggiunti test sorgente per conversione coordinate e overflow; non eseguiti.
- Restano da implementare la riconfigurazione reattiva quando il parent si sposta o cambia monitor e la validazione runtime con più monitor, scale differenti e rotazioni.
- Nessuna build, `cargo fmt` o test eseguiti: verifica raggruppata su Fedora.

### 25 novembre 2026 — Popup reattivi e cambiamenti della geometria desktop

- Introdotta una registry dedicata che conserva il positioner XDG dell'ultimo configure per ogni popup vivo, lo aggiorna con `reposition` e rimuove le risorse distrutte.
- La selezione del monitor usa ora l'anchor point reale calcolato da Smithay, inclusi anchor edges e gravity, non il semplice centro del rettangolo di ancoraggio.
- I popup marcati `reactive` vengono ricalcolati dopo commit delle superfici, modifiche alla topologia/configurazione dei monitor e spostamento di una finestra tramite `AureaCompositor::set_toplevel_location`.
- Per inviare aggiornamenti si usa `send_pending_configure`: nessun evento viene spedito se la geometria non è cambiata; Smithay continua a verificare versione del protocollo e flag reactive.
- L'API di spostamento aggiorna anche l'appartenenza delle superfici agli output e il focus del puntatore. I futuri gestori di move/drag dovranno passare da questa API invece di modificare direttamente il `WindowManager`.
- I popup non reactive restano invariati secondo il protocollo. I figli annidati vengono ricalcolati nuovamente quando il parent conferma la nuova geometria.
- Il trascinamento interattivo delle finestre non è ancora implementato: è predisposto il punto di integrazione, non il gesto completo.
- Nessuna build, `cargo fmt`, check o test eseguiti; verifica cumulativa su Fedora.
 
### 25 novembre 2026 — Chiusura del menu contestuale con Esc e click esterno

- Aggiunto l'evento semantico `DismissContextMenu` e uno stato atomico condiviso tra compositor e Shell che segnala se il menu desktop è aperto.
- Premendo `Esc`, il compositor chiude il menu e consuma anche repeat/rilascio; quando il menu non è aperto, `Esc` continua a essere consegnato normalmente alle applicazioni.
- Il click sinistro sul desktop, il click con altri pulsanti sul canvas e qualsiasi click sopra una finestra applicativa chiudono il menu. Il click destro sul desktop lo riposiziona.
- La Shell chiude soltanto il menu contestuale, senza interferire con Overview o App Exposé.
- Aggiunti test sorgente per canale, stato e comportamento della Shell; non eseguiti.
- Il menu non è ancora disegnato: la richiesta e la chiusura sono operative a livello semantico, ma manca ancora il renderer nativo dei pannelli e quindi non possiamo dichiarare completata la visualizzazione o la selezione delle voci.
- Nessuna build, cargo fmt, check o test: verifica cumulativa su Fedora.

### 25 novembre 2026 — Contratto geometrico del menu contestuale

- Creato `aurea-desktop-api/src/desktop/context_menu.rs`: modello indipendente dalla grafica per i comandi e le aree cliccabili del menu desktop.
- Il menu standard prevede inizialmente quattro voci: Crea cartella, Usa come sfondo, Impostazioni schermo e Personalizzazione. Dimensioni logiche 248 × 160, righe da 36 e padding superiore/inferiore da 8.
- Il layout usa il posizionamento già presente per mantenersi dentro l'output corretto, anche con coordinate globali e configurazioni multi-monitor. Il test di hit-testing non considera il padding come comando.
- Esportato il nuovo contratto dal modulo `desktop` dell'API. È ancora un modello: rendering, sincronizzazione del layout con il compositor e dispatch dei comandi non sono collegati. Nessuna build, `cargo fmt`, check o test eseguiti.

### 25 novembre 2026 — Etichette e passaggio dei comandi contestuali

- Ogni comando del menu espone ora la propria etichetta italiana canonica. La seconda voce è `Usa come sfondo`.
- Aggiunto `ContextMenuCommandSelected` al contratto di interazione tra compositor e Shell. La selezione chiude lo stato condiviso del menu e la Shell chiude il pannello contestuale.
- Il comando selezionato resta nella coda semantica per essere consumato dal servizio competente. Non abbiamo messo operazioni filesystem o impostazioni dentro ShellRuntime.
- Aggiunti test sorgente per il trasferimento del comando e la chiusura del menu; non eseguiti. Nessuna build, `cargo fmt` o check.


### 25 novembre 2026 — Hit-testing del menu e grafica Noto Sans

- Il canale compositor/Shell conserva ora l'ancora globale del menu insieme allo stato aperto/chiuso. La posizione viene eliminata alla chiusura o quando viene selezionato un comando.
- Il compositor individua il monitor dell'ancora e ricostruisce il layout limitato a quell'output. Un click sinistro su una riga produce ContextMenuCommandSelected; il padding chiude senza eseguire; il click esterno chiude e continua il routing normale.
- Il pulsante premuto sul menu e il relativo rilascio vengono consumati, così il click non trapassa verso una finestra sottostante.
- Create le SVG Light/Dark con le quattro etichette italiane e icone essenziali. Font scelto: Noto Sans di Google Fonts, licenza OFL 1.1, già conservato in Human Interfaces (`fonts/source/google-fonts/noto-sans/NotoSans-Variable.ttf`).
- Aggiornato `tools/assets/render-context-menu.sh`: usa il font direttamente dal repository tramite una configurazione Fontconfig temporanea. Non richiede l'installazione di Noto Sans sull'host; restano necessari Fedora `librsvg2-tools` o Debian `librsvg2-bin` e Python 3.
- Da completare: generare e ispezionare gli asset su Fedora, collegare asset root/cache e inserirli nel frame Vulkan sopra le finestre e sotto il puntatore. Il menu non è ancora visibile. Nessuna build, fmt, check o test eseguiti.

### 25 novembre 2026 — Handoff del menu alla RenderScene

- `RenderScene` ora conserva un `DesktopContextMenuLayout` opzionale insieme agli snapshot del desktop.
- `AureaCompositor::render_scene` legge l'ancora condivisa, individua l'output abilitato che la contiene e inserisce nella scena la geometria vincolata. Hit-testing del puntatore e futura presentazione grafica usano così lo stesso contratto, senza duplicare il posizionamento.
- Corretto anche il percorso tipografico: Noto Sans è già presente in Human Interfaces e lo script usa quel file direttamente tramite una configurazione Fontconfig temporanea. Non serve installare un secondo pacchetto font su Fedora o Debian.
- Questo passaggio trasporta soltanto la geometria: asset RGBA, texture Vulkan e draw pass non sono ancora collegati. Il menu non è ancora visibile.
- Nessuna build, `cargo fmt`, check o test eseguiti, in linea con la decisione di accumulare le modifiche prima della verifica su Fedora.

### 25 novembre 2026 — Overlay Vulkan del menu contestuale

- La scena di rendering ora trasporta anche il tema attivo del desktop insieme al layout geometrico del menu.
- Il `VulkanSurfacePipelineCache` carica gli RGBA Light/Dark una sola volta al primo utilizzo, con il loader che valida percorso, dimensioni e contenuto. La texture viene mantenuta nella cache artwork Vulkan; il frame loop non legge file dal disco.
- Aggiunto il draw pass del menu nel compositor Vulkan. L'overlay usa la pipeline testurizzata con alpha già esistente, viene disegnato sopra le finestre e sotto il puntatore e viene ritagliato per ogni output usando origine logica e scala effettiva del monitor.
- Le superfici cursore fornite dai client Wayland sono ora distinte dalle normali finestre nella composizione, così restano sopra il menu. Anche il cursore nativo continua a essere disegnato per ultimo.
- Asset root in sviluppo: variabile `AUREA_HUMAN_INTERFACES_ROOT` puntata alla directory `aurea-human-interfaces`. Installazione prevista: `/usr/share/aureaos/human-interfaces`. In assenza dei file RGBA il compositor segnala l'asset mancante una volta e non interrompe l'avvio.
- Aggiunti test sorgente per coordinate 1x/2x e monitor non interessati. Gli asset binari non sono ancora generati/ispezionati; nessuna build, `cargo fmt`, check o test eseguiti.
- Il convertitore PNG→RGBA condiviso ora converte l'alpha PNG non premoltiplicato in RGBA premoltiplicato, necessario per il blend Vulkan `ONE / ONE_MINUS_SRC_ALPHA` e per evitare aloni sui bordi trasparenti.


### 25 novembre 2026 — Primo frame garantito del puntatore nativo

- Individuato un problema nel caricamento asincrono iniziale della freccia: il primo frame poteva terminare senza disegnarla e non era garantito un secondo frame se nel frattempo non arrivavano altri eventi input.
- La texture Vulkan della freccia (32×32 RGBA) viene ora caricata e completata una sola volta prima del primo frame che la utilizza. Dopo l'inizializzazione resta in cache per tutta la vita del dispositivo Vulkan; i frame successivi non ripetono l'upload.
- La modifica evita che il puntatore resti invisibile sul desktop fermo subito dopo l'avvio. Non sostituisce la verifica fisica del movimento: quella resta da eseguire su Fedora.
- Nessuna build, cargo fmt, check o test eseguiti, come concordato.


### 26 novembre 2026 — Riposizionamento differito del puntatore dopo hot-unplug

- Chiuso un caso limite del lifecycle: se un monitor veniva rimosso mentre un client possedeva un pointer grab, AureaOS proteggeva correttamente il grab ma poteva perdere il ricentramento richiesto.
- Aggiunto `pointer_recenter_pending` nel compositor. Le richieste di ricentramento bloccate da un grab vengono conservate invece di essere scartate.
- La riconciliazione della topologia controlla anche se la posizione corrente è diventata esterna all'unione dei monitor attivi durante un grab; in tal caso registra il riposizionamento differito.
- Dopo ogni dispatch Calloop, il loop DRM ritenta solo se esiste una richiesta pendente. Quando il grab termina, il puntatore viene ricentrato su un output attivo e vengono richiesti i frame.
- Nessuna build, `cargo fmt`, check o test eseguiti. La verifica di hot-unplug e popup grab resta da fare su Fedora insieme al ciclo di test concordato.


### 26 novembre 2026 — Preferenze e shake del puntatore isolate per dispositivo

- Il percorso di movimento relativo ora passa al compositor anche l'identificativo fisico del dispositivo, senza perdere la provenienza dell'evento.
- La preferenza `shake_to_locate` viene letta dal profilo effettivo del singolo mouse/touchpad, coerentemente con il profilo di accelerazione già applicato in fase di dispatch.
- Creato `PointerShakeManager`: ogni dispositivo mantiene una cronologia indipendente dei movimenti non accelerati. Lo stato viene eliminato quando il dispositivo viene scollegato o quando la funzione viene disattivata per quel dispositivo.
- Aggiunti test sorgente per impedire che movimenti alternati provenienti da due dispositivi diversi generino un falso shake e per verificare la pulizia selettiva. Non eseguiti.
- Nessuna build, `cargo fmt`, check o test eseguiti.


## 26 novembre 2026 — Collegamento delle preferenze live del mouse

### Modifiche pubblicate su `main`

- Aggiunti `PointerSettingsController` e `PointerSettingsCommand` nell'API desktop condivisa.
- La Shell possiede il controller e offre il metodo `set_pointer_settings` per inviare un profilo a uno specifico mouse connesso.
- Il canale attraversa l'avvio dell'applicazione, la factory desktop, la factory del compositor e il server Wayland fino al ciclo Calloop.
- Il compositor verifica che il dispositivo esista e sia classificato come mouse; rifiuta dispositivi sconosciuti o di altro tipo.
- Il profilo viene aggiornato nello stato per-device del compositor. L'adattatore libinput applica velocità e profilo al successivo evento di movimento, senza conservare handle del mouse che potrebbero diventare obsoleti dopo una disconnessione.

### Scelte mantenute

- Nessuna interfaccia temporanea è stata aggiunta.
- I controlli di velocità e doppio clic saranno inseriti in futuro nella pagina Input di Impostazioni di sistema, insieme alle preferenze del touchpad.
- L'intervallo di doppio clic attuale non è stato modificato.

### Verifica e limiti

- Aggiunto un test unitario per il passaggio del comando dalla Shell al canale del compositor.
- Non sono stati eseguiti build, `cargo check`, `cargo fmt`, test o verifiche su hardware, in accordo con la decisione di accumulare le modifiche e svolgere i controlli insieme su Fedora.



### 26 novembre 2026 — Pulsanti laterali, rotellina e ripristino del mouse

- Verificato il percorso dei pulsanti ausiliari: `BTN_SIDE`, `BTN_EXTRA`, `BTN_FORWARD` e `BTN_BACK` vengono già inoltrati a Wayland con il codice Linux originale. Il menu contestuale desktop ora intercetta esclusivamente clic sinistro e destro; i pulsanti laterali non vengono più assorbiti dal menu. Aggiunti test unitari per questa separazione.
- La rotellina verticale e orizzontale viene già inoltrata come frame Wayland axis. Sono preservati valori continui, valori ad alta risoluzione `v120`, sorgente e direzione relativa. Restano validi i test esistenti sulla gestione dei valori numerici.
- Il compositor conserva gli handle libinput dei mouse solo finché il dispositivo risulta connesso. I comandi delle preferenze applicano ora accelerazione e velocità immediatamente all'handle attivo.
- Alla riattivazione della sessione libseat, ad esempio tornando da un altro terminale virtuale, i profili di accelerazione vengono riapplicati ai mouse collegati; viene ripristinato anche il profilo di accelerazione dei touchpad attraverso il loro registro separato.
- Alla disconnessione o riclassificazione del dispositivo, l'handle viene rimosso. La gestione esistente rilascia inoltre i pulsanti rimasti premuti prima della pausa della sessione e durante la rimozione.
- Da verificare fisicamente su Fedora: hot-plug, pulsanti laterali, rotellina e sospensione/riattivazione VT. Nessuna build, `cargo fmt`, `cargo check` o test eseguiti in questa sessione.


## 26 novembre 2026 — ISO Live: migrazione a systemd-boot e UKI

### Ambito concordato

- La ISO serve esclusivamente ad avviare in modalità Live il desktop AureaOS realmente presente nel repository e a verificarne i progressi su hardware fisico.
- Installer, partizionamento, installazione su Btrfs, aggiornamenti di sistema e ambiente di ripristino sono rinviati a quando desktop e applicazioni saranno completi.
- La politica Roma 1.0 approvata resta UEFI-only, systemd-boot, UKI e Secure Boot disabilitato. GRUB non deve essere presente nell'immagine finale.
- Si mantiene il flusso Fedora + Podman + Debian Trixie. Nessuna ISO viene generata durante questa sessione di modifica del sorgente.

### Modifiche pubblicate su main

- Creato `tools/iso/os-release` con l'identità AureaOS da incorporare nelle UKI.
- Creato `tools/iso/scripts/build-systemd-boot-live-iso.sh`: usa le coppie kernel/initramfs prodotte da Debian Live, genera UKI con `ukify` e lo stub EFI Debian, prepara un'immagine EFI FAT con systemd-boot e voci di avvio, e rimasterizza la ISO con catalogo El Torito EFI e struttura GPT ibrida.
- L'immagine EFI usa il percorso rimovibile standard `EFI/BOOT/BOOTX64.EFI`; systemd-boot carica le UKI dalla stessa ESP. La riga di comando UKI incorpora `boot=live` e le opzioni Live già selezionate.
- La fase finale elimina i file GRUB intermedi e i kernel/initramfs separati dalla directory `/live`: nella ISO rimane il filesystem Live, mentre kernel e initramfs sono contenuti nella UKI.
- Aggiornato `build-live-iso.sh` per richiamare il generatore UKI e il validatore prima di pubblicare la candidata.
- Aggiornato il wrapper Podman per installare nel container Debian Trixie `systemd-boot-efi`, `systemd-ukify`, `mtools` e `dosfstools`.
- Riscritto `validate-live-image.sh`: controlla loader EFI, configurazione systemd-boot, UKI e parametro `boot=live`, filesystem Live e riferimento El Torito EFI. Rimosso il vecchio hook di branding GRUB.
- Aggiornati i documenti ISO Factory in inglese e italiano, il log di sviluppo e il checkpoint README.

### Percorso di avvio previsto

1. Firmware UEFI avvia `EFI/BOOT/BOOTX64.EFI` dall'immagine EFI El Torito.
2. systemd-boot mostra la voce AureaOS Live e carica la UKI selezionata.
3. La UKI avvia kernel Debian e initramfs con `boot=live`.
4. live-boot cerca il supporto Live e prepara il filesystem `/live/filesystem.squashfs`.
5. systemd raggiunge il target multi-user e avvia `aurea-desktop.service`.
6. PAM/logind registrano la sessione; il servizio esegue il binario Rust `aurea-desktop`.
7. Il compositor Smithay usa DRM/KMS e Vulkan per presentare il desktop e il puntatore.

### Verifica e limiti

- La documentazione Debian Trixie conferma che live-boot è un hook initramfs-tools e viene attivato dal parametro `boot=live`; Trixie fornisce systemd-boot EFI e systemd-ukify.
- Il vecchio panic `/scripts/live` resta una diagnosi aperta: il nuovo preflight controlla la presenza dello script nell'initramfs e il validatore finale controlla la UKI, ma soltanto un avvio reale potrà dimostrare che la catena è corretta.
- **Non verificato:** nessuna build ISO, verifica di sintassi Bash, invocazione ukify, ispezione immagine FAT, report El Torito, boot QEMU/OVMF o avvio fisico è stato eseguito. Nessun comando Cargo è stato eseguito.
- Il prossimo ciclo di verifica deve essere raggruppato su Fedora: sincronizzazione `main`, `cargo fmt`, check/test workspace, generazione ISO, controlli automatici e infine avvio tramite la chiavetta Ventoy già esistente.


### 26 novembre 2026 — Correzione staging preflight ISO Live

- Una revisione aggiuntiva ha individuato un errore nel coordinatore: l'hook chroot eseguiva `/usr/local/libexec/aureaos/validate-chroot-boot.sh`, ma la directory include veniva creata senza copiare al suo interno lo script di validazione. Una build nuova avrebbe quindi potuto fallire al preflight anche se il file esisteva nel repository.
- Aggiornato `build-live-iso.sh` per copiare il validatore nel chroot con permessi eseguibili prima dell'esecuzione di live-build.
- Rafforzata la validazione UKI finale: devono essere presenti le sezioni PE `.linux`, `.initrd`, `.osrel`, `.cmdline`, il parametro incorporato `boot=live` e l'identità AureaOS `ID=aureaos`.
- Corretta la documentazione ISO Factory EN/IT: il supporto Live finale contiene il filesystem squashfs e la UKI, non kernel/initramfs separati; l'unico percorso di avvio previsto è UEFI.
- Nessuna build, verifica sintassi shell, invocazione ukify, generazione ISO, test QEMU/OVMF o prova hardware è stata eseguita. Correzione sorgente in attesa della verifica raggruppata su Fedora.

### 26 novembre 2026 — Pubblicazione sicura e tracciabilità delle ISO Live

- Irrigidito `tools/iso/build-live-iso.sh`: una ISO non validata non può più sovrascrivere il nome stabile `AureaOS.iso`.
- Il coordinatore verifica prima il tree binary, copia poi la ISO in un file candidato temporaneo nascosto e valida proprio quel file. Solo dopo calcola SHA-256 e dimensione e pubblica una candidata storica con timestamp UTC e commit sorgente nel nome.
- Aggiunto un manifest JSON accanto a ogni candidata: remote/branch/commit, stato pulito o modificato del working tree, target Debian Trixie amd64, politica systemd-boot/UKI, versione kernel, dimensione/hash ISO e stato esplicito delle verifiche automatiche e hardware.
- Il nome stabile `AureaOS.iso` e il relativo checksum vengono sostituiti atomicamente soltanto dopo il superamento di entrambe le validazioni automatiche. Le candidate storiche e i manifest con nomi univoci non vengono sovrascritti.
- Il container Debian del wrapper Podman installa esplicitamente Python 3 per generare il manifest JSON.
- Installer, partizionamento, installazione Btrfs, Aurea Update e Aurea Recovery restano fuori ambito. Questo traguardo riguarda soltanto l'avvio Live del desktop reale già presente nel repository.
- **Non verificato:** nessun controllo di sintassi shell, esecuzione del manifest Python, live-build, generazione UKI, validazione ISO, QEMU/OVMF o avvio fisico è stato eseguito. Il precedente panic `/scripts/live` resta aperto fino all'avvio della nuova immagine su hardware reale.
- Una revisione statica successiva ha individuato e corretto un percorso incoerente nel validatore ISO: l'immagine EFI viene ora estratta in `boot/efi.img`, esattamente dove il validatore la cerca per i controlli successivi. Commit: `9de7855e`. Correzione non ancora verificata in esecuzione.
- Ulteriore protezione ISO Factory: il wrapper Podman su Fedora sceglie ora un workspace predefinito legato al commit sorgente e rifiuta repository con modifiche non committate o file non tracciati. Evita così di riprendere un chroot completato che contenga un vecchio binario desktop dopo il cambio di commit. La ripresa vale soltanto per lo stesso commit pulito. Commit: `52d51009`, `3f04e842`. Nessuna build o verifica runtime eseguita.


### 26 novembre 2026 — Hardening della fabbrica Debian Live e controllo initramfs UKI esatto

- Chiarimento del perimetro richiesto: la fabbrica deve consegnare in modo affidabile una Debian 13 Trixie Live avviata; dal punto di consegna in avanti, compositor, Vulkan, sessione desktop e interfaccia visibile sono lavoro AureaOS. Non riaprire continuamente la catena Debian già stabile senza un'evidenza di errore.
- Il generatore UKI rimuoveva kernel/initramfs e file GRUB direttamente da `BUILD_ROOT/binary`. Se xorriso o la validazione fallivano, una successiva `RESUME=1` non avrebbe più trovato kernel/initramfs. Ora crea una copia isolata sullo stesso filesystem con `cp -al`, modifica soltanto quella copia, la valida e crea la ISO da lì. Il tree live-build originale resta integro sia in caso di successo sia di errore.
- Corretto un errore nel nome temporaneo ISO introdotto durante il refactoring e rimossa una doppia registrazione del trap di pulizia; il nome temporaneo contiene nuovamente il PID. Il riepilogo del generatore indica ora `boot/efi.img` dentro la ISO, non un percorso temporaneo già eliminato.
- Il validatore finale controlla il vero SquashFS: identità Debian 13 Trixie, systemd PID 1 e stato installato di `live-boot`, `live-boot-initramfs-tools`, `live-config`, `live-config-systemd` e `systemd-sysv`.
- Soprattutto, estrae la sezione `.initrd` da ogni UKI effettiva con `ukify inspect --section=...:binary@...`, esegue `lsinitramfs` sul payload estratto e richiede `scripts/live`. In questo modo controlla i byte che verranno realmente avviati, non soltanto un initramfs precedente presente nel chroot.
- Il container builder installa ora `initramfs-tools-core` per questa ispezione finale. Il preflight initramfs nel chroot è stato spostato prima della rimozione delle dipendenze di compilazione, così un errore del controllo può essere ripreso senza aver prima perso la toolchain.
- Il preflight Podman su Fedora verifica esplicitamente anche la presenza di `sudo`.
- Modifiche statiche al sorgente. In questa sessione non sono stati eseguiti `bash -n`, ShellCheck, `ukify`, live-build, validazione SquashFS, QEMU/OVMF o boot fisico. Il controllo Debian base è quindi implementato, ma non ancora osservato superare una build.

### 26 novembre 2026 — Audit end-to-end della ISO Live e correzioni

- Verificata staticamente la catena Debian Live, pacchetti, initramfs, UKI, servizio systemd, sessione PAM/logind, avvio del binario desktop, compositor e risorse grafiche runtime.
- Individuate dipendenze di compilazione mancanti: `aurea-media` compila sempre lo shim C che collega TurboJPEG e la ISO abilita la feature `heif`. Aggiunti `libturbojpeg0` runtime, `libturbojpeg0-dev` e `libheif-dev` build-time. Gli header vengono rimossi dopo la compilazione; la libreria TurboJPEG runtime resta nell'immagine.
- Aggiunto un marcatore del commit sorgente al workspace live-build. Il coordinatore diretto rifiuta ora repository sporchi e riprese da workspace senza marcatore o appartenenti a un commit diverso. Evita di pubblicare un binario vecchio con il manifest di una revisione nuova.
- L'hook chroot controlla ora le librerie dinamiche del binario con `ldd`, valida il servizio tramite `systemd-analyze verify` e conferma che il servizio desktop sia abilitato.
- Aggiunta la generazione build-time dei due asset RGBA8 del menu contestuale a partire dalle SVG canoniche nel container Debian temporaneo. Il coordinatore inserisce nel rootfs soltanto i file RGBA; librsvg, Python, SVG e font non entrano nel sistema installato. Gli asset del Cestino restano fuori dal payload Live finché il draw pass degli elementi desktop non li utilizzerà.
- Corretto il percorso degli asset Cestino: il descrittore aggiungeva `aurea-human-interfaces/` anche se il loader riceve già come root `/usr/share/aureaos/human-interfaces`, duplicando il percorso. Ora usa `icons/generated/trash/`; aggiunta un'asserzione di regressione.
- Esteso il validatore ISO: estrae e ispeziona il vero SquashFS, verifica binario desktop, launcher, unit systemd, sessione PAM, link di abilitazione e tutti i 2 asset del menu contestuale RGBA con dimensione esatta. Aggiunto `squashfs-tools` al container Debian di build.
- Il generatore UKI passa esplicitamente a `ukify` lo stub Debian già verificato come presente.
- I metadati dei pacchetti Debian Trixie confermano che `live-config` richiede anche un backend per l'init system; poiché l'immagine disabilita le raccomandazioni APT, aggiunto esplicitamente `live-config-systemd` e reso obbligatorio nel preflight chroot.
- **Non verificato in esecuzione:** non sono stati eseguiti controlli di sintassi shell, build/test Cargo, live-build Debian, generazione UKI, validazione SquashFS, QEMU/OVMF o avvio fisico dopo queste modifiche. Il precedente panic `/scripts/live` resta aperto fino all'avvio della nuova ISO.
- **Limite desktop:** documentazione e codice del repository indicano ancora incompleta la presentazione fisica e la composizione completa della scena. Il client Wayland Welcome esiste e disegna una superficie preview 720×440 dopo aver ricevuto XDG configure, ma un vecchio log si fermava a “waiting for configure”: non dimostra che la finestra sia apparsa sul monitor. La prova hardware deve includere configure ricevuto, frame SHM inviato, presentazione Vulkan/KMS e immagine visibile.

### 26 novembre 2026 — Inventario e doppia palette dei puntatori

- Confrontati i ruoli dei puntatori di Windows 11, macOS e i nomi convenzionali Xcursor/Linux.
- Esteso il catalogo AureaOS da 9 a 26 sorgenti SVG: selezione alternativa, aiuto, precisione, divieto, copia, alias, pin, persona, testo verticale, zoom, menu contestuale, penna, ridimensionamento orizzontale/verticale e attività in background, oltre ai ruoli già presenti.
- Creati 26 master SVG per la palette chiara (riempimento bianco e contorno scuro) e 26 varianti scure (riempimento scuro e contorno bianco).
- Aggiunti manifest separati `manifest.toml` e `manifest-dark.toml` con hotspot e nomi tema distinti.
- Il builder ora accetta un manifest alternativo e produce alias Xcursor/Linux comuni (ad esempio `left_ptr`, `hand2`, `xterm`, `watch`, `not-allowed`, `fleur` e le varianti di resize).
- Aggiornata la CI per costruire e conservare entrambi i temi.
- Non eseguiti build o test in questa sessione: validazione automatica e packaging sono demandati alla CI; resta da verificare l'aspetto a schermo e l'integrazione runtime Smithay/Vulkan.
- Decisione UX: palette chiara/scura selezionabile; modalità invertita e colori personalizzati sono una successiva funzione di accessibilità a livello compositor, non un terzo set statico.

### 26 novembre 2026 — Collegamento dei temi Xcursor al renderer Vulkan

- Sostituita la risorsa GPU limitata alla freccia integrata con una cache per dispositivo delle risorse Xcursor nominate, indicizzata per tema, nome del puntatore e dimensione raster richiesta.
- Aggiunta la risoluzione dei temi tramite `AUREA_CURSOR_THEME_ROOT` (predefinita a `/usr/share/icons`), il limite di dimensione dei file, la decodifica Xcursor, la selezione della dimensione più vicina e il caricamento Vulkan di tutti i fotogrammi della dimensione scelta.
- La scena di rendering conserva ora la posizione globale del puntatore. La geometria usa dimensioni e hotspot specifici del cursore; il tema desktop esistente sceglie la palette Light o Dark.
- Se il tema o il cursore non è disponibile, viene usata la freccia AureaOS integrata. Il fallback viene ridimensionato alla dimensione fisica richiesta per mantenere la dimensione logica corretta sugli schermi HiDPI.
- Aggiornato il packaging Debian Live: vengono incluse le sorgenti SVG e gli strumenti del builder, installate temporaneamente le dipendenze `librsvg2-bin` e `xcursorgen`, generati entrambi i temi nel chroot Debian, installati in `/usr/share/icons`, verificati i 26 cursori canonici e rimosse le dipendenze usate solo per la compilazione.
- Aggiornata la documentazione dei puntatori con percorsi runtime, override per lo sviluppo e limitazioni ancora aperte.
- **Non verificato in esecuzione:** non sono stati eseguiti `cargo fmt`, build/check/test Cargo, controlli di sintassi shell, build ISO Debian o prove visive su hardware. Le modifiche sono state accumulate senza build a ogni correzione, come concordato.
- **Da completare:** tutti i fotogrammi vengono caricati nella cache, ma al momento viene disegnato solo il primo. Restano scheduler per le animazioni `busy` / `left_ptr_watch`, caricamento e decodifica fuori dal callback di rendering, revisione della memoria cache multi-monitor, verifica fractional/high-DPI e suite completa su Fedora.

- Correzione successiva: geometria e selezione raster usano ora `RenderTarget::effective_scale()`, mantenendo la scala frazionaria esatta di ogni monitor invece di approssimarla con la vecchia scala intera.
- Correzione successiva: il validatore dello SquashFS finale estrae e controlla entrambi i temi installati e tutti i 52 file cursore canonici (26 per palette), oltre ai controlli già eseguiti nel chroot.
- Le ultime modifiche sorgente non sono ancora state compilate né testate; per il commit documentale non risultano check GitHub.

### 26 novembre 2026 — Decodifica asincrona e fotogrammi Xcursor animati

- Aggiunto un worker dedicato e limitato, `aurea-xcursor-decode`. Controllo dei metadati, lettura dei file, parsing Xcursor e generazione della freccia di fallback avvengono ora fuori dal callback di rendering.
- Il thread di rendering controlla i risultati decodificati, crea gli snapshot RGBA8, invia gli upload Vulkan e controlla le fence. Le texture vengono rese disponibili solo a upload completato; il caricamento dei puntatori non esegue più un'attesa GPU incondizionata.
- Ogni fotogramma conserva dimensioni, hotspot e durata Xcursor. Il renderer seleziona il fotogramma corrente usando un timestamp monotono e collega la relativa texture Vulkan.
- L'event loop mantiene la pianificazione a 16 ms quando è attivo il cursore animato `busy` e durante una finestra di tre secondi al primo caricamento. In questo modo decodifica e upload asincroni possono terminare anche senza ulteriori eventi del puntatore.
- **Ancora da completare:** limiti di memoria e cache LRU tra nomi di cursore e scale dei monitor, comportamento reduced-motion, eventuale animazione di `left_ptr_watch` e verifica visiva completa.
- **Non verificato in esecuzione:** non sono stati eseguiti formatting Cargo, build, check, test, validazione shell o generazione ISO. Continuiamo ad accumulare modifiche prima della verifica prevista su Fedora.

- Correzione successiva: cache delle texture native limitata a 32 risorse / 16 MiB, con espulsione LRU. Una singola animazione è limitata a 4 MiB; se supera il limite viene mantenuto il primo fotogramma, evitando di esaurire il budget GPU condiviso.
- Aggiunti test unitari indipendenti dalla GPU per durate Xcursor, confini esatti dei fotogrammi, ripetizione ciclica e gestione dei ritardi pari a zero.
- Verificato nel manifest che `busy` definisce 12 fotogrammi / 1100 ms, mentre `left_ptr_watch` è attualmente un'immagine statica; per questo viene animato soltanto `busy`.

### 26 novembre 2026 — Posizione iniziale e individuazione del puntatore

- Verificato il percorso di avvio DRM: il server inizializza il puntatore Wayland, pubblica le uscite video e chiama `center_pointer_on_primary_output()` prima di registrare il socket Wayland. L'immagine iniziale è `CursorImageStatus::default_named()`; il renderer nativo dispone già della garanzia di primo fotogramma. L'obiettivo è quindi mostrare la freccia AureaOS al centro del primo monitor attivo, senza posizione casuale.
- La selezione attuale prende la prima uscita attiva nell'ordine del registro; non esiste ancora un'identità persistente del monitor principale scelto dall'utente. Per un singolo monitor è adeguata; per configurazioni multi-monitor complesse introdurremo la selezione esplicita del display principale nelle impostazioni schermo.
- Aggiunti test sorgente per il calcolo del centro, inclusi monitor con origine negativa, dimensioni logiche dispari e geometrie non valide. Non eseguiti.
- Verificato che il rilevamento dello shake esiste già ed è attivo per impostazione predefinita. Usa il movimento relativo non accelerato, conserva cronologie separate per ogni dispositivo fisico e riconosce tre inversioni di direzione entro una finestra limitata di tempo e distanza. L'animazione attuale ingrandisce la freccia fino a 2,4× e la riporta alla dimensione normale in circa un secondo.
- La documentazione ufficiale Apple conferma che la funzione è disponibile sia con mouse sia con trackpad: basta muovere rapidamente il mouse o il dito sul trackpad. Non serve disegnare un cerchio; l'interazione documentata è un movimento rapido avanti e indietro.
- **Ancora da implementare:** la scia visiva. Apple documenta l'ingrandimento temporaneo; la scia sarà un'aggiunta grafica proprietaria AureaOS. La soluzione prevista conserva per poco tempo le posizioni recenti e disegna dietro al puntatore alcune copie traslucide che svaniscono, con supporto a reduced-motion e preferenza per dispositivo.
- Il rilevamento usa il percorso comune di movimento relativo libinput per mouse e trackpad registrati. La compatibilità fisica dei trackpad resta da provare; gli eventi gesture swipe/pinch sono separati e non devono attivare l'effetto.
- Nessuna build, `cargo fmt`, check o test eseguiti.

## 26 novembre 2026 — Fondamenta modulari della scia del puntatore

Aggiunto `input/cursor_trail.rs` per una cronologia limitata a tre posizioni, con validazione delle coordinate e dissolvenza lineare di 220 ms; aggiunto `input/cursor_trail_animation.rs` per isolare la durata dei frame della scia. Registrati entrambi i moduli in `input/mod.rs`. I test unitari coprono limite dei campioni, ordinamento dal più recente, dissolvenza/scadenza e coordinate non valide. La fase di disegno Vulkan non è dichiarata completata: manca ancora il rendering delle impronte con alpha corretto, che richiede un percorso dedicato di texture/shader e l'integrazione nella scena. Nessuna build o test eseguito, rispettando il flusso di verifica differita.

## 26 novembre 2026 — Rendering Vulkan della scia del puntatore

Estesa la scia dalla cronologia degli input fino agli snapshot della scena e alla geometria specifica per ogni uscita video. Il movimento registra posizioni logiche globali mentre l'animazione di localizzazione è attiva; il ciclo eventi continua a richiedere frame finché i campioni sono visibili. Aggiunto `surface_compositor/cursor_trail_geometry.rs`, che riutilizza hotspot, trasformazioni del monitor, scala frazionaria e clipping del cursore nativo. Aggiunti shader dedicato `cursor_trail.frag` e pipeline Vulkan memorizzata per formato; l'opacità viene passata agli shader tramite push constants del fragment stage e le impronte vengono composte prima del cursore corrente. La pipeline delle superfici normali e il rendering dei cursori Wayland dei client restano invariati. Restano da verificare su Fedora: compilazione shader, rustfmt/check/test, blending Vulkan reale e configurazioni multi-monitor. Nessuna build o test eseguito.


### 26 novembre 2026 — Correzione del primo avvio del cursore

Durante la revisione è emerso un problema concreto: la finestra di tre secondi usata per il caricamento asincrono del cursore partiva dalla costruzione del compositor, prima della ricerca DRM, dell'inizializzazione Vulkan e della pubblicazione dei monitor. Su hardware lento poteva scadere prima del primo frame; il desktop veniva quindi disegnato senza freccia e, se il mouse restava fermo, non era garantito un altro frame.

Aggiunto begin_cursor_asset_warmup(), richiamato soltanto dopo la pubblicazione delle uscite e il corretto centraggio del puntatore. La finestra viene così riavviata nel momento giusto e vengono richiesti subito nuovi frame, consentendo al worker Xcursor e all'upload Vulkan di completarsi anche senza movimento del mouse o del trackpad. Modifica sorgente non ancora compilata o verificata su hardware.


### 26 novembre 2026 — Freccia nativa immediata

Il renderer non deve più aspettare la lettura, la decodifica e l'upload del tema Xcursor per mostrare il puntatore. Aggiunta una piccola freccia AureaOS ad alto contrasto, caricata una sola volta per dimensione raster e conservata in una cache limitata a quattro dimensioni. La freccia di base viene disegnata subito mentre il tema viene preparato in background; quando la texture del tema è pronta, sostituisce automaticamente la freccia provvisoria mantenendo coordinate e hotspot. Se il worker non parte o la sua coda è momentaneamente piena, il puntatore di base resta comunque disponibile. L'upload sincrono riguarda soltanto questa piccola immagine; lettura e decodifica Xcursor restano asincrone. Nessuna build, formattazione, test o prova fisica eseguiti.

Aggiunti anche test unitari indipendenti dalla GPU per dimensioni, hotspot, payload RGBA e limiti minimo/massimo della freccia provvisoria. I test sono nel repository ma non sono ancora stati eseguiti.

### Verifica del percorso puntatore → renderer → KMS — 26 novembre 2026

È stata eseguita una revisione del percorso completo del puntatore nativo:

- Il builder DRM pubblica gli output Wayland, centra il puntatore sull'output primario e solo dopo avvia la finestra di warm-up del cursore.
- RenderScene acquisisce la posizione corrente del puntatore Smithay e il ruolo del cursore.
- Per i cursori nominati, la scena trasporta posizione globale logica e nome dell'asset Xcursor. I cursori forniti dalle applicazioni restano invece superfici Wayland con il proprio hotspot.
- Il renderer chiama ensure_native_cursor_texture, poi recupera l'asset tramite native_cursor_asset. Se il tema non è pronto, il fallback integrato viene restituito dalla stessa lookup e arriva effettivamente al binding Vulkan e al draw pass.
- La geometria applica hotspot dell'asset, origine logica del monitor, scala intera/frazionaria, trasformazione e clipping. I test già presenti coprono freccia predefinita, hotspot specifico e puntatore collocato su un altro monitor.
- Il movimento relativo passa dai controlli sui valori non finiti, viene limitato all'unione degli output attivi, aggiornato tramite pointer.motion e pointer.frame, quindi richiede il ridisegno degli output.
- Il ciclo eventi mantiene richieste a 16 ms durante il warm-up iniziale. Il renderer consegna i frame attraverso Vulkan diretto oppure tramite il percorso compatibile Vulkan + readback CPU + GBM/KMS; gli eventi VBlank/page-flip gestiscono il ritiro delle risorse.

**Limite individuato:** il warm-up di tre secondi è una finestra temporizzata, non una notifica di completamento del worker. Se lettura, decodifica o upload Xcursor superano tale finestra e il mouse resta immobile, il fallback continua a essere mostrato fino alla richiesta di un nuovo frame (ad esempio al successivo movimento). La freccia di fallback resta comunque disponibile e il puntatore continua a funzionare. La soluzione corretta, in futuro, è notificare il loop al completamento del worker, non prolungare indefinitamente il rendering ad alta frequenza.

**Verifica:** revisione statica dei sorgenti e dei collegamenti tra i componenti. Non sono stati eseguiti cargo fmt, build, cargo check, test, boot o prove fisiche su DRM/KMS/Vulkan. Il collegamento architetturale è presente; il funzionamento reale sul laptop deve ancora essere verificato.


### Follow-up — Aggiornamento automatico del cursore dopo il caricamento

- Aggiunta una verifica del lavoro pendente nel gestore della texture nativa: resta attiva finché una richiesta di decodifica Xcursor è in coda oppure finché una o più fence di upload Vulkan non risultano completate.
- Lo stato è esposto dalla cache delle pipeline e dal backend Vulkan.
- Il loop DRM considera ora questo lavoro come attività di animazione: richiede nuovi frame ogni 16 ms e torna automaticamente all'intervallo inattivo di 100 ms quando decodifica e upload sono terminati.
- Il meccanismo è comune ai due percorsi: presentazione Vulkan diretta e compatibilità con upload CPU condividono la stessa cache per GPU.
- La freccia integrata di fallback resta disponibile durante l'intera operazione. Nessuna build o prova runtime è stata eseguita in questa modifica sorgente; la verifica è rinviata al controllo cumulativo su Fedora.


## 26 novembre 2026 — Preparazione asset icone desktop

Aggiunto in `VulkanSurfacePipelineCache` il caricamento/preparazione dell'asset di ciascun elemento desktop: risoluzione tramite descrittore canonico (tema, stato del Cestino e densità effettiva dell'output), lettura RGBA8 validata dalla directory `AUREA_HUMAN_INTERFACES_ROOT` (predefinita `/usr/share/aureaos/human-interfaces`), conservazione dello snapshot CPU per chiave e inserimento nella cache texture Vulkan già esistente. Se un elemento non ha ancora un asset approvato, non viene scelta un'icona arbitraria. Attenzione: questo è solo il livello di preparazione. Il collegamento di geometrie, binding e draw nel `surface_compositor`, la resa visibile delle icone e l'evidenziazione della selezione restano da completare. Nessuna build o test eseguiti.


### 26 novembre 2026 — Intervallo doppio clic configurabile

Il tracker del desktop non usa più un intervallo fisso nel confronto: ogni istanza del compositor conserva il proprio valore. Il predefinito resta 400 ms; sono disponibili metodi pubblici per impostarlo e leggerlo in millisecondi. Il valore viene limitato all'intervallo 150–1000 ms. Quando cambia, il primo clic eventualmente in attesa viene azzerato, così non si combinano due clic misurati con impostazioni differenti. La distanza massima resta 4 pixel logici. Aggiunti test sorgente per modifica e limiti del valore. Il collegamento alla schermata Impostazioni e la persistenza della preferenza sono ancora da realizzare. Nessuna build o test eseguiti.

### 26 novembre 2026 — Preferenza globale comportamento puntatore

Aggiunto un contratto e un canale dedicati alle preferenze globali del puntatore, distinti dai profili dei singoli mouse e trackpad. Il canale attraversa Shell, composition root desktop, servizio compositor e ciclo eventi Calloop. Il compositor salva l'intervallo del doppio clic in `desktop-pointer.toml` nella directory di configurazione input di AureaOS, con formato TOML versionato, scrittura atomica e permessi 0600. File malformati o di versione futura vengono preservati e aperti in sola lettura. Il valore salvato viene ripristinato all'avvio; il predefinito è 400 ms e i limiti sono 150–1000 ms. La schermata Impostazioni e la verifica su hardware reale restano da realizzare. Nessuna build o test eseguiti.

### 26 novembre 2026 — Perimetro del blocco mouse e touchpad

- Riesaminato il collegamento delle preferenze live dalla Shell al compositor: mouse e touchpad usano canali distinti e aggiornano i rispettivi dispositivi libinput.
- Confermata la persistenza separata dei profili fisici: pointers.toml per mouse, touchpads.toml per touchpad e desktop-pointer.toml per l'intervallo globale del doppio clic. Gli ID temporanei eventN non vengono usati come identità persistenti.
- Confermato il ripristino dei profili touchpad alla riconnessione e dopo la riattivazione della sessione libseat; l'accelerazione viene riapplicata anche a mouse e touchpad.
- Decisione per AureaOS 1.0: tavolette Wacom e controlli Tablet Pad sono rinviati e non bloccano il completamento del blocco mouse/touchpad. Il percorso touchscreen resta una famiglia input separata.
- L'interfaccia grafica delle preferenze sarà realizzata nella successiva attività Impostazioni di sistema: il backend è predisposto, ma i controlli visibili non sono ancora presenti.
- Non eseguiti cargo fmt, build, cargo check, test o prove fisiche. Il blocco potrà essere dichiarato verificato solo dopo il controllo cumulativo su Fedora e le prove con mouse e touchpad reali.

### 26 novembre 2026 — Test mirati della scia del puntatore

Estratto in una costante nominata l'intervallo minimo di campionamento di 24 ms e aggiunti test unitari per verificare il filtraggio degli eventi ad alta frequenza, l'ordine temporale dei campioni e il reset completo della cronologia dopo `clear()`. Nessuna build o test eseguito: la verifica resta accorpata alla sessione Fedora prevista.

### 26 novembre 2026 — Warm-up cursore anche dopo hotplug e ripresa sessione

Centralizzato l'avvio del warm-up del cursore in `center_pointer_on_primary_output()`. Ora il caricamento iniziale viene riattivato dopo ogni riposizionamento dovuto a pubblicazione/riconnessione dei monitor o ripresa della sessione libseat, non soltanto al primo avvio del server. In questo modo un puntatore fermo continua a ricevere frame mentre viene preparata la texture per il nuovo output. Rimossa la chiamata duplicata dal builder DRM. Modifica sorgente non ancora compilata o collaudata su Fedora.

### 26 novembre 2026 — Verifica del contratto della freccia predefinita

Estesi i test di `RenderScene`: con il cursore predefinito nominato, la scena deve conservare sia la posizione globale del puntatore sia il nome asset canonico `default`, oltre all'origine con hotspot. Con `CursorImageStatus::Hidden`, posizione, asset e rettangolo nativo devono restare assenti. Sono controlli mirati al percorso minimo della freccia, non nuove funzionalità. Test aggiunti ma non eseguiti; nessuna build lanciata.

### 26 novembre 2026 — Diagnostica della disponibilità del puntatore nativo

Aggiunti log `tracing::info` nei due passaggi che servono a diagnosticare il primo avvio su hardware: upload Vulkan completato della freccia integrata di riserva e pubblicazione in cache dell'asset Xcursor del tema, con nome asset, dimensioni raster e numero di fotogrammi. Al collaudo Fedora questi messaggi permetteranno di distinguere un problema di caricamento asset da uno di geometria/presentazione KMS. Nessuna build o test eseguiti.

### 26 novembre 2026 — Geometria della freccia su monitor HiDPI

Aggiunto un test mirato per il posizionamento del cursore nativo su output con scala 2×: verifica conversione coordinate logiche/fisiche, dimensioni raster 64×64 e hotspot scalato. Il test completa i casi già presenti per monitor primario 1× e puntatore collocato su un altro monitor. Non eseguito; nessuna build avviata.


### 26 novembre 2026 — Percorso comune di aggiornamento del puntatore

Separata in `compositor/pointer.rs` la fase finale comune ai movimenti relativi e assoluti: calcolo del focus sulla superficie, aggiornamento Smithay, invio del frame Wayland e richiesta dei frame di output. Mouse/touchpad e dispositivi assoluti mantengono distinti i rispettivi calcoli e vincoli; condividono soltanto il commit della posizione già validata. La modifica riduce duplicazione e impedisce che i due percorsi divergano nella richiesta di ridisegno. Nessuna build o test eseguiti.

### 26 novembre 2026 — Rilascio pulsanti alla disconnessione del mouse/touchpad

- Durante l'audit del ciclo di vita dei dispositivi è emerso che la pausa della sessione sintetizzava correttamente i rilasci, ma la rimozione fisica di un dispositivo eliminava il suo stato dei pulsanti senza prima inviare al seat Wayland il rilascio finale.
- Corretto `disconnect_pointing_device`: prima di cancellare inventario e stato del dispositivo, ora invia i rilasci dei pulsanti ancora premuti usando il timestamp monotono; il `PointerManager` continua a preservare un pulsante se un altro dispositivo fisico lo tiene ancora premuto.
- Questo evita che un'applicazione mantenga un click logicamente premuto dopo lo scollegamento del mouse o del touchpad.
- Commit: `90dec90c`. Nessuna build o test eseguiti: le verifiche restano raggruppate come concordato.

- Esteso il fix anche alla riclassificazione dello stesso dispositivo da parte di libinput: se un sysname viene riannunciato con una classe diversa, il compositor chiude prima il vecchio ciclo di vita, sintetizza eventuali rilasci e azzera stato/gesti prima di registrare la nuova classe. Commit: `a99c0b60`.


### Audit puntatore — correzione del ciclo di vita gesture (26 novembre 2026)

Nel controllo successivo del blocco mouse/touchpad è stato individuato un secondo caso limite: scollegando fisicamente il touchpad proprietario di una gesture swipe, pinch o hold ancora attiva, il compositor cancellava il proprio stato interno ma non inviava ai client Wayland il corrispondente evento di fine gesture annullata. Un client avrebbe potuto conservare una sequenza protocollo incompleta.

Correzione pubblicata su `main`:
- prima di rimuovere il dispositivo, il compositor verifica se è il proprietario della gesture attiva;
- annulla l'accumulatore semantico e invia il relativo evento Wayland di fine con `cancelled = true`;
- non modifica la gesture eventualmente posseduta da un altro dispositivo;
- rifiuta i begin duplicati sulla stessa sequenza, evitando di azzerarne gli accumulatori.

Aggiunti test unitari per il rifiuto dei begin duplicati e per la rimozione limitata al dispositivo proprietario. Modifiche sorgente non ancora compilate né eseguite; formattazione, build e test restano da effettuare in un unico passaggio concordato.

### Audit puntatore — protezione avvio gesti (26 novembre 2026)

- Gli handler di inizio swipe, pinch e hold ora verificano prima la disponibilità del puntatore Wayland e solo dopo acquisiscono la proprietà del gesto e inizializzano lo stato semantico. Evitato così uno stato orfano in caso di seat senza puntatore.
- Corretto un test rimasto agganciato al vecchio metodo `remove_device`: ora verifica `take_device`, coerentemente con l'API effettiva del proprietario dei gesti.
- Modifiche sorgente e documentazione registrate su `main`. Nessuna build o suite eseguita in questa fase: la validazione Rust resta da eseguire in un unico blocco, come concordato.

- Audit configurazione touchpad: corretto il report della modalità `NoScroll`. In caso di rifiuto o errore da libinput, le opzioni di scorrimento non vengono più dichiarate erroneamente applicate. Il risultato effettivo di libinput viene propagato al report.

- Audit accelerazione mouse/touchpad: eliminata la doppia applicazione dei profili. Il mouse usa `PointerSettings`; il touchpad usa esclusivamente il proprio `TouchpadSettings` per velocità e curva. La vecchia applicazione generica poteva sovrascrivere il profilo touchpad all'avvio, al movimento o alla riattivazione della sessione.
- I profili generici sono ora accettati e ripristinati solo per i mouse; aggiunte asserzioni per impedire l'assegnazione di un profilo mouse a un touchpad.

- Audit ciclo di vita dispositivi: la nuova registrazione dello stesso sysname non viene più considerata la stessa periferica soltanto perché conserva la medesima classificazione. Se cambiano identificativi hardware stabili già noti (persistent ID, vendor ID o product ID), il compositor chiude il vecchio ciclo di vita: rilascia i pulsanti, cancella gli stati input associati e poi registra il nuovo dispositivo. I campi mancanti non provocano da soli un reset.
- Restano da fare la validazione Rust raggruppata e la prova fisica di scollegamento/ricollegamento.


### 26 novembre 2026 — Precisione del report delle opzioni di scorrimento touchpad

Durante l'audit è emerso che, selezionando lo scorrimento a due dita, il report dichiarava anche lo scorrimento ai bordi come "Applied" (e viceversa), pur essendo le due modalità mutuamente esclusive. Aggiunto lo stato API `NotRequested` per distinguere un'opzione volutamente inattiva da una capacità hardware non supportata. `all_applied()` considera correttamente riuscite le opzioni non richieste, mentre continua a segnalare come incomplete quelle non supportate. Aggiunto un test unitario sul contratto del report. Modifiche sorgente non ancora formattate, compilate o testate; verifica rinviata al passaggio cumulativo su Fedora.


### 26 novembre 2026 — Report corretto per la modalità touchpad senza scorrimento

Completata la distinzione dei tre casi mutuamente esclusivi: con `NoScroll` né lo scorrimento a due dita né quello dai bordi risultano richiesti, quindi entrambi vengono riportati come `NotRequested` invece che come `Applied`. La selezione di `DisabledOnExternalMouse` resta delegata a libinput tramite `config_send_events_set_mode`, senza introdurre un secondo monitoraggio concorrente dei mouse esterni. Modifica sorgente e documentazione non ancora formattate, compilate o testate.

### 26 novembre 2026 — Semplificazione dello stack mouse/touchpad

Riaffermato il confine architetturale di AureaOS: non riscrivere ciò che il kernel
Debian, udev, libinput e Smithay già forniscono. I driver kernel gestiscono il
trasporto hardware; libinput gestisce classificazione, quirks, normalizzazione,
accelerazione, configurazione touchpad e riconoscimento dei gesti supportati;
Smithay integra il flusso nel compositor Wayland. AureaOS mantiene la proprietà
delle preferenze esposte, delle azioni desktop e dell'esperienza utente.

Corretto anche il comportamento di DeviceDefault: quando l'API espone il valore
predefinito del dispositivo, il compositor lo ripristina tramite libinput;
quando non è disponibile un valore ripristinabile, il report usa NotRequested.
all_applied() considera soddisfatte le opzioni non richieste, senza nascondere
capacità non supportate o errori reali.

Nessuna build o test eseguiti: la verifica resta accorpata al controllo Fedora
concordato.

### 26 novembre 2026 — Secondo passaggio di semplificazione mouse/touchpad

Controllati gli usi effettivi del registro dispositivi, delle capability e del report
di configurazione. Il registro e il manager restano necessari: conservano inventario
per la sessione, preferenze per periferica, notifiche ai client e soprattutto
l'aggregazione dei pulsanti fisici sul singolo seat Wayland. Quest'ultima non è una
duplicazione di libinput: serve a mantenere coerente lo stato condiviso del seat
quando più dispositivi tengono premuto lo stesso pulsante o uno viene scollegato.

Rimossa da AppliedTouchpadSettings l'operazione invalidate(), identica a remove()
e priva di un uso runtime distinto. Un solo metodo gestisce ora la rimozione della
cache dei profili applicati. Conservati capability snapshot e report perché sono
dati reali letti/applicati tramite libinput e costituiscono il contratto per le
future Impostazioni; non viene introdotto un secondo sistema di rilevamento.

Nessuna build o test eseguiti. La validazione Rust resta accorpata come concordato.


### 26 novembre 2026 — Esito esplicito della modalità di scorrimento

Il controllo del report ha evidenziato che `NoScroll` veniva inviato a libinput,
ma il suo esito non era conservato: le due opzioni mutuamente esclusive risultavano
correttamente `NotRequested`, però un eventuale rifiuto della modalità scelta
poteva restare nascosto. Aggiunto `scroll_method` a `TouchpadApplyReport`, così
il report mantiene separati lo stato dei controlli UI e l'esito effettivo della
modalità selezionata. `all_applied()` e `has_failures()` considerano anche questo
campo. Aggiunti test del contratto API; nessuna build o test eseguiti ancora.
