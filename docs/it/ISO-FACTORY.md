<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS ISO Factory

**Stato:** procedura operativa e specifica obiettivo  
**Ultima revisione:** 26 novembre 2026  
**Repository:** `AleessandroCeccarelli/AureaOs`  
**Branch principale:** `main`  
**Piattaforma prevista:** Debian GNU/Linux 13 (Trixie), amd64

Questo documento definisce il ciclo ricorrente che trasforma lo stato verificato del repository AureaOS in una nuova ISO Live. Distingue esplicitamente ciò che è già presente nel repository, ciò che deve ancora essere migliorato e ciò che va verificato su hardware reale. Il registro cronologico dettagliato rimane nel [diario di bordo](DIARIO-DI-BORDO.md).

## 1. Scopo

ISO Factory è la procedura stabile e ripetibile con cui produciamo una nuova ISO Live a partire dallo stato corrente e verificato di AureaOS.

Il ciclo settimanale è il seguente:

1. Durante la settimana sviluppiamo e pubblichiamo le modifiche su GitHub.
2. Venerdì o sabato sincronizziamo Fedora con `origin/main`.
3. Su Fedora eseguiamo formattazione, compilazione e test Rust.
4. Generiamo la ISO da quello specifico stato locale del repository usando gli script mantenuti.
5. Eseguiamo automaticamente i controlli sul contenuto di boot e sulla struttura ISO.
6. Avviamo l'immagine su hardware fisico e registriamo il risultato.
7. Conserviamo ISO, checksum, manifest di build, report di validazione e log.

L'obiettivo è aggiornare il sistema ogni settimana senza riscrivere la procedura e senza perdere la storia delle immagini di prova precedenti.

## 2. Regole operative non negoziabili

- **Un'unica fonte di verità:** il sorgente previsto per la release è il commit esatto di `main` sincronizzato su Fedora e sottoposto ai controlli locali concordati.
- **Nessuna riscrittura settimanale degli script:** gli script esistenti si migliorano quando emerge un difetto verificato o un nuovo requisito; non si crea ogni settimana una procedura usa-e-getta.
- **Nessuna dichiarazione di successo non verificata:** la conclusione positiva di `live-build` o la presenza di un file ISO non dimostrano che il sistema Live si avvii.
- **Conservazione delle prove:** le build fallite devono mantenere workspace e diagnostica. Anche dopo una build riuscita workspace e cache vanno conservati, salvo richiesta esplicita di pulizia.
- **Nessuna perdita dello storico:** le immagini candidate precedenti e i relativi report devono restare identificabili tramite commit e ID di build.
- **Fedora resta il riferimento per i controlli locali:** formattazione, compilazione e test Rust concordati vengono eseguiti sul computer Fedora. Una modifica su GitHub non dimostra che tali controlli siano passati.
- **Modularità:** ogni script o modulo ha una sola responsabilità; il coordinatore deve restare contenuto e delegare i controlli a script specifici.
- **Rispetto dell'architettura AureaOS:** Debian Trixie amd64; desktop Wayland-only, Vulkan-only e basato su Smithay. La pipeline non deve introdurre X11, XWayland, OpenGL, GNOME, KDE o XFCE come dipendenze del prodotto AureaOS.
- Si lavora su `main`, salvo diversa richiesta esplicita. Non si eliminano modifiche locali non committate per forzare la sincronizzazione.

## 3. Ciclo settimanale

### Fase A — Sviluppo su GitHub (lunedì-venerdì)

- Lavoriamo sul branch `main`.
- Manteniamo commit circoscritti e messaggi descrittivi.
- Aggiorniamo la documentazione tecnica e il diario quando cambiano decisioni, comportamento, limitazioni o risultati delle verifiche.
- Durante le normali sessioni dedicate al desktop non generiamo una ISO ogni giorno.
- Non consideriamo completata una funzionalità solo perché sono stati aggiunti API, file, specifiche o test: distinguiamo prototipo, implementazione, verifica automatica e verifica hardware.

### Fase B — Sincronizzazione Fedora (venerdì o sabato)

Dalla cartella locale del repository AureaOS su Fedora:

```bash
git status --short --branch
git switch main
git pull --ff-only origin main
git status --short --branch
git rev-parse HEAD
git log -1 --oneline --decorate
```

Prima dei controlli devono essere vere queste condizioni:

- Il branch corrente è `main`.
- Il pull è terminato con fast-forward oppure ha confermato che il branch è già aggiornato.
- Non ci sono modifiche locali inattese non committate.
- L'ID completo del commit viene registrato per la verifica settimanale.

Se il working tree contiene modifiche, ci fermiamo e le esaminiamo. Non eseguiamo automaticamente reset, clean o sovrascritture. Prima di proseguire, le modifiche devono essere risolte o conservate consapevolmente.

### Fase C — Controlli qualità Rust su Fedora

Eseguiamo i controlli in questo ordine:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
```

Aggiungiamo i controlli specifici delle funzionalità effettivamente coinvolte, ad esempio il controllo DRM del compositor quando pertinente:

```bash
cargo check -p aurea-compositor --features drm
```

Gli ulteriori comandi dipendono dall'implementazione corrente e dall'hardware supportato. Non dichiariamo superato un comando se non ne abbiamo osservato l'output e il codice di uscita. Registriamo con precisione errori, test ignorati e test saltati perché dipendenti dall'hardware.

**Condizione di accesso alla build:** non iniziamo la produzione ISO se un controllo obbligatorio di formattazione, compilazione o test fallisce. Un'eventuale eccezione deve essere decisa esplicitamente e annotata nel manifest e nel diario.

### Fase D — Generazione della ISO

L'entry point mantenuto per Fedora è il wrapper Podman:

```bash
bash tools/iso/build-live-iso-podman.sh
```

Attualmente il wrapper:
- utilizza un container Debian Trixie slim per l'ambiente live-build;
- monta il repository in sola lettura;
- usa per impostazione predefinita una directory persistente distinta per ogni commit sorgente e un lock del workspace;
- rifiuta la build se il repository contiene modifiche non committate o file non tracciati, perché lo snapshot viene creato con `git archive HEAD`;
- delega la generazione effettiva a `tools/iso/build-live-iso.sh`;
- pubblica una candidata storica con nome univoco, manifest JSON e checksum SHA-256;
- aggiorna l'alias stabile `AureaOS.iso` soltanto dopo la validazione del tree binary e della ISO effettiva.

Per riprendere una build interrotta, riesegui il wrapper dallo stesso checkout pulito e dallo stesso commit:

```bash
RESUME=1 bash tools/iso/build-live-iso-podman.sh
```

Il percorso predefinito del workspace contiene l'ID del commit sorgente: un nuovo commit riceve quindi un workspace nuovo, evitando di riutilizzare per errore un chroot completato che contiene un vecchio binario `aurea-desktop`. Se `BUILD_PARENT` è stato personalizzato, va passato esattamente lo stesso valore anche durante la ripresa. Non si avvia una seconda build sullo stesso workspace e non si cancella una build fallita prima di averne esaminato log e causa.

Lo script interno prepara il sorgente tramite `git archive HEAD`. Ora rifiuta working tree modificati e registra il commit completo in `.aureaos-source-commit` dentro ogni workspace. La ripresa è consentita soltanto se il marcatore coincide con il commit corrente; i workspace legacy senza marcatore vengono rifiutati. Compila quindi il commit locale attualmente selezionato, non esegue automaticamente un fetch di GitHub. La sincronizzazione Fedora e i controlli della fase precedente garantiscono che quel commit locale corrisponda allo stato `main` che intendiamo verificare. Durante la build lo script non deve cambiare branch né eseguire pull silenziosi.

### Fase E — Validazione automatica dell'immagine

Nel repository sono presenti un preflight del chroot, un generatore UKI/ISO e un validatore specifico dell'immagine finale:

- `tools/iso/tests/validate-chroot-boot.sh` — controlla nel chroot i pacchetti di boot richiesti e l'integrazione initramfs di live-boot, rigenera l'initramfs e verifica la presenza di `scripts/live`.
- `tools/iso/scripts/build-systemd-boot-live-iso.sh` — genera una UKI dalla coppia kernel/initramfs Debian esatta, crea un'immagine EFI FAT con systemd-boot e le voci UKI, rimuove il payload GRUB intermedio e rimasterizza la ISO UEFI-only.
- `tools/iso/tests/validate-live-image.sh` — controlla immagine EFI, percorso fallback systemd-boot, voce di avvio, sezioni UKI e parametro `boot=live`, oltre al vero rootfs SquashFS: binario desktop, launcher, unit systemd, definizione PAM, link di abilitazione e 26 asset RGBA8 con dimensioni esatte; verifica anche il catalogo El Torito EFI.

Il coordinatore installa esplicitamente il validatore boot del chroot in `/usr/local/libexec/aureaos/` prima dell'esecuzione di live-build. L'hook lo richiama dopo l'installazione e la pulizia dei pacchetti. Il coordinatore genera poi UKI/immagine EFI e valida sia il tree binary sia la ISO copiata.

I livelli di validazione devono coprire almeno:

1. Stato dei pacchetti nel chroot: `live-boot`, `live-boot-initramfs-tools`, `live-config`, `initramfs-tools` e il pacchetto kernel previsto.
2. Presenza ed esecuzione dell'hook initramfs di live-boot e dello script di avvio anticipato.
3. Generazione corretta dell'initramfs per il kernel installato.
4. Presenza di `scripts/live` in ogni initramfs selezionato.
5. Immagine filesystem Live non vuota in `/live/filesystem.squashfs`, con binario desktop e integrazione della sessione realmente presenti al suo interno.
6. Immagine EFI FAT con `EFI/BOOT/BOOTX64.EFI`, systemd-boot e configurazione loader.
7. Una o più UKI con kernel, initramfs, metadati os-release AureaOS e parametro `boot=live` incorporato.
8. Voci systemd-boot corrispondenti e voce predefinita valida.
9. Assenza di GRUB e di kernel/initramfs separati nell'immagine finale.
10. Leggibilità della ISO e catalogo El Torito EFI che punta all'immagine systemd-boot.
11. Checksum SHA-256 e report di validazione leggibile anche da strumenti automatici.

Un controllo non implementato o non eseguito deve risultare in sospeso, non essere considerato superato per supposizione.

### Fase F — Verifica su hardware reale

L'ispezione automatica non sostituisce un avvio reale. Copiamo la ISO candidata nella partizione dati Ventoy già esistente e la selezioniamo dal menu Ventoy. Questa procedura ordinaria non richiede di formattare o ricreare la chiavetta.

Registriamo separatamente:

- Ventoy ha caricato l'immagine?
- systemd-boot ha mostrato la voce AureaOS Live?
- Il kernel è stato caricato?
- L'initramfs è terminato correttamente e ha trovato il filesystem Live?
- systemd ha raggiunto il target previsto?
- Il servizio di sessione AureaOS è partito?
- Il compositor ha ottenuto il dispositivo DRM e il seat?
- È stato rilevato e utilizzato un output video fisico?
- La shell è apparsa ed è rimasta responsiva?
- Tastiera, mouse, rotellina, touchpad e altre periferiche disponibili hanno funzionato?
- Si sono verificati errori, blocchi, kernel panic o firmware mancanti?

Quando noti, annotiamo macchina, firmware/modalità di boot, GPU e configurazione display. Non attribuiamo un errore initramfs iniziale a Vulkan, Smithay o GPU se i log dimostrano che l'esecuzione non è arrivata a quei livelli.

### Fase G — Archiviazione e registrazione

Ogni candidata deve essere riconducibile al sorgente e ai controlli che ha superato. L'insieme di artefatti previsto è:

- Immagine ISO.
- Checksum SHA-256.
- Manifest di build (ad esempio `manifest.json`).
- Report di validazione automatica.
- Log completo della build e diagnostica degli errori.
- Scheda dei test hardware.
- ID completo del commit Git e branch.

Identificativo consigliato:

`AureaOS-AAAAMMGG-<commit-breve>.iso`

Possiamo mantenere una copia di comodo con nome stabile `AureaOS.iso`, ma soltanto dopo il superamento dei controlli automatici obbligatori. La candidata con nome univoco e i suoi report vanno conservati anche se non viene promossa.

Il manifest deve contenere almeno:

- nome progetto e immagine;
- ID build e data/ora con fuso orario o UTC;
- remote Git, branch e ID completo del commit;
- indicazione se il working tree era pulito;
- release Debian e architettura;
- metodo di build e identità dell'immagine container/builder;
- versione kernel;
- dimensione ISO e SHA-256;
- ogni controllo di validazione con il relativo esito;
- stato dei test hardware, descrizione macchina e note;
- errori noti e deviazioni dalla procedura.

**Significato di riproducibilità:** ogni immagine deve essere riconducibile in modo ripetibile a uno specifico commit e ai relativi input di build. Ottenere byte per byte la stessa ISO è un requisito più forte: non va promesso finché timestamp, snapshot dei repository pacchetti, versioni toolchain/container e tutti gli altri input variabili non saranno bloccati e verificati.

## 4. Implementazione attuale e obiettivo

Il repository dispone già di una base funzionante, ma ISO Factory non è ancora un sistema completo di gestione delle release.

### Già presente nel repository al 26 novembre 2026

- Configurazione live-build Debian Trixie amd64.
- Wrapper Podman con container Debian Trixie.
- Workspace live-build persistente e serializzazione dell'accesso tramite `flock`.
- Procedura di ripresa di una build interrotta.
- Hook chroot che compila e installa il binario desktop AureaOS e l'integrazione di sessione.
- Lista pacchetti di boot che include `live-boot-initramfs-tools`.
- Script di validazione del boot nel chroot.
- Script di validazione del payload binary e della ISO.
- Assemblaggio dell'immagine EFI systemd-boot + UKI e rimasterizzazione ISO UEFI-only.
- Ispezione UKI per parametri Live incorporati e validazione dell'immagine finale.
- Conservazione del workspace dopo il successo o il fallimento nello script interno attuale.
- File SHA-256 sia per la candidata con nome univoco sia per l'alias stabile.
- Manifest JSON di provenienza per ogni candidata validata, con commit sorgente esatto, versione kernel, dimensione/hash dell'immagine e stato della verifica hardware.
- Promozione atomica dell'alias stabile `AureaOS.iso` soltanto dopo la validazione della ISO temporanea esatta; le candidate storiche con nomi univoci vengono conservate.

### Lavoro ancora necessario prima di definire ISO Factory pronta per le release

- Ispezionare su Fedora l'initramfs della ISO che non ha superato il primo avvio, prima di eseguire una nuova build.
- Determinare perché il boot reale ha segnalato che `/scripts/live` non era accessibile. La causa esatta non è ancora confermata.
- Verificare le ipotesi del validatore rispetto all'immagine UEFI effettivamente generata e al percorso El Torito EFI.
- Conservare log completi con timestamp e un riepilogo finale sia per le build riuscite sia per quelle fallite.
- Aggiungere controlli di pulizia espliciti e sicuri; non cancellare automaticamente workspace o artefatti storici utili per diagnosi e rollback.
- Valutare caching/versionamento del container builder per evitare reinstallazioni superflue, senza compromettere aggiornamenti e tracciabilità.
- Verificare il boot UEFI-only con systemd-boot/UKI e l'effettiva sessione Live su hardware fisico.

Questi punti sono requisiti e attività future, non funzionalità che dichiariamo già completate.

## 5. Problema di boot attualmente aperto

La prima ISO di test basata su GRUB era leggibile e aveva un checksum SHA-256 valido. Al primo test reale tramite Ventoy, GRUB ha caricato il kernel Debian, ma init è fallito prima dell'avvio del desktop:

```text
init: line 249: can't open '/scripts/live': No such file or directory
Kernel panic - not syncing: Attempted to kill init: exitcode=0x00000200
```

Il fatto accertato è che `/scripts/live` non era disponibile quando init ha provato a eseguirlo. Il kernel panic è seguito a quel fallimento. La causa non è ancora stata isolata. La nuova factory mantiene il preflight nel chroot, rigenera l'initramfs dopo l'installazione dei pacchetti, verifica `scripts/live` e incorpora quello stesso initramfs nella UKI. È una protezione automatica più forte, non la prova che la causa precedente sia stata risolta: servirà avviare la nuova immagine.

Questo errore non dimostra un problema GPU, Vulkan o Smithay: il sistema non ha raggiunto l'avvio del compositor.

## 6. Checklist per venerdì/sabato

- [ ] Verificare che il repository sia su `main`.
- [ ] Eseguire pull fast-forward da `origin/main`.
- [ ] Verificare il working tree pulito e registrare il commit completo.
- [ ] Eseguire `cargo fmt --all -- --check`.
- [ ] Eseguire `cargo check --workspace`.
- [ ] Eseguire `cargo test --workspace`.
- [ ] Eseguire i controlli specifici delle funzionalità interessate.
- [ ] Registrare risultati dei comandi ed eventuali eccezioni.
- [ ] Finché il problema di boot è aperto, ispezionare prima l'initramfs esistente.
- [ ] Eseguire l'entry point ISO Podman mantenuto solo dopo la diagnosi/decisione.
- [ ] Verificare i report di validazione chroot e immagine.
- [ ] Verificare checksum e struttura UEFI-only con catalogo El Torito, systemd-boot e UKI.
- [ ] Avviare tramite la configurazione Ventoy esistente su hardware reale.
- [ ] Registrare risultati di boot, desktop e periferiche di input.
- [ ] Conservare ISO, manifest, log e report.
- [ ] Aggiornare entrambi i diari di sviluppo e questo documento quando cambiano fatti o decisioni.

## 7. File correlati

- `tools/iso/build-live-iso-podman.sh` — entry point Fedora/Podman.
- `tools/iso/build-live-iso.sh` — coordinatore Debian live-build.
- `tools/iso/package-lists/aureaos.list.chroot` — manifest dei pacchetti target.
- `tools/iso/hooks/010-build-aureaos.hook.chroot` — build desktop e integrazione chroot.
- `tools/iso/scripts/build-systemd-boot-live-iso.sh` — UKI, immagine EFI e assemblaggio ISO finale.
- `tools/iso/os-release` — identità AureaOS incorporata nella UKI.
- `tools/iso/tests/validate-chroot-boot.sh` — preflight boot nel chroot.
- `tools/iso/tests/validate-live-image.sh` — validazione immagine e ISO.
- [Diario di bordo](DIARIO-DI-BORDO.md).
- [Promemoria interno per la continuità dell'assistente](../../tools/iso/INTERNAL-ASSISTANT-HANDOFF.md) — note operative interne, non specifica pubblica del prodotto.
