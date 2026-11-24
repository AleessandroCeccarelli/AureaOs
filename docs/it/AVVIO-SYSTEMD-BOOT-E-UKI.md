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
