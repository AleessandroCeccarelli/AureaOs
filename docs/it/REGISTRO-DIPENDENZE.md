<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Registro delle dipendenze AureaOS

## Scopo e autorità

Questo è il registro ingegneristico obbligatorio per librerie esterne, pacchetti di sistema, crate Rust, strumenti e servizi runtime utilizzati o valutati da AureaOS.

**Debian GNU/Linux 13 (Trixie), amd64, è la piattaforma di destinazione e l'autorità di compatibilità. Fedora è esclusivamente l'ambiente di sviluppo.** La disponibilità su Fedora è utile, ma non costituisce prova sufficiente per accettare una dipendenza.

## Ordine di selezione obbligatorio

1. Confermare disponibilità e supportabilità su Debian 13 Trixie amd64.
2. Registrare nomi e versioni esatte dei pacchetti binari e sorgente Debian.
3. Confermare la disponibilità, nei repository Debian supportati, di header, librerie, strumenti e componenti runtime necessari.
4. Verificare separatamente i pacchetti di sviluppo Fedora. API, percorsi o presupposti di packaging esclusivi Fedora non sono accettabili.
5. Esaminare manutenzione upstream, cronologia release, advisory di sicurezza, licenza, stabilità ABI, architetture supportate e footprint transitivo.
6. Preferire infrastrutture Linux esistenti o componenti AureaOS già approvati quando soddisfano il requisito.
7. Registrare qui decisione ed evidenze prima di aggiungere la dipendenza al codice di produzione o al packaging.

Una dipendenza non è approvata solo perché compare in Cargo.toml, in un documento, prototipo, immagine CI o macchina dello sviluppatore.

## Stati

- **Adottata** — scelta architetturale; non implica integrazione completa o validazione runtime.
- **Approvata** — verificata per l'ambito indicato e autorizzata all'implementazione.
- **Candidata** — promettente, ma con verifiche aperte; non integrarla ancora.
- **Condizionata** — utilizzabile solo dopo aver soddisfatto le condizioni indicate.
- **Respinta** — non accettata per l'ambito indicato.
- **Da riesaminare** — dipendenza/integrazione esistente con evidenze incomplete o da rivalutare.

## Campi obbligatori per ogni voce

| Campo | Informazioni richieste |
|---|---|
| Nome e scopo | Progetto/libreria/pacchetto esatto e funzionalità AureaOS |
| Tipo | Pacchetto Debian, crate Rust, servizio di sistema, tool build, protocollo o tool asset |
| Stato e ambito | Stato sopra indicato, componente e funzionalità |
| Destinazione Debian | Pacchetti Debian 13, pacchetto sorgente, versione e repository/componente, se verificati |
| Sviluppo Fedora | Pacchetti Fedora e versione, se verificati; altrimenti Non verificato |
| Integrazione Rust | Crate/versione, feature, MSRV, linking nativo/bindgen, se pertinenti |
| ABI e architettura | Versione ABI/API, linking dinamico/statico, supporto amd64 e vincoli |
| Licenza | Identificativo SPDX e verifica compatibilità, incluse licenze transitive rilevanti |
| Manutenzione e sicurezza | Attività upstream, supporto, advisory e mitigazioni |
| Footprint | Dipendenze transitive runtime/build, servizi, impatto disco/memoria |
| Evidenze | Metadati ufficiali, documentazione upstream, fonti, data e versioni esatte |
| Decisione | Necessità, alternative, vincoli e data approvazione |
| Validazione | Cosa è stato realmente compilato/testato su Debian e Fedora; disponibilità pacchetto non è validazione |

I valori sconosciuti devono essere indicati come **Non verificato**. Non colmare le lacune con supposizioni.

## Baseline delle piattaforme

| Tecnologia | Ruolo AureaOS | Destinazione Debian | Sviluppo Fedora | Stato |
|---|---|---|---|---|
| Debian 13 Trixie amd64 | Base distribuzione e target runtime | Piattaforma di destinazione | Non applicabile | Adottata |
| Toolchain Rust | Linguaggio per implementazione proprietaria | Politica esatta pacchetti/toolchain da registrare | Toolchain sviluppo | Adottata; politica versioni da riesaminare |
| Wayland | Protocollo desktop/modello display | Pacchetti protocollo/librerie supportati Debian | Verificare pacchetti sviluppo per funzionalità | Adottata |
| Smithay 0.7 | Framework compositor Wayland | Crate risolto da Cargo; dipendenze native compatibili Debian | Solo host sviluppo | Adottata; revisione sicurezza transitiva aperta |
| Vulkan | API esclusiva rendering compositor | Loader/driver/toolchain Debian per GPU target | Stack GPU Fedora solo per sviluppo | Adottata |
| DRM/KMS, udev | Interfacce kernel display/dispositivi | Kernel e integrazione userspace Debian | Host sviluppo | Adottate |
| libinput | Gestione dispositivi input | Pacchetto/versione e funzionalità da verificare | Pacchetto sviluppo Fedora da identificare | Adottata architetturalmente; scheda incompleta |
| libseat | Accesso sessione/dispositivi | Pacchetto/versione/backend Debian da verificare | Pacchetto sviluppo Fedora da identificare | Adottata architetturalmente; scheda incompleta |
| calloop | Integrazione event loop | Crate e dipendenze native da verificare | Ambiente Cargo sviluppo | Adottata |
| systemd | Init, servizi e integrazione sistema | systemd Debian 13 | Non è autorità runtime | Adottato |
| PipeWire | Infrastruttura multimediale | Pacchetti/moduli Debian 13 per funzionalità da verificare | Pacchetti sviluppo Fedora | Adottata nella direzione tecnica; revisione per funzionalità |
| NetworkManager | Gestione rete | Pacchetto/servizio Debian 13 | Pacchetto sviluppo Fedora | Adottato nella direzione tecnica |
| Flatpak / Flathub | Distribuzione app terze parti | Compatibilità runtime e portal Debian 13 | Fedora non prova compatibilità | Adottati nella direzione prodotto; revisione app per app |

La tabella registra scelte architetturali, non dimostra che ogni pacchetto sia installato, integrato o validato.

## Decisioni per componente

### Decodifica JPEG — libjpeg-turbo / API di compatibilità TurboJPEG

- **Stato:** Candidata — un adapter mirato verso la libreria nativa è l'opzione principale, ma non è approvata per l'implementazione finché non supererà la verifica ABI/simboli e la revisione del confine del decoder su entrambe le distribuzioni.
- **Scopo:** decodificare il wallpaper JPEG iniziale in pixel RGBA8 con limiti, senza introdurre uno stack immagini generalista.
- **Evidenza Debian verificata:** Debian 13 Trixie amd64 fornisce `libturbojpeg0` e `libturbojpeg0-dev`, versione `1:2.1.5-4`. Il pacchetto di sviluppo dipende dal corrispondente pacchetto runtime.
- **Evidenza Fedora:** Fedora fornisce `libjpeg-turbo-devel` e `turbojpeg-devel`; i flussi esaminati usano libjpeg-turbo 3.1.x. La build esatta del pacchetto installato su Fedora deve ancora essere verificata sulla macchina dell'utente.
- **Verifica compatibilità:** le note di rilascio upstream di libjpeg-turbo 3.x descrivono esplicitamente un wrapper di compatibilità TurboJPEG 2.x. Questo rende plausibile un piccolo adapter basato sull'API legacy tra Debian 2.1 e Fedora 3.1, ma simboli esportati, dichiarazioni e linking runtime devono essere verificati sugli header e sulle librerie effettive di entrambe le distribuzioni prima dell'approvazione.
- **Decisione sui binding Rust:** non usare il percorso attualmente esaminato dei crate `turbojpeg` / `turbojpeg-sys`: i binding esaminati espongono l'API TurboJPEG 3.0 `tj3*` e non si può presumere la compatibilità con la libreria di sistema 2.1 di Debian. Non introdurre bindgen né un crate immagini generalista per questo singolo decoder.
- **Confine d'integrazione proposto:** piccolo adapter Rust di proprietà AureaOS, con tutto l'FFI unsafe isolato in un modulo; esporre soltanto un contratto sicuro di decodifica che restituisca l'attuale `WallpaperPixels` con limiti. Decodificare su un worker limitato, mai nel render thread. Validare dimensione dell'input compresso, dimensioni decodificate, allocazione RGBA con controlli overflow, numero massimo di pixel e budget totale di memoria prima dell'allocazione.
- **Alternative esaminate:** `zune-jpeg` 0.5.15 è bloccato perché upstream segnala colori errati per JPEG baseline validi; la correzione individuata è soltanto nella 0.5.16-rc3, una pre-release. `jpeg-decoder` 0.3.2 è Rust puro ma upstream lo descrive in modalità manutenzione; Debian Trixie pacchettizza la 0.3.0. Nessuna delle due è attualmente selezionata come percorso primario.
- **Verifiche ancora necessarie per l'approvazione:** verificare API legacy, simboli esatti e ABI su Debian 13 e Fedora; confermare l'inclusione del pacchetto runtime nell'immagine target; riesaminare advisory di sicurezza correnti e avvisi di licenza BSD-style di libjpeg-turbo; definire contenimento errori/panic e cancellazione del worker; testare input baseline/progressive, grayscale, troncati e malformati, oltre al wallpaper incluso nel repository.
- **Validazione:** questa modifica al registro non ha installato né collegato la libreria nativa. Nessun manifest Cargo o lockfile è stato modificato. Non sono stati eseguiti build, format, check o test.
### Rasterizzazione SVG — librsvg

- **Stato:** Respinta per l'attuale ambito runtime del desktop core.
- **Motivo:** la catena include Cairo, GLib, Pango e componenti correlati, sproporzionati per l'attuale requisito di icone statiche.
- **Direzione accettata:** mantenere gli SVG master modificabili in Human Interfaces e generare asset raster limitati tramite toolchain build-time documentata e riproducibile. Documentare provenienza e licenze.
- **Riesame:** solo per un requisito SVG runtime concreto e una nuova valutazione Debian-first del footprint.

### SQLite

- **Stato:** Candidata per catalogo locale/metadati ricerca; non approvata.
- **Evidenza Debian registrata:** Debian 13 fornisce pacchetti runtime e development SQLite.
- **Verifiche aperte:** FTS5 nella build target Debian; binding Rust; definizione MSRV; versioni Debian/Fedora, comportamento WAL/recupero, licenza e manutenzione.
- **Vincolo:** non aggiungere un database per indicizzazione ipotetica. Scansione e riconciliazione filesystem restano autorevoli dove applicabile.

### Notifiche filesystem — notify / inotify nativo

- **Stato:** Condizionata; nessun crate selezionato.
- **Evidenza registrata:** notify 9 documenta supporto Linux inotify e licenza CC0, ma la MSRV attuale è Rust 1.91. I manifest workspace esaminati usano edition 2024 e non dichiarano rust-version.
- **Prima dell'approvazione:** definire MSRV AureaOS; verificare compatibilità toolchain Debian; confrontare crate e piccola integrazione inotify nativa; documentare overflow/riconciliazione.
- **Vincolo:** le notifiche sono suggerimenti a bassa latenza, non sostituiscono scansioni e riconciliazione autorevoli.

### Passkey / FIDO2 / WebAuthn

- **Stato:** Da riesaminare a livello architetturale/dipendenze; nessun crate selezionato.
- **Evidenza registrata:** passkey-rs fornisce componenti WebAuthn client e CTAP2 authenticator; le librerie FIDO Linux hanno trade-off differenti per trasporto e licenza.
- **Prima dell'approvazione:** definire se servono client, authenticator, trasporto CTAP o tutti; valutare pacchetti Debian e accesso hardware/udev; esaminare licenza, manutenzione, sicurezza, opzionalità TPM e confini privilegi.
- **Vincolo:** non presumere che tutte le passkey siano memorizzate in un TPM. Non introdurre servizi privilegiati o librerie native prima di definire confini sicurezza e IPC.

### Sicurezza dipendenze Rust — cgmath tramite Smithay

- **Stato:** Da riesaminare; criticità manutenzione/sicurezza tracciata.
- **Evidenza registrata:** Smithay 0.7.0 introduce attualmente cgmath 0.18.0, associato agli advisory RustSec RUSTSEC-2026-0196 (non mantenuto) e RUSTSEC-2026-0197 (unsound).
- **Azione richiesta:** rivalutare il grafo bloccato esatto e la soluzione upstream prima di cambiare versioni o applicare override. È stato osservato che Smithay master upstream sostituisce cgmath con glam, ma master non è automaticamente una scelta sicura o compatibile.
- **Vincolo:** un job audit verde con warning esplicitamente consentiti non equivale a un report dipendenze pulito.

## Checklist di approvazione

- [ ] Debian 13 Trixie amd64 fornisce il pacchetto o un percorso di compilazione sorgente supportabile.
- [ ] Versioni esatte pacchetto e upstream registrate.
- [ ] Pacchetti Fedora di sviluppo identificati separatamente.
- [ ] Nessuna API, percorso o assunzione packaging esclusiva Fedora.
- [ ] ABI/API, linking e packaging runtime compresi.
- [ ] Licenza e licenze transitive rilevanti compatibili con la distribuzione AureaOS.
- [ ] Manutenzione upstream e advisory di sicurezza esaminati.
- [ ] Vincoli MSRV, edition e toolchain Rust espliciti.
- [ ] Footprint transitivo e runtime giustificato.
- [ ] Infrastrutture Linux esistenti e componenti approvati considerati.
- [ ] Decisione ed evidenze registrate qui.
- [ ] Dichiarazioni di validazione coerenti con test realmente eseguiti.

Se una risposta richiesta è sconosciuta, mantenere la dipendenza come Candidata, Condizionata o Da riesaminare.

## Procedura di modifica

1. Aggiornare il registro prima o nello stesso cambiamento che introduce la dipendenza.
2. Aggiornare entrambi i registri, inglese e italiano.
3. Registrare motivazione, versioni esatte, nomi pacchetto, fonti e data.
4. Aggiornare il documento architetturale pertinente e il diario quando cambia la direzione del progetto.
5. Non dichiarare compatibilità Debian basandosi solo su compilazione Fedora o CI Ubuntu.
6. Non eseguire build, formattazione, test o CI senza approvazione esplicita dell'utente secondo le regole correnti.

## Manutenzione

Rivedere il registro quando viene proposta/aggiunta/rimossa/aggiornata una dipendenza; cambiano versioni o sicurezza dei pacchetti Debian stable; cambia la politica MSRV/toolchain Rust; un advisory interessa una dipendenza diretta/transitiva; oppure un componente modifica ABI nativa, linking o requisiti runtime.

Questo registro è la fonte condivisa di verità per le decisioni sulle dipendenze. Se la comodità d'implementazione entra in conflitto con la compatibilità Debian, prevale la compatibilità Debian.

## Strategia desktop universale e multimedialità — 24 novembre 2026

### Direzione del prodotto
- AureaOS è destinato a bambini, anziani, utenti alle prime armi, studenti, professionisti creativi e d'ufficio, amministratori e sviluppatori attraverso un unico desktop coerente; niente edizioni basate sull'età o supposizioni sulle capacità.
- Principio adottato: semplice per impostazione predefinita, potente quando serve, accessibile fin dall'inizio.
- Accessibilità, multimedialità e produttività sono requisiti fondamentali, non rifiniture successive al rilascio.

### Valutazioni tecnologiche registrate (nessuna dipendenza aggiunta)
- PipeWire + WirePlumber: base raccomandata per instradamento audio/video e politiche di sessione. Evidenze pacchetti Debian Trixie esaminate: PipeWire 1.4.2 e WirePlumber 0.5.8. Inclusione nell'immagine e integrazione runtime restano da verificare.
- GStreamer 1.x: framework modulare raccomandato per pipeline di riproduzione, acquisizione, codifica, muxing e streaming. Evidenze pacchetti Debian Trixie esaminate: famiglia 1.26.2. Licenze e disponibilità di codec/plugin vanno valutate per singola funzionalità; non installare tutti i gruppi plugin per impostazione predefinita.
- XDG Desktop Portal: confine raccomandato per interoperabilità applicativa e permessi Flatpak. Evidenza pacchetto Debian Trixie esaminata: xdg-desktop-portal 1.20.3. AureaOS deve implementare un proprio backend per le interfacce supportate, senza dipendere dai backend desktop GNOME/KDE.
- AT-SPI2: obiettivo di interoperabilità con le tecnologie assistive Linux. Orca è un bersaglio concreto per la validazione con screen reader. AccessKit è un candidato livello di astrazione Rust per l'accessibilità dei widget nativi, non sostituisce AT-SPI2 né costituisce un toolkit completo.
- Parley e cosmic-text: candidati come componenti Rust per layout testuale, shaping, bidi, interruzione righe e fallback dei font. Nessuno dei due è un toolkit UI completo; occorre prototipare prima della scelta.
- Il toolkit UI nativo resta da decidere. Ogni candidato deve dimostrare funzionamento Wayland-only, rendering compatibile Vulkan, accessibilità AT-SPI2/AccessKit, IME, testo complesso, touch, scaling, prestazioni e packaging Debian.

### Confini architetturali
- Mantenere separate le responsabilità Aurea Media e Aurea Capture: il decode immagini restituisce pixel CPU-owned con limiti; Capture coordina autorizzazioni e ciclo sessione; PipeWire trasporta gli stream; GStreamer gestisce pipeline/codifica selezionate; il compositor possiede soltanto i frame finali autorizzati e la presentazione Vulkan.
- Non inserire codec, muxing, mix audio o scrittura file nel compositor/render thread.
- Non introdurre un daemon multimediale monolitico. Iniziare con librerie e adapter; rivalutare un helper process ristretto per isolare media non fidati quando lo giustificheranno threat model e numero di client.
- Le tecnologie nuove o immature possono essere prototipate in crate/processi/feature isolati, ma non diventano dipendenze base o di boot/sessione prima di aver dimostrato beneficio misurabile, sicurezza, compatibilità Debian e possibilità di rollback.

### Documentazione
- Creato docs/architecture/UNIVERSAL-DESKTOP-TECHNOLOGY-STRATEGY.md.
- Allineati i documenti di visione e principi tecnologici in inglese e italiano; aggiunti i collegamenti nei due README.
- Nessuna dipendenza, manifest Cargo, lockfile, sorgente Rust o pacchetto di sistema modificato. Nessun build, format, check, test o CI eseguito.

### LocalSend Protocol v2.x — interoperabilità trasferimento tra dispositivi

- **Stato:** protocollo candidato; nessuna dipendenza o implementazione approvata.
- **Scopo:** trasferimento locale di file, foto, audio, video, link e testo tra AureaOS e client Android/iOS/macOS/altri desktop.
- **Evidenze esaminate:** il repository della specifica pubblica documenta LocalSend v2.2, discovery REST e flussi HTTPS di upload/download, approvazione del destinatario, token per file e verifica SHA-256 opzionale. Il progetto LocalSend documenta client multipiattaforma Linux, macOS, Windows, Android e iOS.
- **Uso proposto:** adapter Rust ristretto dietro API stabile Aurea Link. Preferire HTTPS app-to-app autenticato; discovery non equivale ad autenticazione. Prevedere pairing manuale IP/QR quando multicast è bloccato. Trasferire file grandi con buffer limitati e storage temporaneo sicuro.
- **Riutilizzo codice:** l'interoperabilità del protocollo non autorizza a copiare o includere l'app LocalSend. Prima del riuso verificare licenza attuale della specifica, manutenzione e sicurezza upstream; registrare versione esatta, licenza e dipendenze transitive di ogni crate.
- **Gate sicurezza:** pairing/consenso espliciti, validazione identità peer, nessun downgrade HTTP silenzioso, validazione nomi/percorso, limiti dimensione/sessioni/frequenza, verifica hash, destinazione senza collisioni, pubblicazione atomica, annullamento e pulizia.
- **Debian/Fedora:** nessun pacchetto o crate selezionato. Verificare disponibilità Debian 13 e requisiti reali Rust/toolchain/runtime prima dell'implementazione. La sola verifica Fedora non basta.
- **Alternative:** KDE Connect è target opzionale d'interoperabilità; AirDrop/AWDL/OWL resta sperimentale e dipendente dall'hardware, non requisito di rilascio.
- **Vincoli:** niente daemon sempre attivo, relay cloud, sincronizzazione messaggi/contatti/notifiche o streaming media live nel primo traguardo. Il trasferimento resta fuori da compositor, render thread e decoder Aurea Media.
- **Riferimenti:** https://github.com/localsend/protocol e https://github.com/localsend/localsend
- **Verifiche:** nessuna dipendenza, modifica Cargo o lockfile; nessun build o test.

### API dei servizi online/social — provider candidati

- **Stato:** sola ricerca. Nessun SDK, client OAuth o adapter provider approvato.
- **Scopo:** condividere file/media/testo locali da Fides e dai visualizzatori AureaOS senza riselezionare manualmente il file.
- **Servizi candidati:** Telegram, WhatsApp, Facebook, Instagram, Reddit, YouTube ed email.
- **Distinzione importante:** le API ufficiali differiscono per tipo account e azione. Telegram Bot API non è un'API per account personali; WhatsApp Cloud API è messaggistica business; Facebook Graph pubblica sulle Pagine; Instagram pubblica su account Professional idonei; Reddit richiede OAuth/accesso approvato e permesso scritto per uso commerciale; YouTube Data API carica sul canale autenticato.
- **Fallback:** API ufficiale diretta solo dove consentita → share/deep link ufficiale → handoff ad applicazione desktop registrata → non supportato (mai endpoint privati/scraping).
- **Sicurezza:** OAuth/PKCE dove supportato, privilegi minimi, credential store di sistema, revoca, conferma finale utente, nessun secret confidenziale nei binari, nessuna pubblicazione silenziosa.
- **Riferimenti:** documentazione provider ufficiale registrata in docs/it/ARCHITETTURA-AUREA-SHARE-ONLINE.md.
- **Compatibilità:** base Debian 13; nessuna dipendenza GNOME/KDE, X11/XWayland o OpenGL. Gli adapter API restano fuori da compositor e pipeline media.
- **Verifiche:** nessuna dipendenza, sorgente, registrazione OAuth o test aggiunti.


## Prerequisiti nativi di build del compositor — audit Debian preliminare (24 novembre 2026)

### Compilatore shader

- **Strumento:** `glslangValidator`, invocato da `components/aurea-compositor/build.rs` quando è attiva la feature `drm`.
- **Evidenza Debian 13:** `glslang-tools` è disponibile per Trixie amd64, versione `15.1.0+1.4.309.0-1`; Debian lo identifica come il pacchetto che contiene gli strumenti validator.
- **Pacchetto Fedora di sviluppo:** `glslang`.
- **Ambito:** solo build-time. Non aggiungerlo ai pacchetti runtime del desktop, salvo una distinta necessità runtime.
- **Stato:** disponibilità del pacchetto Debian verificata; build Debian non ancora eseguita.
- **Evidenza:** https://packages.debian.org/trixie/amd64/glslang-tools

### Librerie native di sviluppo Smithay / DRM / input

- **Integrazione nota:** Smithay 0.7 abilita `backend_drm`, `backend_gbm`, `backend_libinput`, `backend_udev`, `backend_session_libseat` e `backend_vulkan` nella feature `drm` del compositor.
- **Evidenza pacchetti Debian:** `libinput-dev` esiste in Trixie e dipende da `libinput10`, `libudev-dev`, `libmtdev-dev` e `libwacom-dev`. Il sorgente wlroots di Trixie elenca inoltre `libgbm-dev`, `libseat-dev`, `libvulkan-dev` e `libwayland-dev` tra le proprie dipendenze di build.
- **Stato:** solo insieme candidato di pacchetti nativi di sviluppo; i requisiti diretti esatti per Smithay 0.7 e i moduli pkg-config necessari devono essere confermati dal grafo Cargo bloccato e da una build Debian. Non copiare alla cieca l'intero elenco wlroots.
- **Separazione runtime:** header e strumenti shader di sviluppo devono restare fuori dall'immagine runtime, salvo una necessità specifica.
- **Evidenza:** https://packages.debian.org/en/stable/libdevel/libinput-dev e https://packages.debian.org/it/source/trixie/wlroots
- **Verifica:** nessuna build Debian del compositor è stata ancora eseguita; nessun pacchetto è stato aggiunto ai manifest runtime.


## rsvg-convert — generazione di asset SVG statici

- **Tipo:** CLI di generazione asset, solo host di sviluppo; riutilizza il renderer già richiesto dal builder del tema cursore.
- **Stato:** Candidato per export desktop; non è una dipendenza runtime AureaOS.
- **Debian 13 Trixie:** `librsvg2-bin`, versione `2.60.0+dfsg-1+deb13u1`; fornisce `rsvg-convert`.
- **Fedora 45:** `librsvg2-tools`, versione `2.63.2-2.fc45`; fornisce `rsvg-convert`.
- **Footprint:** il renderer host dipende da librsvg e componenti Cairo/GLib/Pango e altre librerie. Questo footprint resta confinato all'host di sviluppo/generazione; nessuna libreria SVG viene aggiunta al runtime AureaOS.
- **Licenza:** librsvg LGPL-2.1-or-later con licenze di dipendenze elencate dai pacchetti; revisione necessaria solo se in futuro si decidesse di distribuire il tool o librerie.
- **Uso:** rasterizzazione offline delle sei SVG Trash approvate; la versione effettiva del renderer viene registrata nel manifest export. Il tool non viene linkato dal compositor.
- **Evidenze:** metadati Debian Trixie e Fedora 45 verificati il 24 novembre 2026.
- **Validazione:** tool già usato dal generatore del tema cursore; nuova pipeline Trash non ancora eseguita e output non ancora ispezionati. Nessuna dipendenza runtime aggiunta.


### Asset generator: Python standard library

- `tools/assets/png_to_rgba.py` uses only Python 3 standard-library modules (`argparse`, `struct`, `zlib`, `pathlib`); no pip package or OS runtime library is required.
- **Classificazione:** host-only asset-generation tool; non entra nei manifest Cargo né nell'immagine AureaOS.
- **Limiti:** PNG non interlacciato RGBA8, massimo 256 × 256, verifica CRC, decompressione limitata, conversione filtri 0–4.
- **Stato:** sorgente aggiunto; non ancora eseguito o validato sul laptop.


## Aurea Media — formati immagine statici per gli sfondi (24 novembre 2026)

### Crate Rust image

- **Stato:** Candidato; non ancora aggiunto ai manifest Cargo.
- **Scopo:** adapter di decodifica per PNG, BMP, GIF, TIFF e WebP; JPEG può restare temporaneamente sul backend TurboJPEG esistente durante la migrazione.
- **Upstream candidato:** image 0.25.x. Le feature documentate includono BMP, GIF, JPEG, PNG, TIFF e WebP. Usare default-features = false e abilitare soltanto i formati necessari; non ereditare la feature Rayon predefinita né codec estranei.
- **AVIF:** non presumere che la feature avif predefinita abiliti la decodifica. Il percorso nativo documentato per AVIF usa avif-native e libdav1d; va confrontato con libheif prima della scelta.
- **Debian 13:** l'integrazione codec in puro Rust dovrebbe evitare pacchetti codec di sistema per questi formati, ma grafo Cargo bloccato, MSRV, licenze esatte e comportamento di build sono Non verificati.
- **Fedora sviluppo:** dipendenza Cargo; build host non eseguita.
- **Licenza:** il progetto upstream è dual-licensed; i metadati Cargo della versione esatta e tutte le licenze transitive abilitate vanno controllati prima dell'approvazione.
- **Sicurezza/limiti:** dimensioni e allocazioni del decoder vanno limitate prima dell'allocazione dei pixel; il solo limite sui byte compressi non basta.
- **Decisione:** solo candidato. Nessuna modifica a sorgenti, manifest o lockfile autorizzata da questa voce.

### libheif e plugin decoder immagini

- **Stato:** candidato per decodifica di immagini HEIC/HEIF e AVIF; non ancora collegato ad AureaOS.
- **Scopo:** parsing del contenitore HEIF ISO-BMFF e dispatch ai codec per immagini statiche. HEIC usa codifica immagine HEVC; AVIF usa codifica immagine AV1. Questa voce non autorizza riproduzione video.
- **Debian 13 Trixie:** i metadati confermano libheif1 e libheif-dev; i metadati correnti esaminati riportano 1.23.6-2~deb13u1 per il runtime. La versione amd64 esatta va ricontrollata al momento dell'integrazione/packaging perché gli aggiornamenti di sicurezza possono cambiarla.
- **Plugin HEIC:** libheif-plugin-libde265, con dipendenza runtime libde265-0. Debian lo descrive come plugin per decodificare immagini HEVC.
- **Plugin AVIF:** scegliere un plugin decoder AV1, preferibilmente libheif-plugin-dav1d oppure libheif-plugin-aomdec, dopo verifica del pacchetto Trixie esatto e delle licenze/footprint transitive.
- **Fedora sviluppo:** nomi e versioni dei pacchetti non ancora verificati.
- **Integrazione Rust:** nessun wrapper Rust selezionato. Preferire un piccolo shim ABI C sottoposto ad audit oppure un wrapper mantenuto e ristretto; mantenere handle nativi e codice unsafe dentro l'adapter.
- **Licenza:** combinazione delle licenze libheif/plugin/codec non ancora completamente verificata rispetto agli obblighi di distribuzione GPL-3.0-or-later. Non approvato finché la revisione non è conclusa.
- **Footprint runtime:** installare soltanto i plugin decoder necessari. Non installare libheif-plugins-all, plugin encoder o plugin FFmpeg per questa funzione.
- **Sicurezza:** decodifica di file non fidati fuori dal render thread; limiti su dimensioni, pixel, byte di output e numero di item del contenitore; verifica advisory upstream e ABI supportata.
- **Evidenza:** metadati e descrizioni pacchetti Debian Trixie per libheif1 e libheif-plugin-libde265; https://packages.debian.org/trixie/libheif1 e https://packages.debian.org/trixie/libheif-plugin-libde265
- **Decisione:** implementazione opzionale aggiunta dietro `aurea-media/heif`; adozione runtime e approvazione produzione restano subordinate a CI, revisione sicurezza, fixture e verifica pacchetti/licenze.

### JPEG 2000

- **Stato:** candidato rinviato.
- **Scopo:** interoperabilità opzionale con macOS per immagini .jp2/.j2k.
- **Evidenza:** Apple Preview documenta l'esportazione JPEG 2000, ma non è ancora stato approvato un decoder Debian 13, un adapter Rust, la licenza o il footprint.
- **Decisione:** non dichiarare il supporto finché non viene selezionato e testato un backend compatibile Debian.

### Esclusione ambito

Nessun formato video o libreria di riproduzione è selezionato per gli sfondi. GStreamer, plugin video FFmpeg, H.264, HEVC video e AV1 video restano fuori dall'ambito degli sfondi statici di Aurea Media.


## Dipendenza decoder raster selezionata — 24 novembre 2026

- **Crate:** image 0.25.10.
- **Stato:** selezionata in Cargo.toml di aurea-media; Cargo.lock non è stato rigenerato in questa sessione perché l'utente ha chiesto di non avviare build/check/test. Il lockfile dovrà essere aggiornato su Fedora prima di una build bloccata.
- **Sole feature decoder abilitate:** bmp, gif, png, tiff, webp. default-features = false; Rayon, AVIF, EXR, DDS, HDR e gli altri formati non necessari restano disabilitati.
- **Scopo:** adapter Rust per BMP, GIF, PNG, TIFF e WebP. JPEG resta sul backend TurboJPEG esistente; non viene aggiunto un secondo decoder JPEG.
- **Evidenza upstream:** image 0.25.10 documenta questi formati e raccomanda di disabilitare le feature predefinite nelle librerie. Limits offre limiti stretti per larghezza/altezza massime; il limite di allocazione è best-effort e alcuni decoder possono ignorarlo.
- **Licenza:** espressione di licenza del crate e licenze di tutte le dipendenze transitive abilitate da verificare sul grafo Cargo risolto prima dell'approvazione alla distribuzione.
- **Debian 13:** dipendenza Rust; questa selezione non richiede pacchetti codec di sistema. Build e comportamento runtime su Trixie non ancora verificati.
- **Nota sicurezza:** AureaOS applica limiti sugli assi prima della decodifica e verifica separatamente pixel, byte RGBA8 e lunghezza esatta dell'output. Poiché max_alloc è non stretto per alcuni decoder, non può essere considerato da solo una barriera di memoria assoluta. La coda worker resta limitata.
- **Da completare:** rigenerare Cargo.lock su Fedora, controllare grafo e licenze esatti, aggiungere fixture e test per formati/limiti, poi eseguire un unico ciclo pianificato di build e test.

## Confine di proprietà degli asset wallpaper — 24 novembre 2026

- **Stato:** implementazione sorgente aggiunta; verifica build in sospeso.
- **Filesystem:** nessuna nuova dipendenza. L'I/O Rust standard è confinato in WallpaperAsset::from_file, che controlla la dimensione dei metadati e poi esegue una lettura limitata.
- **Contratto thread:** il caricamento file va invocato da un'attività impostazioni/media-I/O, mai dal callback frame/render. La decodifica resta nel DecodeWorker.
- **Identità:** ID degli asset e chiavi decode/texture sono stringhe possedute; eliminato il vincolo static per questa pipeline.
- **Rilevamento input:** JPEG, PNG, BMP, GIF, TIFF e WebP sono identificati tramite firma. L'estensione non è attendibile per scegliere il decoder. HEIC/HEIF e AVIF non sono ancora abilitati.
- **Memoria:** input compresso limitato a 64 MiB. I byte compressi sono condivisi in sola lettura nell'asset, poi copiati nella richiesta worker limitata. La copia temporanea va conteggiata nel budget memoria.
- **Verifica:** nessuna build/test/rustfmt eseguiti. API ownership e call site snapshot da rivedere nel prossimo ciclo Fedora.

## Aggiornamento verifica backend HEIF/AVIF — 24 novembre 2026

Nuova verifica dei metadati pacchetti Debian Trixie:

- I pacchetti sorgente/runtime/development libheif sono disponibili nella versione 1.23.6-2~deb13u1 nei metadati Trixie attualmente aggiornati con le security update.
- La decodifica immagini HEVC è fornita da libheif-plugin-libde265, che dipende da libde265-0.
- La decodifica immagini AV1 è fornita da libheif-plugin-dav1d, che dipende da libdav1d7. Il plugin occupa 71 KiB installati su amd64 secondo l'elenco pacchetti attuale; il runtime codec è aggiuntivo.
- libheif documenta HEIF come contenitore per immagini codificate HEVC o AV1 e carica le implementazioni codec tramite plugin separati.
- Non installare il metapacchetto libheif-plugins-all. Se approvati, selezionare soltanto i plugin decoder HEVC e AV1.
- Compatibilità esatta delle licenze upstream di libheif, libde265 e dav1d, ABI, parità pacchetti Fedora e scoperta plugin runtime restano da verificare. Nessuna dipendenza nativa è stata aggiunta a Cargo.toml o ai pacchetti target.

Evidenza: pagine pacchetto Debian Trixie libheif-dev, libheif-plugin-libde265 e libheif-plugin-dav1d: https://packages.debian.org/trixie/libheif-dev, https://packages.debian.org/trixie/libheif-plugin-libde265, https://packages.debian.org/trixie/libheif-plugin-dav1d.


## Noto Sans — tipografia menu contestuale — 25 novembre 2026

- **Scelta:** Noto Sans di Google Fonts / The Noto Project, peso Regular 500 per il menu contestuale iniziale.
- **Licenza:** SIL Open Font License 1.1 (OFL-1.1). La famiglia può essere usata e redistribuita insieme al software rispettando le condizioni OFL; se in futuro i file font verranno inclusi, mantenere copyright e licenza upstream. Riferimento: [guida Noto a uso e licenza](https://notofonts.github.io/noto-docs/website/use/).
- **Uso in generazione:** le SVG dichiarano font-family=Noto Sans; `tools/assets/render-context-menu.sh` risolve direttamente il file `aurea-human-interfaces/fonts/source/google-fonts/noto-sans/NotoSans-Variable.ttf` tramite una configurazione Fontconfig temporanea. Non serve installare il font sull'host né scaricarlo.
- **Pacchetti host:** Debian 13 `librsvg2-bin`; Fedora `librsvg2-tools`. Python 3 è usato dal convertitore PNG→RGBA già presente.
- **Impatto runtime:** nessuna dipendenza Cargo e nessuna dipendenza fontconfig/Pango/Cairo/GLib aggiunta al runtime AureaOS. Nessun download font o accesso alla rete durante l'esecuzione.
- **Stato:** sorgenti SVG e script di esportazione committati. Generazione RGBA8 e controllo visivo ancora da eseguire su Fedora.
