<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Decoder HEIF / HEIC e AVIF — decisione d'integrazione

- Data: 2026-11-24
- Stato: **Candidato condizionato; implementazione non ancora approvata**
- Responsabile: `components/aurea-media`
- Ambito: esclusivamente decodifica di singole immagini statiche per gli sfondi
- Documento inglese: [HEIF-AVIF-DECODER-DECISION.md](HEIF-AVIF-DECODER-DECISION.md)

## Confine di prodotto

AureaOS ha approvato HEIF/HEIC e AVIF come formati immagine statici per gli sfondi nel registro multimediale ufficiale. L'approvazione dei formati non approva automaticamente una dipendenza decoder e non significa che questi formati funzionino già.

L'adattatore dovrà:
- decodificare soltanto l'immagine statica principale;
- rifiutare sequenze di immagini e contenitori multi-immagine nella prima implementazione;
- non esporre mai riproduzione video o di sequenze;
- restituire pixel RGBA8 posseduti e compatti;
- lavorare esclusivamente sul worker multimediale già limitato;
- mantenere i dettagli dei codec nativi fuori dal compositor e dal renderer Vulkan.

## Tecnologia candidata

La candidata attuale è **libheif**, tramite il wrapper Rust sicuro **libheif-rs 3.0.0**.

Evidenze verificate:
- libheif usa una sola API di decodifica per HEIF e AVIF e supporta implementazioni codec tramite plugin separati.
- libheif-rs è un wrapper sicuro con licenza MIT. La documentazione corrente richiede gli header di sviluppo libheif versione 1.17 o successiva e indica Rust MSRV 1.82.
- Il wrapper espone i limiti di sicurezza del contesto con la feature API `v1_19`, i limiti di memoria totale e il rilevamento delle sequenze con `v1_20`.
- Debian 13 Trixie fornisce libheif 1.23.x, `libheif-dev` e pacchetti separati per i plugin decoder.

Riferimenti:
- https://github.com/strukturag/libheif
- https://docs.rs/libheif-rs/3.0.0/libheif_rs/
- https://docs.rs/libheif-rs/3.0.0/libheif_rs/struct.SecurityLimits.html
- https://docs.rs/libheif-rs/3.0.0/libheif_rs/struct.HeifContext.html
- https://packages.debian.org/trixie/libheif-dev
- https://packages.debian.org/trixie/libheif-plugin-libde265
- https://packages.debian.org/trixie/libheif-plugin-dav1d

## Proposta per l'ingombro delle dipendenze

Usare plugin runtime dedicati alla sola decodifica:
- Immagini statiche HEIC / HEVC: `libheif-plugin-libde265`.
- Immagini statiche AVIF: `libheif-plugin-dav1d` (oppure `libheif-plugin-aomdec` soltanto se i test di compatibilità Debian dimostrano una ragione concreta per preferirlo).

Per questa funzionalità non installare plugin encoder, il metapacchetto `libheif-plugins-all`, plugin decoder FFmpeg o librerie di riproduzione video. Il grafo esatto delle dipendenze runtime dovrà essere confermato tramite i metadati dei pacchetti Debian prima del packaging.

Dipendenza di compilazione:
- `libheif-dev` (oltre ai requisiti pkg-config del wrapper Rust).

Dipendenze runtime:
- `libheif1` e soltanto i plugin decoder selezionati con le rispettive librerie runtime transitive.

Nomi e versioni dei pacchetti Fedora restano **non verificati**. Non dedurre la disponibilità sulla workstation locale dai risultati di GitHub Actions o dai metadati Debian.

## Verifica di sicurezza e manutenzione

Il README upstream di libheif consultato il 24/11/2026 segnala che il progetto è mantenuto da un singolo sviluppatore indipendente con finanziamenti ricorrenti limitati e che nel 2026 sono state esaminate, corrette e rilasciate 89 segnalazioni di sicurezza.

È un segnale rilevante di rischio per un parser. Non esclude automaticamente libheif, ma impedisce un'adozione incondizionata. Prima dell'uso in produzione:
- esaminare la politica di sicurezza upstream e le segnalazioni correnti;
- stabilire come gli aggiornamenti di sicurezza Debian raggiungeranno l'immagine AureaOS;
- mantenere il decoder isolato dietro l'API Aurea Media;
- impedire che file malformati raggiungano il thread di rendering;
- valutare un processo helper ristretto se il modello delle minacce non accetta un parser nativo in-process.

## Sequenza di decodifica richiesta

1. Applicare il limite già esistente alla dimensione compressa prima di passare i byte a libheif.
2. Creare un contesto e applicare i limiti di sicurezza **prima del parsing** del contenitore non attendibile.
3. Impostare il numero massimo di pixel immagine su `DecodeLimits.max_pixels()`.
4. Impostare il limite di memoria totale del contesto libheif su `DecodeLimits.max_output_bytes()` (o su un budget di memoria di lavoro documentato e più restrittivo).
5. Eseguire il parsing dal buffer preso in prestito; mantenerlo vivo per tutta la durata del contesto.
6. Rifiutare `has_sequence()` e qualsiasi contenitore che non abbia esattamente un'immagine di primo livello.
7. Ottenere l'handle dell'immagine principale e verificare larghezza, altezza, numero di pixel, limite per asse e lunghezza RGBA con aritmetica controllata prima della decodifica.
8. Decodificare in RGBA interleaved a 8 bit. La conservazione della sorgente HDR è fuori dalla politica sfondi v1.0.
9. Verificare presenza del piano, stride di riga e byte leggibili totali. Copiare riga per riga in un vettore RGBA8 compatto allocato in modo fallibile; non assumere che lo stride nativo sia uguale a larghezza × 4.
10. Rilasciare immagine, handle e contesto prima di restituire il buffer Rust posseduto.

L'insieme delle feature del wrapper deve includere almeno `v1_20` per esporre le API di sicurezza e rilevamento sequenze richieste. Non abilitare implicitamente la feature `latest` del crate: selezionare deliberatamente la superficie API minima necessaria.

## Identificazione del contenitore

Identificare i brand principali e compatibili ISO-BMFF `ftyp`, non l'estensione del file.

- I brand AVIF per immagini statiche includono `avif`; rifiutare il brand di sequenza `avis`.
- I brand HEIC per immagini statiche includono `heic` e `heix`; rifiutare il brand di sequenza `msf1`.
- Non considerare il solo brand generico `mif1` come prova di supporto HEIC/AVIF.
- Validare lunghezza dei box, overflow degli interi e allineamento dei brand compatibili prima di leggere i campi.
- Il rilevamento della firma serve soltanto a selezionare il decoder. Parsing libheif, controllo sequenze e validazione decoder restano obbligatori.

## CI e compatibilità Debian

L'attuale build Rust CI gira su Ubuntu 24.04 e non installa libheif. Non aggiungere una feature crate che richieda libheif 1.20+ a quel job senza aver prima verificato la versione della libreria nativa disponibile nel runner.

Prima di abilitare l'adattatore nella build predefinita del workspace:
1. Aggiungere un job di compatibilità dedicato Debian 13 Trixie con `libheif-dev`, `libheif-plugin-libde265`, `libheif-plugin-dav1d` e pkg-config.
2. Verificare compilazione e linking esatti di `libheif-rs` / `libheif-sys` contro la libreria installata da Debian, non una copia vendorizzata o embedded.
3. Verificare lo stesso percorso API/ABI sulla workstation Fedora di sviluppo quando disponibile.
4. Confermare che l'immagine runtime Debian includa soltanto i pacchetti runtime selezionati.
5. Aggiungere fixture HEIC e AVIF piccole, con licenza chiara o generate, oltre a fixture malformate, troncate, fuori limite, multi-immagine e con sequenze.
6. Eseguire i normali controlli workspace, DRM/Vulkan, audit di sicurezza e test media dopo aver raggruppato le modifiche.

## Stato attuale dell'implementazione — 24 novembre 2026

- Aggiunto un adapter HEIF/AVIF privato in `aurea-media`, basato sulla dipendenza opzionale `libheif-rs = 3.0.0` con la sola feature API `v1_20`.
- `aurea-media` espone la feature Cargo opt-in `heif`; `aurea-compositor` la inoltra con la propria feature `heif`. Nessuna delle due è attiva per default: le build ordinarie del workspace non acquisiscono una dipendenza nativa libheif.
- L'adapter applica limiti a input, pixel, dimensioni e memoria del decoder nativo; rifiuta sequenze e contenitori multi-immagine; verifica il formato richiesto rispetto ai brand ISO-BMFF; copia il piano RGBA riga per riga in memoria posseduta e compatta.
- Aggiunto un job CI Debian 13 che installa header libheif e plugin decoder selezionati, quindi compila/testa Aurea Media con `heif` attivo. La prima esecuzione ha rilevato un errore di visibilità Rust e Rust 1.85 Debian troppo vecchio per `image` (MSRV 1.88). Entrambe le cause sono state corrette nel sorgente/workflow, ma serve ancora una nuova esecuzione.
- Revisione sicurezza/manutenzione, fixture reali HEIC/AVIF, parità pacchetti Fedora e verifica manifest runtime restano gate aperti. È un'implementazione opt-in in validazione, non ancora approvata per la produzione.
