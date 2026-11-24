<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Scelta di Btrfs e layout del disco di sistema

## Stato

Direzione approvata per filesystem e partizionamento. Questo documento non specifica l'implementazione dell'installer né la configurazione di funzionalità Btrfs non approvate.

## Perché Btrfs

Btrfs è stato scelto come filesystem Linux moderno per AureaOS. Non è stato scelto per introdurre un'architettura di backup, aggiornamento o rollback basata su snapshot.

## Layout del disco installato

Il layout previsto è:
- Una EFI System Partition (ESP), formattata per il firmware UEFI.
- Una partizione Btrfs contenente l'intero sistema operativo installato.
- Nessuna partizione separata per `/home`, `/var`, cache, snapshot o backup.
- Nessuna partizione XBOOTLDR nell'attuale architettura systemd-boot.

La dimensione esatta dell'ESP, le opzioni di mount e il comportamento dell'installer richiedono verifiche. La partizione Btrfs è l'unica partizione del filesystem di sistema; l'archiviazione esterna dei backup non fa parte del layout del disco di sistema.

## Responsabilità di backup e ripristino

Aurea Time è l'unico sistema AureaOS previsto per backup e ripristino. Dovrà supportare backup versionati nel tempo su unità USB esterne o NAS e recupero dai singoli file e cartelle fino alle impostazioni e all'intero sistema operativo.

Un ambiente indipendente Aurea Recovery dovrà avviarsi senza dipendere dal sistema installato e ripristinarlo utilizzando un backup Aurea Time autorizzato. Formato di archiviazione, modello di consistenza, conservazione, policy di cifratura e procedura di ripristino dettagliata restano da progettare.

## Esclusioni esplicite

- Gli snapshot Btrfs nativi non sono il meccanismo pianificato di backup o ripristino.
- Non è previsto un flusso di aggiornamento o rollback basato su snapshot.
- Non è prevista una partizione dedicata a snapshot o backup.
- Non sono previste partizioni separate per `/home`, `/var` o cache.

Funzionalità Btrfs come sottovolumi, compressione, scrub, quote, RAID, cifratura e swap/ibernazione non sono selezionate da questo documento. Richiedono valutazione separata e approvazione esplicita.

## Architetture correlate

- [systemd-boot e UKI](AVVIO-SYSTEMD-BOOT-E-UKI.md)
- Aurea Time gestisce cronologia dei backup e comportamento di ripristino.
- Aurea Recovery fornisce l'ambiente indipendente per il ripristino completo del sistema.

Questo documento registra decisioni architetturali; non afferma che installazione, backup o ripristino siano implementati.
