<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Progettazione Btrfs e ripristino di AureaOS

## Stato
Proposta architetturale da sottoporre a revisione. Non è una specifica definitiva dell'installer né un sistema di recupero già implementato.

## Obiettivi
- Ripristinare deliberatamente lo stato del sistema operativo.
- Preservare documenti e dati personali durante il rollback del sistema.
- Mantenere coerente lo stato del package manager con i file installati.
- Conservare log diagnostici dopo aggiornamenti falliti.
- Rendere esplicita la copertura degli snapshot.
- Evitare inizialmente sottovolumi annidati.
- Considerare gli snapshot punti di recupero locali, mai backup.

## Layout iniziale proposto

Usare sottovolumi fratelli e piatti sotto l'albero top-level Btrfs (subvolume ID 5). Montare il root selezionato su / e gli altri sottovolumi in percorsi espliciti. Non annidare sottovolumi.

| Sottovolume | Punto mount | Policy | Motivazione |
|---|---|---|---|
| @root | / | Snapshot e rollback | File OS, /usr, /etc, /opt, /var/lib, database pacchetti e stato Flatpak di sistema restano coerenti |
| @home | /home | Preservare per impostazione predefinita | Documenti e impostazioni personali non devono tornare indietro silenziosamente |
| @snapshots | /.snapshots (candidato) | Preservare | Catalogo snapshot fuori dallo snapshot root |
| @var-log | /var/log | Preservare | La diagnostica di un aggiornamento fallito sopravvive al rollback |
| @var-cache | /var/cache | Rigenerabile; policy da definire | La cache è generalmente ricostruibile e non deve necessariamente tornare indietro |
| EFI System Partition | /boot/efi (se scelta) | Fuori dagli snapshot Btrfs | I file firmware di avvio risiedono su partizione FAT separata |

Mantenere /var/lib dentro @root. Contiene stato del package manager e dei servizi che deve corrispondere allo snapshot di sistema. In particolare, il database dpkg Debian deve tornare indietro insieme ai file installati. Lo stato Flatpak installato a livello di sistema segue la stessa regola; i dati Flatpak utente restano nella home.

La posizione di /boot e l'integrazione bootloader sono aperte. Se /boot è dentro @root, kernel e initramfs sono inclusi negli snapshot root. La EFI System Partition resta fuori dagli snapshot Btrfs e richiede una propria strategia di recupero.

Il mount /.snapshots è una proposta, non una dipendenza da uno specifico gestore snapshot. Lo storage degli snapshot non deve essere annidato in @root.

## Perché un layout piatto
Gli snapshot Btrfs non sono ricorsivi. Un sottovolume annidato costituisce un confine: i suoi contenuti non compaiono nello snapshot del sottovolume genitore. Un layout piatto rende esplicita la copertura ed evita omissioni accidentali. Il gestore di sistema dovrà coordinare ogni eventuale sottovolume aggiuntivo incluso intenzionalmente nella transazione.

## Ciclo di vita della transazione di aggiornamento
Un aggiornamento deve essere una transazione con stati espliciti, non uno script shell che crea incidentalmente uno snapshot.

1. **Preflight:** verificare identità filesystem, layout atteso, spazio libero, destinazione snapshot, modalità avvio e stato package manager.
2. **Quiescenza:** impedire transazioni sovrapposte; coordinare servizi che mantengono database o stato mutabile quando necessario.
3. **Snapshot:** creare uno snapshot read-only pre-aggiornamento di @root. Registrare ID/UUID sottovolume, data, identità transazione pacchetti e contesto configurazione boot.
4. **Applicazione:** eseguire la transazione Debian approvata sul root attivo.
5. **Validazione:** verificare completamento package manager, artefatti boot necessari, generazione initramfs e configurazione critica.
6. **Commit:** rendere corrente la nuova generazione solo dopo la validazione; conservare lo snapshot precedente secondo policy.
7. **Errore:** lasciare intatto lo snapshot precedente, registrare diagnostica e offrire recupero controllato.

Integrazione package manager, lock transazionale, criteri di validazione e selezione atomica del boot richiedono ancora progettazione dettagliata.

## Modello di rollback
Il rollback deve avvenire da un ambiente di recupero o da un contesto equivalente nel quale il root destinazione non sia attivo.

Deve:
- identificare gli snapshot tramite metadati stabili, non solo nomi visualizzati;
- verificare identità filesystem e sottovolume prima di modificare;
- preservare il root fallito come punto diagnostico quando lo spazio lo consente;
- creare un nuovo root scrivibile dallo snapshot read-only selezionato, senza modificare l'originale;
- aggiornare la selezione boot tramite integrazione bootloader collaudata;
- mantenere coerenti artefatti boot/EFI con il root ripristinato;
- registrare operazione e risultato;
- fornire una voce alternativa di avvio/recupero.

Non ripristinare mai /home silenziosamente. Una futura funzione di recupero dati utente richiederà anteprima e conferma esplicita.

## Conservazione e pressione sullo spazio
La conservazione è ancora da decidere. Deve considerare almeno un punto di recupero noto e funzionante, snapshot pre-aggiornamento con identità transazione pacchetti, eventuali snapshot manuali, limiti per quantità/età, spazio libero, pressione metadati e pulizia sicura. Non eliminare mai root attivo o unico punto di recupero.

Se il preflight non può garantire spazio sufficiente, fermare l'aggiornamento prima di modificare i pacchetti. Il fallimento nella creazione snapshot non deve essere ignorato.

## Backup, cifratura e integrità
- Gli snapshot non sono backup: un danno al filesystem o dispositivo può colpire dati attivi e snapshot.
- Serve una strategia separata di backup/esportazione. Btrfs send/receive può essere valutato, ma non costituisce da solo una policy completa.
- Non presumere che Btrfs fornisca cifratura trasparente nativa dei dati a riposo. Valutare LUKS2/dm-crypt come livello separato di installazione e recupero. Sblocco assistito da TPM richiede threat model esplicito e progetto delle chiavi di recupero.
- Definire separatamente pianificazione scrub, salute dispositivi e comportamento filesystem degradato.
- Non promettere autoriparazione o ridondanza senza profilo multi-dispositivo supportato e policy collaudata.

## Policy mount e prestazioni
Validare le opzioni mount esatte su hardware e carichi di lavoro destinazione. Valutare compress=zstd, noatime/relatime, discard e comportamento SSD. Non impostare nodatacow globalmente: modifica checksum e copy-on-write.

Swapfile e ibernazione richiedono progettazione e test di compatibilità separati. Non creare uno swapfile Btrfs predefinito finché vincoli, configurazione resume e recupero non sono validati.

## Requisiti installer e recupero
L'installer deve identificare disco destinazione e operazioni distruttive, creare filesystem top-level e sottovolumi nominati in modo deterministico, installare tabella mount e configurazione bootloader documentate, registrare un manifest layout/versione leggibile da macchina, verificare mount e identità sottovolumi prima del primo avvio e fornire recupero in caso di installazione interrotta.

Gli strumenti di recupero devono ispezionare UUID filesystem, UUID/ID sottovolume e topologia mount. Il solo nome di un sottovolume non ne dimostra l'identità.

## Decisioni aperte
- GRUB o systemd-boot e integrazione delle voci di avvio snapshot.
- Se /boot resta dentro @root e layout ESP preciso.
- Policy @var-cache ed eventuale ulteriore separazione di /var.
- Impostazioni predefinite LUKS2 e UX chiavi recupero.
- Limiti conservazione snapshot e soglie spazio libero.
- Supporto swapfile e ibernazione.
- Profili single-disk e multi-device supportati.
- Servizio transazionale aggiornamenti dedicato e responsabilità API package manager.

## Sequenza di realizzazione
1. Validare grafo sottovolumi e tabella mount su Debian 13.
2. Specificare bootloader e ambiente recupero prima del rollback.
3. Definire stati transazione, lock, gestione errori e validazione.
4. Definire metadati snapshot e conservazione.
5. Specificare cifratura, backup, scrub e salute storage.
6. Implementare installer/recupero solo dopo revisione dei contratti.
7. Testare aggiornamenti interrotti, boot fallito, spazio insufficiente, rollback, mismatch EFI e conservazione dati utente su sistemi sacrificabili.

## Riferimenti
- Documentazione Btrfs — Subvolumi: https://btrfs.readthedocs.io/en/latest/Subvolumes.html
- btrfs-subvolume(8): https://btrfs.readthedocs.io/en/latest/btrfs-subvolume.html

Questo documento non dichiara che installer, gestore snapshot, transazioni aggiornamento o rollback siano implementati.
