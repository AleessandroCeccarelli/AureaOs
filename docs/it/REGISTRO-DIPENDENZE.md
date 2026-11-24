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
