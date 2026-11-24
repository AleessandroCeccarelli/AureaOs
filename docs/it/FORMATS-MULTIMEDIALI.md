<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Registro ufficiale dei formati multimediali AureaOS

- Stato: **Politica ufficiale di prodotto**
- Versione: 1.0
- Data: 2026-11-24
- Documento principale: [MEDIA-FORMATS.md](../architecture/MEDIA-FORMATS.md)
- Ambito: formati accettati dalle applicazioni e dai servizi di sistema AureaOS. Questo registro è il riferimento ufficiale; lo stato dell'implementazione viene indicato separatamente.

## 1. Politica

AureaOS supporta un insieme selezionato di formati multimediali. Un formato non diventa supportato soltanto perché esiste una libreria di terze parti capace di decodificarlo. Ogni nuovo formato richiede una decisione esplicita sul prodotto, un'implementazione compatibile con Debian, una verifica delle licenze e delle dipendenze, limiti di risorse, test e aggiornamento di entrambe le versioni linguistiche di questo registro.

Il sistema degli sfondi accetta **esclusivamente immagini raster statiche**. Le immagini animate e i file video non sono formati validi per gli sfondi desktop.

Le estensioni sono suggerimenti per i selettori di file, non un confine di sicurezza. Gli importatori devono verificare firme e metadati del contenitore, validare il contenuto decodificato, applicare limiti a dimensioni e memoria e rifiutare file malformati o non supportati.

## 2. Formati ufficiali per gli sfondi desktop

I seguenti formati sono approvati dalla politica AureaOS per gli sfondi. “Approvato” descrive la decisione di prodotto: non significa che ogni decoder sia già integrato e verificato su Fedora e Debian.

| Formato | Estensioni accettate | Utilizzo | Stato implementazione |
|---|---|---|---|
| JPEG | `.jpg`, `.jpeg`, `.jpe` | Fotografie e immagini di uso generale | Percorso TurboJPEG esistente; verifica locale in attesa |
| PNG | `.png` | Immagini senza perdita, trasparenze e grafica digitale | Codice decoder presente; build e test da verificare |
| BMP | `.bmp` | Compatibilità bitmap e immagini legacy | Codice decoder presente; build e test da verificare |
| TIFF | `.tif`, `.tiff` | Fotografia professionale, scansioni, prestampa e archiviazione | Codice decoder presente; build e test da verificare |
| WebP statico | `.webp` | Immagini statiche efficienti per il web e uso generale | Codice decoder presente; build e test da verificare |
| HEIF / HEIC (immagini statiche) | `.heif`, `.heic` | Fotografie moderne da fotocamere e smartphone | Approvato; adapter statico libheif da realizzare |
| AVIF (immagini statiche) | `.avif` | Immagini statiche moderne ad alta efficienza | Approvato; adapter statico libheif/AV1 da realizzare |

### Regole di gestione delle immagini

- JPEG, PNG, BMP, TIFF, WebP, HEIF/HEIC e AVIF costituiscono l'elenco completo dei formati immagine approvati per gli sfondi in questa versione della politica.
- GIF **non è accettato**, neppure come immagine statica del primo fotogramma.
- APNG e WebP animato vengono rifiutati. L'importatore non deve trasformare silenziosamente un contenuto animato in uno sfondo scegliendone un fotogramma.
- TIFF è un formato per immagini statiche. Se un contenitore TIFF include più pagine/immagini, l'importatore dello sfondo utilizza soltanto la prima pagina. Non riproduce né alterna le pagine.
- Il supporto HEIF/HEIC e AVIF riguarda esclusivamente la decodifica di immagini statiche. Non autorizza la riproduzione video HEVC o AV1.
- I pixel decodificati degli sfondi vengono normalizzati nel formato RGBA8 validato richiesto dal compositor. Questa politica iniziale non garantisce la conservazione della profondità colore elevata o dell'HDR della sorgente, né un'uscita HDR con gestione colore.
- L'estensione da sola non dimostra il supporto: firma del contenuto e capacità del decoder devono corrispondere.

## 3. Formati esplicitamente non supportati per gli sfondi

I seguenti formati sono esclusi dall'elenco approvato per gli sfondi nella versione 1.0:

| Formato / tipo | Decisione |
|---|---|
| GIF | Rifiutato |
| APNG | Rifiutato |
| WebP animato | Rifiutato |
| Contenitori e codec video (inclusi MP4, MOV, MKV, H.264, HEVC e AV1 video) | Rifiutati |
| SVG | Non accettato come sfondo; la grafica vettoriale richiede una decisione separata |
| PDF | Non accettato come sfondo; la rasterizzazione dei documenti richiede una decisione separata |
| Formati fotografici RAW | Non accettati nella prima versione degli sfondi |
| JPEG 2000 | Non approvato |
| OpenEXR | Non approvato nella prima versione degli sfondi |

L'elenco è volutamente chiuso. Non aggiungere decoder di ripiego e non accettare silenziosamente altri formati senza una decisione registrata.

## 4. Formati audio e musicali

**Al momento questo registro non approva ancora alcun codec o contenitore audio/musicale.**

La sezione è riservata alle future decisioni di prodotto riguardanti almeno:
- formati e contenitori per la riproduzione musicale;
- codec audio lossless e lossy;
- gestione di metadati e copertine;
- registrazione, streaming e suoni di sistema, se approvati separatamente.

La presenza di una libreria, di un pacchetto Linux o del supporto in un'altra applicazione non implica il supporto audio in AureaOS. I formati audio potranno essere aggiunti soltanto dopo una decisione esplicita e una verifica di compatibilità e licenze per Debian.

## 5. Altre categorie multimediali

Altre categorie (per esempio riproduzione video, acquisizione, streaming, sottotitoli e interscambio professionale) **non sono approvate da questo documento**. Prima di aggiungere codec o contenitori serviranno decisioni separate su ambito e architettura.

## 6. Requisiti per dipendenze e implementazione

- Debian 13 è la base di compatibilità. Fedora è l'ambiente di sviluppo e validazione, non l'autorità per le dipendenze del prodotto.
- Mantenere le implementazioni specifiche dei codec dentro `aurea-media` o un altro componente multimediale designato esplicitamente. Il compositor riceve buffer di pixel validati e non deve possedere dipendenze dai codec immagine.
- Preferire feature decoder abilitate in modo selettivo. Non installare raccolte generiche di plugin/codec o encoder quando serve soltanto la decodifica.
- Verificare licenze upstream e transitive, disponibilità dei pacchetti Debian, caricamento dei plugin runtime, sicurezza, manutenzione, ingombro e compatibilità ABI.
- Decodificare su worker con limiti, mai nel callback di frame/render del compositor.
- Applicare limiti alla dimensione dell'input, alle dimensioni dell'immagine, al numero di pixel, alle allocazioni e ai byte in uscita. Rifiutare in modo deterministico file malformati e funzionalità non supportate.
- Registrare le decisioni precise su decoder/librerie e pacchetti nel documento architetturale e nel diario di sviluppo.
- Aggiornare questo registro inglese e la variante italiana nella stessa modifica ogni volta che cambia la politica.

## 7. Procedura per le modifiche

Un nuovo formato può essere aggiunto soltanto quando sono documentati tutti i seguenti elementi:
1. Caso d'uso e ambito esatto di estensioni/contenitori.
2. Comportamento statico o animato, quando pertinente.
3. Decoder/libreria compatibile con Debian e relativi pacchetti runtime.
4. Verifica delle licenze e delle dipendenze transitive.
5. Limiti delle risorse e comportamento con input malformati.
6. Copertura dei test e stato delle verifiche su Debian e Fedora.
7. Aggiornamento corrispondente di entrambe le versioni linguistiche del registro.

La disponibilità tecnica di un codec non costituisce, da sola, un'approvazione alla distribuzione.
