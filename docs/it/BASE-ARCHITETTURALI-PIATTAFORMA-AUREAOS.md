<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Fondamenta della piattaforma AureaOS

## Stato
Documento di indirizzo architetturale futuro. Registra visione e vincoli del prodotto; non afferma che le funzionalità descritte siano già implementate.

## Visione del prodotto
AureaOS è una piattaforma desktop coerente e nativa Linux, costruita su Debian, Btrfs, Rust, Wayland, Smithay e Vulkan. Deve offrire continuità e facilità d'uso senza copiare interfacce, servizi privati o implementazioni di altri sistemi.

La domanda guida è: **quale esperienza vogliamo offrire e qual è il modo corretto, nativo Linux, per realizzarla?** L'utente non deve conoscere i dettagli interni di Linux per usare il desktop, ma AureaOS deve rispettare semantica filesystem, permessi, interoperabilità e standard.

## Fondamenta tecnologiche
- Debian GNU/Linux 13 Trixie, inizialmente amd64.
- Btrfs come filesystem di sistema scelto, con una sola partizione Btrfs per l'intero sistema installato; backup e ripristino sono responsabilità di Aurea Time.
- systemd-boot come boot manager esclusivamente UEFI; Secure Boot è disabilitato in Roma 1.0.
- Rust come linguaggio principale per servizi, API, shell e componenti desktop.
- Wayland + Smithay come architettura nativa display/compositor; niente X11 o XWayland.
- Vulkan come API grafica esclusiva; nessun ripiego OpenGL.
- Convenzioni FreeDesktop/XDG per interoperabilità.
- GPL-3.0-or-later.

Queste scelte sono vincoli, non un motivo per duplicare funzionalità già offerte da Linux.

## Architettura di avvio
Il boot manager approvato è systemd-boot. AureaOS supporta esclusivamente sistemi UEFI; BIOS Legacy/CSM non è supportato. Secure Boot è disabilitato in Roma 1.0 perché i costi di firma/certificazione Microsoft non sono sostenibili come prerequisito della prima release. UKI è la direzione scelta per gli artefatti di avvio, subordinata alla verifica dell'integrazione tra kernel Debian 13, initramfs-tools e systemd. L'ESP ospita systemd-boot e le UKI; il sistema installato resta su una sola partizione Btrfs. Non è prevista una partizione XBOOTLDR. Vedi [Avvio systemd-boot e UKI](AVVIO-SYSTEMD-BOOT-E-UKI.md).

## Confini del sistema
- **Aurea Desktop:** superficie desktop, disposizione elementi, selezione e stato di presentazione.
- **Fides:** navigazione e operazioni sui file.
- **Quaero:** ricerca universale e punto di accesso conversazionale al sistema.
- **Aurea Intelligence:** orchestrazione, policy del contesto locale e confini dei provider.
- **Aurea File Intelligence:** catalogo file locale, metadati, indicizzazione e query rispettose dei permessi.
- **Aurea Media:** analisi limitata dei media, decodifica e miniature.
- **Aurea Passkey:** autenticazione di sistema e integrazione credenziali.
- **Aurea Compositor:** superfici Wayland, output, input e composizione Vulkan soltanto.

Le API devono essere piccole ed esplicite. Inizialmente il composition root desktop può ospitare servizi legati alla sessione. Un demone permanente o un servizio privilegiato richiede una motivazione concreta di ciclo di vita, isolamento o accesso multi-client.

## Quaero e Aurea Intelligence
Quaero è l'esperienza universale di ricerca e assistenza del sistema. La direzione è un'interfaccia conversazionale simile a ChatGPT, disponibile a livello di sistema operativo e integrata con Aurea Intelligence, non una semplice casella di ricerca.

Dovrà comprendere richieste in linguaggio naturale relative alle capacità autorizzate: applicazioni, file, impostazioni, assistenza e, in futuro, operazioni consentite. Risultati e azioni manterranno provenienza e controlli sui permessi.

### Modello locale e provider
- Ricerca locale, individuazione applicazioni, impostazioni e operazioni desktop essenziali funzionano offline.
- Aurea Intelligence gestisce orchestrazione, policy del contesto, selezione provider e confini delle capacità.
- ChatGPT può essere un provider remoto configurato esplicitamente, mai una dipendenza nascosta per avvio, login, ricerca di base o funzionamento desktop.
- Le richieste remote indicano il contesto trasmesso e richiedono autorizzazione appropriata.
- Contenuti privati dei file, credenziali, token e dati personali non pertinenti non vengono mai caricati silenziosamente.
- L'output remoto è input non attendibile e non può aggirare autorizzazioni, permessi filesystem, conferme o policy dei comandi.
- Azioni distruttive, finanziarie, sensibili per la sicurezza o visibili all'esterno richiedono conferma esplicita e anteprima.
- Il sistema resta utilizzabile se rete o provider remoto non sono disponibili.

Quaero possiede UX e presentazione dell'intento; Aurea Intelligence orchestrazione e policy; File Intelligence e gli altri servizi i rispettivi dati. L'assistente non deve diventare un secondo indice filesystem o una shell senza restrizioni.

## Aurea Passkey: autenticazione di sistema
Aurea Passkey dovrà offrire un'esperienza coerente di credenziali e autenticazione a livello di sistema, non soltanto un gestore password.

Punti d'integrazione previsti:
- login utente e sblocco sessione;
- elevazione privilegi e richieste di autorizzazione;
- impostazioni di sistema e operazioni sensibili;
- applicazioni tramite API documentate e basate sul consenso;
- chiavi di sicurezza FIDO2 esterne e autenticatori compatibili;
- protezione tramite TPM 2.0 quando supportata da hardware e firmware.

### Architettura di sicurezza
Separare protocollo, archivio credenziali, policy e interfaccia:
1. **Broker autenticazione:** punto d'ingresso di sistema/sessione che media richieste e applica policy.
2. **Adattatori autenticatori:** trasporti FIDO2/CTAP e integrazione con autenticatori di piattaforma.
3. **Archivio credenziali:** conservazione protetta di materiale pubblico e metadati; quando il protocollo lo consente, le chiavi private restano nell'autenticatore.
4. **Adattatore hardware trust:** integrazione TPM 2.0 opzionale per sigillare/proteggere segreti locali. TPM non significa automaticamente che ogni passkey sia memorizzata al suo interno.
5. **Integrazione PAM/sessione:** adattatore circoscritto per login/sblocco ed elevazione, progettato con recupero e fallback.
6. **Esperienza utente:** registrazione, scelta autenticatore, consenso, recupero e stato verificabile.

WebAuthn e CTAP2 sono correlati ma distinti: WebAuthn definisce il flusso relying-party/client; CTAP collega un client a un autenticatore. Un login Linux locale non è automaticamente un flusso WebAuthn relying-party. Riutilizzare librerie solo dopo aver verificato ambito, licenza, MSRV, trasporti e sicurezza.

Nella prima versione una passkey non sarà l'unico metodo di recupero. Occorrono recupero sicuro, amministrazione account e fallback documentato. Non inventare crittografia proprietaria e non conservare chiavi private in normali configurazioni applicative. Il supporto TPM è opzionale e rilevato in base alle capacità. Il compositor non possiede segreti delle credenziali.

## Btrfs: filesystem di sistema e Aurea Time
Btrfs è stato scelto come filesystem Linux moderno per AureaOS. La scelta non implica l'uso degli snapshot Btrfs nativi, aggiornamenti basati su snapshot o rollback.

Il layout del disco installato è volutamente semplice:
- Una EFI System Partition (ESP) per gli artefatti di avvio UEFI.
- Una partizione Btrfs contenente l'intero sistema operativo installato.
- Nessuna partizione separata per `/home`, `/var`, cache, snapshot o backup.
- Nessuna partizione XBOOTLDR nell'attuale architettura di avvio.

Aurea Time è l'unico sistema AureaOS previsto per backup e ripristino. La destinazione sarà un'unità USB esterna o un NAS e l'esperienza dovrà consentire il recupero nel tempo dai singoli file e cartelle, fino alle impostazioni e all'intero sistema operativo. Implementazione, modello di consistenza, formato di archiviazione, conservazione e flusso di ripristino completo restano da progettare.

È necessario un ambiente indipendente Aurea Recovery, così da poter ripristinare il sistema operativo anche quando il sistema installato non si avvia. Il recupero non deve dipendere dagli snapshot Btrfs.

La configurazione delle funzionalità Btrfs — inclusi sottovolumi, compressione, scrub, quote, RAID, cifratura e swap/ibernazione — non è stata selezionata in questa baseline. Ogni funzionalità richiede una motivazione tecnica separata e una decisione esplicita.

## File Intelligence e dati locali
Aurea File Intelligence resta il catalogo locale condiviso per Fides, Quaero e client autorizzati. Deve rispettare i permessi, essere incrementale, limitato nelle risorse e ricostruibile.

SQLite resta il candidato principale per il database embedded, ma non è ancora una dipendenza runtime approvata finché FTS5, binding Rust, MSRV, linking, packaging e recupero non saranno verificati su Debian e Fedora. I metadati scritti dall'utente restano separati dai dati d'indice ricostruibili.

La ricerca iniziale privilegia nome, percorso, MIME, metadati economici e testo indicizzato esplicitamente. Ricerca semantica/vettoriale ed estrazione tramite modelli sono livelli opzionali successivi, non prerequisiti.

## Risorse, privacy e affidabilità
- I percorsi di rendering/input non devono eseguire indicizzazione, decodifica, chiamate AI o I/O bloccante.
- Usare worker limitati, cancellazione, backpressure e budget osservabili.
- Preferire elaborazione locale; rendere esplicita quella remota.
- Rispettare permessi filesystem a ogni confine di query/azione.
- Trattare file, metadati, output dei modelli e dispositivi esterni come input non attendibili.
- Mostrare onestamente stati offline, non aggiornati o degradati.
- Mantenere API piccole, indipendenti dai framework UI e versionate quando necessario.
- Evitare moduli enormi: un file, una responsabilità.

## Note di valutazione tecnologica
- Debian 13 fornisce pacchetti SQLite runtime e sviluppo. I metadati dei pacchetti Fedora indicano FTS5 abilitato. Il solo elenco pacchetti Debian non prova le opzioni di compilazione della libreria installata: verificare FTS5 sull'immagine destinazione.
- La documentazione corrente di notify 9 indica supporto Linux tramite inotify, licenza CC0 e MSRV 1.91. Il workspace esaminato usa Rust edition 2024 ma non dichiara rust-version. Non adottare notify 9 finché la policy toolchain/MSRV non è esplicita; resta valida l'alternativa di un adattatore inotify nativo.
- Le librerie Rust passkey hanno ambiti diversi. La famiglia passkey-rs di 1Password copre componenti client WebAuthn e autenticatore CTAP2; le librerie FIDO Linux hanno vincoli distinti su trasporti e licenze. Nessuna crate è approvata qui.
- Gli snapshot Btrfs nativi non fanno parte del flusso pianificato di backup, aggiornamento o ripristino di AureaOS.

## Sequenza di realizzazione
1. Fissare principi piattaforma e confini di responsabilità.
2. Definire i contratti di backup/ripristino di Aurea Time e l'ambiente indipendente Aurea Recovery.
3. Definire threat model Aurea Passkey, broker, recupero e strategia PAM.
4. Definire provider Aurea Intelligence, privacy e autorizzazione azioni.
5. Completare valutazione compatibilità database/watcher File Intelligence.
6. Implementare servizi piccoli e verificabili indipendentemente, integrandoli progressivamente.
7. Validare su Debian 13 e hardware Fedora, incluse macchine limitate e uso offline.

Questo documento non implica che le funzionalità siano implementate, compilate, testate o disponibili a runtime.


## Proposta layout Btrfs
Un documento dedicato propone un layout piatto di sottovolumi fratelli: @root su /, @home su /home, @snapshots su un mount dedicato e i candidati @var-log e @var-cache. Mantenere /var/lib dentro @root affinché dpkg e lo stato dei servizi restino coerenti con i file installati. Preservare /home e i log diagnostici per impostazione predefinita. Bootloader/ESP, cifratura, conservazione, policy cache e swap/ibernazione restano aperti. È una proposta da validare, non una specifica installer.
