<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# systemd-boot e Unified Kernel Images

## Stato

**Decisione di piattaforma approvata:** systemd-boot è il boot manager principale di AureaOS Roma 1.0.

**Vincoli della release:** solo UEFI; BIOS Legacy/CSM non supportato. Secure Boot disabilitato nella prima release. GRUB 2 non fa parte dell'installazione predefinita e potrà essere rivalutato soltanto se in futuro emergerà una necessità dimostrata.

UKI è la direzione architetturale scelta per gli artefatti di avvio del kernel, subordinata alle verifiche d'implementazione e compatibilità elencate sotto. Questo documento descrive architettura e requisiti; non afferma che installer, pipeline di avvio o ambiente di ripristino siano già implementati.

## Perché systemd-boot

AureaOS si rivolge a computer UEFI moderni e a Debian GNU/Linux 13 Trixie. systemd-boot è un boot manager UEFI essenziale, mantenuto nell'ecosistema systemd e disponibile nei pacchetti Debian 13. Il suo ambito circoscritto è più adatto di quello più ampio di GRUB 2 quando AureaOS non supporta il vecchio BIOS.

La scelta dipende dalla coerenza con la piattaforma, non dalla novità. GRUB 2 resta un bootloader maturo e tecnicamente valido, ma non è l'architettura predefinita di AureaOS.

## Firmware e hardware di destinazione

- Il firmware UEFI è obbligatorio.
- BIOS Legacy e avvio UEFI CSM non sono supportati.
- Il target iniziale è costituito da PC amd64 moderni, indicativamente degli ultimi dieci anni, purché implementino un ambiente UEFI utilizzabile.
- L'età del computer da sola non garantisce la compatibilità. Qualità del firmware, implementazione UEFI, controller di archiviazione, grafica e altri requisiti hardware devono essere verificati separatamente.
- L'installer deve rilevare e segnalare chiaramente quando è stato avviato in modalità Legacy/CSM; non deve installare silenziosamente un sistema incapace di utilizzare il percorso di avvio scelto.

La revisione UEFI minima esatta e la matrice hardware restano subordinate alla qualificazione hardware di Roma 1.0.

## Politica Secure Boot

Secure Boot è **disabilitato in Roma 1.0**. Il progetto non renderà la firma Microsoft, l'inserimento della Microsoft Third-Party UEFI CA o un accordo commerciale di certificazione/firma un prerequisito della release finché i relativi costi non saranno sostenibili.

È una scelta esplicita di ambito della prima release, non un'omissione accidentale. L'installer e la documentazione devono spiegare che per la release iniziale Secure Boot deve essere disabilitato nel firmware. Il sistema non deve dichiarare di offrire protezione Secure Boot o verifica della catena di firme quando questa funzione è disabilitata.

L'architettura deve evitare decisioni che rendano inutilmente difficile un futuro supporto. Possiamo predisporre generazione UKI, provenienza degli artefatti, considerazioni sulla riproducibilità e un futuro confine di firma; tuttavia una UKI non firmata non diventa attendibile soltanto perché è un'immagine unificata. Prima di abilitare Secure Boot in una release successiva, AureaOS dovrà definire proprietà delle chiavi, autorità di firma, revoca e procedure di recupero, compatibilità firmware e un percorso sostenibile per distribuire binari di avvio considerati attendibili. Chiavi gestite dal proprietario e percorso di firma riconosciuto da Microsoft sono opzioni distinte da valutare; questo documento non ne approva nessuna.

## Artefatti di avvio: direzione UKI

Una Unified Kernel Image riunisce kernel Linux, initramfs e metadati di avvio selezionati (come riga di comando del kernel e identificazione del sistema operativo) in un unico eseguibile EFI. Debian 13 fornisce il pacchetto `systemd-ukify`, mentre `systemd-boot` e i relativi strumenti sono disponibili in Trixie.

La direzione prevista è:
- Generare una UKI versionata per ogni kernel supportato.
- Gestire la riga di comando del kernel tramite configurazione AureaOS, non tramite frammenti arbitrari dell'installer o della shell utente.
- Includere soltanto i contenuti initramfs necessari e il microcodice hardware quando applicabile.
- Conservare almeno un precedente artefatto di avvio noto come funzionante durante la transizione di kernel/aggiornamento, secondo policy di spazio e conservazione.
- Verificare contenuto e metadati della UKI prima di renderla predefinita.
- Non incorporare segreti, chiavi private, dati utente o stato applicativo mutabile in una UKI.

L'integrazione esatta tra pacchetti kernel Debian, initramfs-tools, `kernel-install`, `ukify` e Aurea Update deve essere prototipata su Trixie prima di considerare completa l'implementazione.

## Layout disco ed ESP

Il layout del sistema installato resta volutamente semplice:
- Una EFI System Partition (ESP), formattata con un filesystem FAT compatibile UEFI.
- Una partizione Btrfs contenente l'intero sistema AureaOS installato.
- Nessuna partizione XBOOTLDR.
- Nessuna partizione separata per `/home`, `/var`, cache, snapshot o backup.

systemd-boot e le UKI risiederanno nell'ESP. Btrfs resta il filesystem di sistema; non si deve presumere che firmware e systemd-boot possano leggere direttamente da Btrfs gli artefatti del kernel.

L'ESP deve avere spazio sufficiente per boot manager, UKI corrente, almeno una UKI precedente/di fallback e l'artefatto di avvio indipendente di Aurea Recovery, se tale disegno sarà confermato. **La dimensione finale dell'ESP non è ancora fissata.** 1 GiB è un obiettivo provvisorio da validare misurando le dimensioni reali di kernel, initramfs, UKI e artefatto Recovery su hardware supportato e durante gli aggiornamenti. Non ridurre silenziosamente questa dimensione a una ESP convenzionale più piccola senza dati.

L'installer deve verificare lo spazio libero prima di scrivere una nuova UKI e non deve mai eliminare l'unica voce di avvio nota come funzionante per liberare spazio durante un aggiornamento.

## Voci di avvio, aggiornamenti e fallback

systemd-boot supporta voci Boot Loader Specification, UKI e conteggio dei tentativi con fallback a voci precedenti quando una voce non riesce ripetutamente a completare l'avvio. AureaOS deve definire il segnale userspace che conferma il successo: il boot manager non può dimostrare autonomamente che desktop, servizi o sessione utente siano sani.

Aurea Update e il gestore degli artefatti di avvio dovranno coordinare:
1. Preparazione del nuovo kernel/initramfs/UKI senza sostituire l'artefatto noto come funzionante.
2. Validazione dell'artefatto e dei metadati.
3. Pubblicazione di una voce con un numero limitato di tentativi.
4. Avvio della nuova voce.
5. Conferma del corretto avvio tramite un servizio esplicito, dopo i controlli di salute richiesti.
6. Conservazione o selezione della precedente voce funzionante se il nuovo avvio fallisce ripetutamente.
7. Pulizia degli artefatti più vecchi soltanto secondo policy definite di conservazione e spazio libero.

È un ciclo di vita obiettivo, non una funzionalità implementata. Gli hook del package manager non devono entrare in conflitto con Aurea Update né eliminare artefatti ancora referenziati dalle voci di avvio.

## Aurea Recovery

Aurea Recovery è un ambiente di ripristino separato, non uno snapshot Btrfs e non un secondo sistema di backup.

Il disegno previsto è un'immagine di ripristino avviabile in modo indipendente, selezionabile da systemd-boot e capace di ripristinare il sistema installato tramite Aurea Time anche quando la root installata non si avvia. L'artefatto Recovery sul disco non deve essere l'unico percorso: se l'archiviazione interna è danneggiata o non disponibile, servono supporti esterni di ripristino.

L'ambiente Recovery deve essere progettato e validato separatamente per:
- avviarsi senza dipendere dalla root installata;
- rilevare il disco di destinazione e la singola partizione Btrfs del sistema;
- individuare un backup Aurea Time autorizzato su USB o NAS;
- ripristinare in sicurezza file di sistema, impostazioni, metadati e artefatti di avvio;
- reinstallare o riparare systemd-boot e le voci EFI;
- segnalare gli errori senza distruggere l'unico backup disponibile.

## Confini di responsabilità

- **Firmware UEFI:** inizializza l'hardware e avvia l'eseguibile EFI selezionato.
- **systemd-boot:** presenta le scelte di avvio e lancia l'immagine EFI selezionata.
- **Pipeline UKI/aggiornamento:** crea, valida, prepara e rimuove gli artefatti di avvio.
- **Initramfs:** individua e prepara la root del sistema installato.
- **Aurea Recovery:** fornisce un ambiente indipendente di manutenzione e ripristino.
- **Aurea Time:** gestisce cronologia e dati di backup, dai singoli file all'intero sistema operativo.
- **Aurea Update:** coordina aggiornamenti del sistema e transizioni degli artefatti di avvio.

Il boot manager non deve implementare policy di backup, gestire credenziali Passkey o orchestrare servizi desktop.

## Predisposizione Secure Boot e futura attivazione

Secure Boot resta disabilitato per impostazione predefinita e non è un criterio di accettazione di Roma 1.0. L'architettura di avvio deve tuttavia essere progettata affinché una release successiva possa abilitarlo senza sostituire boot manager, formato UKI, layout dell'installer o ciclo di vita degli aggiornamenti.

### Direzione della catena di fiducia

Debian 13 Trixie fornisce già pacchetti per un percorso Secure Boot con systemd-boot:
- `shim-signed` fornisce un primo avviatore firmato da una Microsoft UEFI CA.
- Debian fornisce helper shim firmati e un pacchetto systemd-boot firmato per le architetture supportate.
- `systemd-ukify` può creare UKI e firmare l'immagine PE finale con un certificato Secure Boot.

La catena futura prevista è:

1. Il firmware UEFI verifica shim firmato da Microsoft, se il computer si fida della relativa Microsoft UEFI CA.
2. Shim verifica e avvia systemd-boot firmato da Debian usando il certificato di distribuzione attendibile.
3. La policy dello stadio successivo verifica la UKI AureaOS con un certificato di firma AureaOS esplicitamente attendibile, registrato tramite un percorso supportato.
4. La UKI contiene kernel, initramfs e riga di comando controllata, così la firma copre l'intero payload di avvio invece di kernel e initramfs modificabile separatamente.

Questo è il modello di fiducia obiettivo, non l'affermazione che la catena sia già integrata o testata. In particolare, l'interazione esatta tra systemd-boot di Trixie, database MOK di shim, verifica delle immagini UEFI e firme UKI deve essere dimostrata su firmware reali prima di scegliere il percorso definitivo di registrazione.

### Opzioni per registrare la fiducia

Devono essere valutati due percorsi distinti:

- **Registrazione da parte del proprietario:** l'utente registra esplicitamente il certificato pubblico AureaOS nel database Secure Boot del firmware oppure tramite il flusso Machine Owner Key (MOK) di shim, se la catena selezionata lo supporta. Questo può evitare che AureaOS debba ottenere un proprio rapporto di firma Microsoft, ma richiede un passaggio chiaro e sicuro su ogni dispositivo e firmware compatibile.
- **Distribuzione ampiamente pre-attendibile:** AureaOS ottiene un accordo di firma/distribuzione che renda il suo primo componente di avvio attendibile su un'ampia gamma di PC standard senza registrazione di una chiave personalizzata su ogni dispositivo. È una decisione futura distinta, commerciale e di rilascio; non bisogna presumere che sia gratuita, garantita o necessaria per ogni installazione Secure Boot.

Lo shim Microsoft-signed di Debian è una possibile base, non una garanzia di compatibilità universale. Alcuni firmware disabilitano la Microsoft UEFI CA di terze parti, contengono generazioni differenti di CA o presentano problemi firmware/revoche. L'installer deve rilevare e spiegare gli errori della catena di fiducia, senza promettere silenziosamente supporto universale.

### Architettura di firma da predisporre ora

La pipeline degli artefatti di avvio deve esporre un confine di firma indipendente dalla creazione UKI:
- Creare e validare prima la UKI; la firma è una fase distinta di finalizzazione.
- Roma 1.0 usa un profilo esplicito di sviluppo/rilascio non firmato, dato che Secure Boot è disabilitato.
- Un futuro profilo Secure Boot firma la UKI finale dopo ogni modifica al contenuto. Non modificare mai l'immagine PE dopo la firma.
- Separare identità di firma di sviluppo, test e produzione.
- Tenere le chiavi private di produzione fuori dai sistemi installati, dalle home degli utenti e dai normali log/artefatti CI. Usare custodia offline o hardware-backed controllata e accesso alla firma limitato.
- Distribuire soltanto certificati pubblici e materiale di registrazione; mai le chiavi private.
- Verificare firma e digest dell'artefatto prima di pubblicare o installare una UKI firmata.
- Definire scadenza, rinnovo, rotazione, revoca d'emergenza, risposta alla compromissione e recupero prima di abilitare la funzionalità.
- Includere e mantenere metadati SBAT dove applicabile e seguire gli aggiornamenti di revoca firmware DBX e shim.
- Trattare firma/sealing delle policy PCR TPM come funzionalità opzionale separata; non è necessaria per abilitare Secure Boot.

Aurea Update deve conservare l'ultimo artefatto firmato noto come avviabile mentre prepara e valida il sostituto. Un errore di verifica della firma deve impedire la promozione del nuovo artefatto e non deve eliminare la precedente voce funzionante. Aurea Recovery deve essere avviabile secondo la policy di fiducia scelta oppure offrire un percorso di ripristino esterno documentato separatamente.

### Comportamento di Roma 1.0

- Secure Boot non è richiesto e resta disabilitato nel profilo d'installazione supportato.
- La pipeline UKI dovrebbe già usare lo stesso formato di artefatto UKI e le stesse fasi di validazione che userà il futuro profilo firmato.
- Per Roma 1.0 non viene generata né incorporata nei PC degli utenti alcuna chiave di firma Secure Boot di produzione.
- Non registrare chiavi AureaOS, modificare variabili firmware DB/KEK/PK o cambiare automaticamente lo stato Secure Boot.
- Non descrivere una UKI non firmata come protetta da Secure Boot.

### Verifiche necessarie prima della futura attivazione

1. Verificare versioni esatte Trixie amd64 e comportamento dei pacchetti `shim-signed`, `shim-helpers-amd64-signed`, `systemd-boot`, `systemd-boot-efi-signed`, `systemd-ukify`, `mokutil` e strumenti di firma.
2. Dimostrare la catena completa shim → systemd-boot → UKI firmata AureaOS con Secure Boot attivo su firmware rappresentativi.
3. Confermare se la registrazione MOK è rispettata dal percorso esatto di caricamento UKI; in caso contrario usare e testare la registrazione esplicita nel DB firmware o un altro percorso supportato.
4. Testare registrazione, annullamento, rimozione, rotazione e recupero delle chiavi, inclusi PC con Microsoft third-party CA disabilitata e database CA più recenti.
5. Testare firme revocate/scadute, aggiornamenti DBX e SBAT, aggiornamenti UKI interrotti e fallback.
6. Validare Aurea Recovery indipendente secondo la stessa policy Secure Boot.
7. Decidere se la registrazione da parte del proprietario su ogni dispositivo è accettabile o se serve una distribuzione ampiamente pre-attendibile.
8. Solo allora abilitare Secure Boot come profilo d'installazione opzionale o predefinito in una release successiva.

Questa decisione documentale non modifica chiavi di firma, variabili firmware, stato Secure Boot o artefatti di avvio.


## Verifiche richieste prima di congelare l'implementazione

1. Verificare l'esatto insieme di pacchetti Debian 13 e le interfacce systemd 257 supportate nell'immagine amd64 di destinazione.
2. Prototipare la generazione UKI dai pacchetti kernel Debian e initramfs-tools, inclusi microcodice e gestione della riga di comando.
3. Misurare l'uso ESP per UKI corrente, precedente e Recovery; validare l'obiettivo provvisorio di 1 GiB.
4. Testare installazione, aggiornamento, interruzione aggiornamento, conteggio avvii falliti, fallback e pulizia.
5. Testare il percorso firmware di fallback e creazione/rimozione delle voci NVRAM.
6. Testare l'avvio Recovery quando la root Btrfs installata è danneggiata o non disponibile.
7. Testare il ripristino del sistema operativo e della catena di avvio da una destinazione Aurea Time esterna.
8. Documentare Secure Boot disabilitato in Roma 1.0 e mantenere il lavoro futuro di firma fuori dai criteri di accettazione della prima release.

Questo documento non implica build, installazione di pacchetti, modifica del firmware o test hardware.
