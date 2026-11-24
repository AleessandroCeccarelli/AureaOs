<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Architettura di Aurea File Intelligence e dei metadati

## Stato

Documento di decisione architetturale: fondazione proposta, non dichiarazione di funzionalità implementate.

Questo documento definisce come AureaOS dovrà individuare, descrivere, indicizzare ed esporre i file dell'utente a Fides, Quaero e Desktop. Distingue l'interoperabilità Linux dal comportamento proprietario del prodotto AureaOS.

## Principi di prodotto

1. AureaOS deve risultare familiare senza riprodurre il linguaggio visivo, l'architettura interna o i comportamenti proprietari di altri desktop.
2. Le informazioni sui file devono avere un unico proprietario autorevole. Le interfacce grafiche consumano contratti condivisi e non mantengono indici concorrenti.
3. Non creare file nascosti di supporto nelle cartelle dell'utente soltanto per memorizzare preferenze di visualizzazione.
4. Preferire le funzionalità Linux/FreeDesktop esistenti quando risolvono il problema; introdurre comportamenti AureaOS solo dove la piattaforma lascia una lacuna reale.
5. L'indicizzazione deve essere incrementale, limitata nelle risorse, rispettosa dei permessi e recuperabile.
6. L'indice contiene dati derivati. I metadati scritti dall'utente e le preferenze devono essere persistenti e non andare persi quando l'indice viene ricostruito.
7. Ogni estrattore costoso deve lavorare fuori dal compositor, dal ciclo di rendering e dai percorsi sensibili alla latenza dell'interfaccia.
8. Filesystem di rete, rimovibili, virtuali o non supportati devono degradare in modo controllato.

## Modello delle responsabilità

### Aurea File Intelligence

Aurea File Intelligence è la funzionalità di prodotto che mantiene un catalogo locale dei file ed espone interrogazioni strutturate. Non è un gestore file, un renderer desktop o un'interfaccia di ricerca.

Responsabilità:

- Individuare le radici configurate e le relative esclusioni.
- Eseguire scansioni iniziali limitate e riconciliazioni incrementali.
- Identificare i tipi di file tramite il database MIME di sistema, non soltanto tramite le estensioni.
- Raccogliere informazioni filesystem a basso costo.
- Pianificare l'estrazione opzionale di metadati e contenuti in base al tipo e alla policy.
- Mantenere un indice interrogabile e comunicarne aggiornamento e copertura.
- Esporre contratti stabili, rispettosi dei permessi, ai componenti desktop.
- Gestire rimozioni, spostamenti, percorsi inaccessibili, montaggio e smontaggio dei volumi e ricostruzione dell'indice.

La prima implementazione dovrebbe essere una libreria/servizio AureaOS ospitata dal composition root della sessione desktop. Non creare un demone permanente o un protocollo D-Bus prima che ciclo di vita, client multiprocesso, isolamento o carichi misurati ne dimostrino la necessità. Il confine API deve restare indipendente dal trasporto, così da poter introdurre in futuro un servizio separato nella sessione utente senza riprogettare i client.

### Fides (Aurea Files)

È responsabile delle operazioni e della navigazione: esplorazione, copia, spostamento, rinomina, eliminazione, ripristino, apertura con applicazione e modifica delle proprietà visibili all'utente. Richiede i dati catalogati a File Intelligence, ma non mantiene un secondo indice.

### Quaero

È responsabile dell'esperienza di ricerca, delle politiche di ordinamento per rilevanza e della presentazione dei risultati. Interroga File Intelligence e deve indicare se i risultati sono completi, se l'indicizzazione è ancora in corso o se un elemento non è disponibile per permessi o stato del volume. La ricerca non deve aggirare i controlli di accesso al filesystem.

### Desktop

È responsabile della disposizione degli elementi sul desktop, della selezione, dell'organizzazione e delle preferenze di presentazione specifiche del desktop. Non cataloga l'intero filesystem e non salva lo stato di visualizzazione delle cartelle in file nascosti per-directory. Le posizioni delle icone sono configurazioni utente associate a un'identità stabile dell'elemento e al contesto di uscita/layout.

### Aurea Media

Gestisce analisi media limitata, generazione miniature e adattatori di decodifica. Esegue il lavoro su worker, applica limiti a dimensione file, tempo e memoria e restituisce i risultati al catalogo o alla cache. Non decide quali directory indicizzare né come Quaero ordina i risultati.

### Compositor

Gestisce superfici Wayland, uscite, instradamento del puntatore e composizione Vulkan. Non deve scandire directory, analizzare documenti, estrarre metadati o accedere al catalogo dal ciclo di rendering.

## Tassonomia dei metadati

Mantenere separate queste categorie sia nello storage sia nelle API:

1. **Informazioni filesystem** — percorso/URI, indizi di identità del file, dimensione, date, permessi, proprietario e contesto del volume. Interrogare il filesystem quando serve freschezza; memorizzare in cache solo dichiarando esplicitamente la freschezza.
2. **Metadati nativi del formato** — EXIF, XMP, ID3, metadati dei contenitori multimediali e proprietà dei documenti. Leggerli senza modificare i file originali, salvo esplicita modifica richiesta dall'utente.
3. **Metadati estratti** — tipo MIME, dimensioni, durata, codec, numero di pagine, testo e riferimenti alle anteprime generate. Sono derivati e ricostruibili.
4. **Metadati scritti dall'utente** — tag, valutazioni, descrizioni, etichette e preferiti. Sono persistenti e non devono essere eliminati silenziosamente durante una nuova indicizzazione.
5. **Stato di presentazione** — modalità di visualizzazione/ordinamento per cartella e posizioni delle icone sul desktop. È di competenza della relativa interfaccia, non dell'indice globale dei contenuti.
6. **Miniature e artefatti di cache** — elementi temporanei, versionati e limitati. Non sono mai la fonte autorevole.

Non presumere che i numeri inode siano globalmente stabili. L'identità di un file dovrebbe combinare, quando disponibili, contesto device/mount e identità filesystem, mantenendo un fallback URI/percorso e gestendo sostituzioni, rinomine e spostamenti tra volumi. L'hash del contenuto è opzionale e su richiesta: non calcolare l'hash di ogni file durante una scansione ordinaria.

## Direzione dello storage

### Dati persistenti dell'utente

Conservare metadati e preferenze AureaOS nelle directory XDG di dati/configurazione dell'utente, con regole chiare di backup e migrazione. Non inserire per impostazione predefinita file privati AureaOS nelle cartelle arbitrarie dell'utente.

### Catalogo derivato

Usare come candidato principale un database locale embedded. SQLite è il candidato preferito per la prima fase: maturo, ispezionabile, transazionale, portabile e senza necessità di un server database. È una scelta candidata, non una dipendenza già approvata o introdotta.

Prima dell'implementazione verificare su Debian 13 e Fedora:
- disponibilità della libreria SQLite e dei pacchetti di sviluppo;
- disponibilità di FTS5 nella build di sistema selezionata;
- manutenzione, licenza, MSRV e modalità di linking statico/dinamico del binding Rust;
- comportamento WAL e recupero sicuro dopo perdita di alimentazione;
- strategia di migrazione e versione dello schema;
- dimensione del database e amplificazione delle scritture con carichi realistici.

Mantenere normalizzato lo schema dei metadati canonici e usare un indice full-text dedicato soltanto per i contenuti testuali esplicitamente indicizzati. Non inserire grandi anteprime binarie o payload media nelle righe del database: conservarli in una cache separata e limitata.

### Nessun sidecar per ogni directory

Non creare un file nascosto di metadati in ogni cartella. Lo stato di presentazione delle cartelle deve risiedere nella configurazione/directory dati AureaOS dell'utente, associato a un URI normalizzato e validato all'accesso. Se in futuro servirà interoperabilità in lettura/scrittura con i metadati di altri desktop, realizzarla come livello di compatibilità esplicito, mai come formato canonico AureaOS.

## Interoperabilità Linux

Rispettare e utilizzare le convenzioni Linux esistenti:

- XDG Base Directory per configurazione, dati, cache e stato.
- Shared MIME-info per identificazione e descrizione MIME.
- Desktop Entry e convenzioni Icon Theme per individuazione applicazioni e icone.
- Specifica FreeDesktop Trash per interoperabilità di eliminazione e ripristino.
- Thumbnail Managing Standard e thumbnailer compatibili, dove applicabile.
- URI file e protocolli Wayland esistenti per riferimenti ai file e drag-and-drop.
- Permessi filesystem e semantica dei mount come confine di sicurezza.

Non dipendere da GNOME Tracker, KDE Baloo, GTK, Qt o database/servizi privati di altri ambienti desktop. Sono riferimenti di studio, non dipendenze runtime.

La specifica FreeDesktop shared-file-metadata non viene considerata un contratto stabile universalmente distribuito. Il suo stato dovrà essere ricontrollato prima di promettere interoperabilità.

## Individuazione e rilevamento delle modifiche

Adottare un modello a due componenti:

1. **Scanner di riconciliazione** — individuazione autorevole delle radici configurate, con batch limitati, cancellazione, consapevolezza dei mount e avanzamento persistente.
2. **Adattatore eventi di modifica** — notifiche native Linux per aggiornamenti a bassa latenza.

Una crate Rust come notify può essere valutata, ma non va adottata automaticamente nella sua versione maggiore più recente. Bloccare una versione stabile compatibile con l'MSRV del workspace e verificarne backend Linux, licenza e limitazioni note. Un adattatore nativo inotify più ristretto è un'alternativa se offre un modello di risorse e ciclo di vita sensibilmente migliore.

Le notifiche sono suggerimenti, non la fonte della verità. Accorpare gli eventi ravvicinati, gestire overflow e invalidazione delle watch, registrare nuovamente le directory sostituite e pianificare riconciliazioni dopo interruzioni, ripresa dalla sospensione, cambi di mount o incoerenze. I filesystem di rete possono richiedere un fallback a polling esplicito. Evitare di sorvegliare ricorsivamente ogni file se bastano watch a livello directory e riconciliazioni limitate.

Non iniziare con intercettazione privilegiata fanotify o monitoraggio kernel con privilegi elevati. Aumenterebbero privilegi e complessità senza essere necessari per un catalogo desktop dell'utente.

## Policy di indicizzazione e budget risorse

- Le radici predefinite devono essere posizioni utente esplicite, non l'intero filesystem.
- Escludere per impostazione predefinita alberi di sistema, cache, directory nascoste di implementazione e alberi di sviluppo/build; consentire eccezioni configurabili dall'utente.
- Non seguire symlink fuori dalle radici configurate, salvo abilitazione esplicita.
- Volumi rimovibili e di rete devono essere opzionali o soggetti a policy; indicare il loro stato online/indicizzato.
- Applicare limiti per file su dimensione, tempo, memoria e profondità di ricorsione agli estrattori.
- Dare priorità ai metadati necessari alle viste Fides visibili e alle ricerche Quaero recenti; rinviare l'estrazione full-text costosa.
- Sospendere o ridurre il lavoro a batteria, con carico elevato, poco spazio libero o su richiesta dell'utente.
- Fornire controlli per esaminare posizioni indicizzate, esclusioni, stato, pausa/ripresa e ricostruzione.
- Rispettare i permessi in fase di individuazione, estrazione e interrogazione. Un risultato nell'indice non deve mai concedere accesso al file.
- Trattare file malformati e contenuti media/documenti ostili come input non attendibile. Gli estrattori devono essere isolati dal compositor e soggetti a limiti rigorosi.

## Contratto di interrogazione e freschezza

Le query dovrebbero restituire record strutturati con:
- Identificatore stabile dell'elemento Aurea.
- URI/percorso e migliore identità filesystem disponibile.
- Tipo MIME e metadati tipizzati.
- Campi di spiegazione della corrispondenza/rilevanza, dove utili.
- Data dell'ultima osservazione/verifica e stato di freschezza.
- Stato di disponibilità (online, offline, inaccessibile, obsoleto, estrazione in attesa).
- Riferimento alla miniatura soltanto se esiste un artefatto valido in cache.

L'assenza di un risultato nell'indice non deve essere presentata come prova che il file non esista. I client devono poter richiedere un fallback diretto al filesystem per la cartella corrente, distinguendo chiaramente risultati in tempo reale da risultati catalogati.

## Capacità future

Mantenere punti di estensione per:
- Ricerca semantica e in linguaggio naturale sui contenuti indicizzati localmente, opzionale e rispettosa della privacy.
- Embedding dei contenuti o recupero vettoriale come indice opzionale separato, mai prerequisito per la normale ricerca per nome.
- Indici privati/cifrati per utente, quando modello di minaccia e ciclo di vita delle chiavi saranno definiti.
- Provider di contenuti diversi dai file locali (portali documentali, cloud, supporti rimovibili) tramite interfacce esplicite.
- Provenienza e affidabilità dei metadati, per distinguere fatti estratti, modifiche utente e valori nativi del formato.

Non introdurre nel desktop iniziale un database vettoriale, runtime ML, indice cloud o processo AI permanente. Prima stabilire contratti tipizzati per metadati e provider; aggiungere recupero avanzato soltanto quando esiste un beneficio misurabile e un budget hardware adeguato.

## Fasi di implementazione

1. Finalizzare contratti pubblici per elementi/metadati/query e regole di identità stabile.
2. Esaminare moduli file, desktop, ricerca e media già esistenti per evitare responsabilità duplicate.
3. Prototipare disponibilità di database e FTS5 su Debian 13 e Fedora senza introdurre ancora una dipendenza runtime.
4. Implementare scanner e macchina a stati di riconciliazione con test filesystem sintetici.
5. Aggiungere integrazione eventi con gestione overflow e recupero.
6. Aggiungere estrazione MIME e metadati a basso costo.
7. Aggiungere estrattori media/miniature limitati tramite Aurea Media.
8. Collegare Fides e Quaero; mantenere separato lo stato di presentazione specifico del Desktop.
9. Aggiungere indicizzazione contenuti opzionale e capacità semantiche future soltanto dopo profilazione.

## Esclusioni esplicite

- Nessun clone dei file metadati per-directory di macOS.
- Nessuna dipendenza da GNOME Tracker, KDE Baloo o API private.
- Nessuna indicizzazione dell'intero disco per impostazione predefinita.
- Nessun monitoraggio privilegiato basato su fanotify.
- Nessun servizio cloud, database vettoriale o modello AI obbligatorio.
- Nessuna estrazione metadati nel compositor o nel ciclo Vulkan.

## Riferimenti

- Specifiche FreeDesktop: https://www.freedesktop.org/specifications/
- Specifica Shared File Metadata: https://www.freedesktop.org/specifications/shared-filemetadata-spec/
- Documentazione Rust notify: https://docs.rs/notify/latest/notify/
- Documentazione Linux inotify: https://man7.org/linux/man-pages/man7/inotify.7.html
