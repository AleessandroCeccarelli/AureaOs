<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Diario di bordo dello sviluppo AureaOS

Questo documento è il registro cronologico dell’attività tecnica di AureaOS. Affianca, senza sostituirli, i documenti di architettura, gli audit di sicurezza, la roadmap e la cronologia Git.

## 18 novembre 2026 — Stato del repository e fondamenta della scala dei display

### Stato del repository

- Il ramo di sviluppo attivo è `main`.
- Il repository non presenta altri branch né Pull Request aperte al momento della verifica.
- Il workspace Rust contiene quattro crate: `aurea-desktop`, `aurea-shell`, `aurea-desktop-api` e `aurea-compositor`.
- Il repository dispone già di documentazione dedicata ad architettura, prodotto, sicurezza, test e interfacce umane. Questa sezione aggiunge una cronologia operativa, senza duplicare quei documenti.
- Durante questa revisione non sono stati rimossi file sorgente o branch: l’ispezione non ha dimostrato che elementi tracciati fossero obsoleti o eliminabili in sicurezza.

### Lavoro tecnico registrato

I commit recenti hanno stabilito le prime fondamenta per coordinate dei display e scala frazionaria:

- Rappresentazione esatta e razionale del fattore di scala e conversioni coordinate con controlli di sicurezza.
- Separazione tra coordinate logiche globali e pixel fisici locali all’uscita.
- Posizionamento logico delle uscite in base alle larghezze corrette per la scala.
- Trasporto del valore razionale della scala nei render target e nella geometria del puntatore.
- Registrazione del global Wayland fractional-scale manager.
- Impostazione conservativa della scala preferita iniziale a 1.0 per i nuovi oggetti fractional-scale.

Commit di riferimento:

- `9796c023` — documentazione delle fondamenta delle coordinate display e dello stato della migrazione.
- `26f47d60` — definizione del contratto di interoperabilità desktop Linux.
- `6bebd8f1` e `d77db70d` — posizionamento logico delle uscite e test della disposizione con scala.
- `8a0df018`, `9cce2778`, `2ae83868`, `d9ff4e36` e `91f2b214` — protocollo fractional-scale e stato del manager.
- `12cddaa8` — correzione dell’accesso allo stato fractional-scale del compositor e della formattazione.

### Verifica

L’esecuzione GitHub Actions [44290834407](https://github.com/AleessandroCeccarelli/AureaOs/actions/runs/44290834407), relativa al commit `12cddaa8fe35dff6d8b5e1eece5c5f70ce267424`, è terminata con successo il 18 novembre 2026.

Tutti e tre i job sono passati:

- Formattazione Rust.
- Compilazione e test del workspace.
- Audit di sicurezza delle dipendenze Rust.

Questa è una prova relativa ai controlli configurati nella CI. Non dimostra un avvio riuscito su hardware fisico, uno scanout DRM/KMS completo, un desktop grafico finito o un’immagine ISO di installazione avviabile.

### Limitazioni note e lavoro aperto

- La scala frazionaria non è integrata end-to-end. Restano da collegare aggiornamenti della scala preferita in base all’uscita, gestione di buffer e viewport delle superfici, hotspot del cursore, clipping, hit-testing e integrazione completa del renderer.
- Uscita DRM/KMS fisica e scanout devono ancora essere verificati su hardware supportato.
- Il flusso di bootstrap Debian e pubblicazione del rootfs richiede ancora una validazione completa end-to-end sulla macchina Fedora di build supportata.
- L’audit di sicurezza del 13 novembre 2026 registra attività di hardening ancora aperte. I relativi rilievi non possono essere considerati chiusi solo perché il job CI di audit delle dipendenze Rust è passato.
- L’avviso RustSec relativo alla dipendenza transitiva `cgmath 0.18.0` resta documentato nell’audit di sicurezza e richiede una soluzione consapevole dello stato upstream. Un job di audit terminato con successo non equivale a un rapporto dipendenze privo di avvisi.

### Prossime attività tecniche

1. Proseguire l’integrazione della scala frazionaria esaminando associazione tra superfici e uscite e API Smithay 0.7 pertinenti prima di modificare il comportamento.
2. Definire e testare una politica coerente di enter/leave delle uscite e scala preferita per toplevel, popup e superfici figlie.
3. Completare l’integrazione di buffer, viewport, cursore, clipping e hit-testing in moduli piccoli e verificabili separatamente.
4. Mantenere esplicita nella documentazione la distinzione tra verifica CI e verifica su hardware fisico.
5. Riprendere i rilievi aperti dell’audit di sicurezza e pianificare la validazione del rootfs/bootstrap su Fedora quando la macchina di build sarà disponibile.

---

## Modello per le nuove registrazioni

Per ogni nuova voce utilizzare questa struttura:

- **Data e ambito**
- **Riferimenti a repository e commit**
- **Modifiche effettivamente registrate**
- **Verifiche eseguite e risultati precisi**
- **Limitazioni note o regressioni**
- **Prossime attività**

Non registrare come completate attività che sono soltanto intenzioni.


## 22 novembre 2026 — Prima generazione ISO e primo avvio reale

### Contesto e decisione operativa

La giornata è stata dedicata alla prima generazione completa dell'immagine Live Debian Trixie e alla sua prova reale tramite Ventoy. L'utente ha confermato che il flusso abituale è copiare il file ISO nella partizione dati Ventoy e selezionarlo dal menu di avvio: non va proposta la preparazione di una nuova chiavetta.

La generazione della ISO ha richiesto numerosi interventi sul layout live-build, sul recupero del workspace e sulla gestione dei lock. L'esito positivo della build non è stato sufficiente a dimostrare che il sistema Live fosse avviabile.

### Interventi e riferimenti Git

Commit pubblicati su `main` nella sessione:
- `47fd5ed7` — sincronizzazione delle modifiche locali del repository.
- `6bfc72c7` — test d'integrazione per verificare che i gesti touchpad raggiungano la shell desktop.
- `ad524e94` — sistemazione della formattazione dei test d'integrazione dei gesti.
- `f38bd6ac` — serializzazione dell'accesso al workspace live-build persistente tramite lock.
- `63c4bed8` — recupero del lock live-build obsoleto durante la ripresa di una build interrotta.

File ISO principali coinvolti:
- `tools/iso/build-live-iso.sh`
- `tools/iso/build-live-iso-podman.sh`
- `tools/iso/hooks/010-build-aureaos.hook.chroot`
- `tools/iso/hooks/020-brand-boot-menu.hook.binary`
- `tools/iso/package-lists/aureaos.list.chroot`

Il workspace live-build era stato inizialmente creato con una struttura annidata non canonica (`config/config`). Lo script è stato aggiornato per migrare in modo prudente il vecchio workspace al layout Debian standard, preservando chroot, marker di fase e cache quando possibile. Il wrapper Podman ora serializza l'accesso al workspace con `flock`; lo script interno controlla e rimuove un lock residuo solo nel percorso di resume e solo se è un file regolare non symlink.

### Esito della generazione

Il log di build fornito dall'utente il 22 novembre riporta:
- recupero del problema `lb chroot_devpts`;
- kernel e initramfs generati;
- esecuzione riuscita dell'hook di personalizzazione GRUB;
- creazione dell'immagine tramite `xorriso`;
- messaggio finale `P: Build completed successfully`.

Artefatti indicati:
- `~/Documenti/AureaOS.iso`
- `~/Documenti/AureaOS.iso.sha256`
- SHA-256: `1d929c0f99b665f1307191d9502f5c094a25452e32dd28615309feb8350c0704`

Controlli eseguiti su Fedora:
- ISO di circa 350 MB.
- `sha256sum -c` ha restituito `AureaOS.iso: OK`.
- `file` ha riconosciuto ISO 9660 con settore di boot DOS/MBR e volume `AUREAOS_BLUE_TEST`.
- Dopo installazione di `xorriso`, il report El Torito ha rilevato GRUB BIOS (`/boot/grub/grub_eltorito`), immagine EFI (`/boot/grub/efi.img`), MBR protettivo e GPT.

Questi controlli provano la creazione e la presenza delle strutture di boot nell'immagine; non provano il corretto avvio del sistema Live.

### Primo avvio tramite Ventoy: problema bloccante

Sul portatile Lenovo, Ventoy ha caricato GRUB e il kernel Debian `6.12.117+deb13-amd64`. Il boot si è poi interrotto prima dell'avvio del desktop.

Messaggi fotografati:
```
init: line 249: can't open '/scripts/live': No such file or directory
Kernel panic - not syncing: Attempted to kill init: exitcode=0x00000200
```

Conclusione verificata: durante l'esecuzione di init manca il percorso `/scripts/live` nell'ambiente initramfs. Il kernel panic è conseguenza del fallimento del processo init. Non è stato dimostrato un problema GPU, Vulkan o Smithay: il sistema non ha ancora raggiunto quella fase. La causa precisa deve essere determinata ispezionando l'initramfs e il contenuto del chroot.

### Rilievi sullo script da correggere

L'ispezione dei file su `main` ha evidenziato un punto importante: `build-live-iso.sh` rimuove automaticamente l'intero `BUILD_ROOT` quando `lb build` termina con codice zero. Questo elimina anche workspace e cache dopo una build riuscita, vanificando il riutilizzo incrementale. La pulizia dovrà diventare esplicita e opzionale, non automatica.

Inoltre il controllo finale attuale verifica essenzialmente che esista una ISO non vuota e che live-build termini con successo. Non controlla che l'initramfs contenga davvero `/scripts/live`, gli hook di live-boot, il kernel previsto o parametri GRUB coerenti. Una build non dovrà più essere dichiarata valida solo in base all'esistenza del file ISO.

### Decisioni e priorità concordate

- Da lunedì 23 novembre il lavoro torna esclusivamente alla costruzione del desktop AureaOS su GitHub.
- Nessuna build ISO e nessun intervento sugli script ISO durante le sessioni desktop, salvo richiesta esplicita.
- Venerdì 27 novembre si riprende la verifica ISO su Fedora, prima ispezionando l'immagine già prodotta e l'initramfs, senza ricompilare alla cieca.
- La prova continua tramite Ventoy, copiando direttamente la ISO nella sua partizione dati.
- Le modifiche restano su `main`; non eseguire build/test Rust remoti. L'utente eseguirà `cargo fmt`, controlli e test su Fedora quando concordato.
- Mantenere modularità rigorosa: un file, una responsabilità; script di build suddivisi in moduli piccoli e coordinatore limitato.

### Prossime attività

**Lavoro desktop (da lunedì):** proseguire lo sviluppo del desktop e delle sue funzionalità, senza riaprire il filone ISO.

**Sessione ISO di venerdì:**
1. Montare la ISO esistente in sola lettura e individuare kernel/initramfs.
2. Ispezionare il contenuto initramfs per `scripts/live`, hook e moduli live-boot.
3. Se necessario, ispezionare chroot/workspace; non presumere che il workspace sia conservato, perché il cleanup attuale lo elimina dopo il successo.
4. Verificare pacchetti `live-boot`, `live-config`, hook initramfs-tools, parametri kernel `boot=live` e voci GRUB.
5. Solo dopo aver identificato la causa, correggere in modo mirato la pipeline.
6. Rendere persistente il workspace dopo il successo e introdurre pulizia esplicita.
7. Aggiungere validazione pre-ISO e post-ISO, inclusa verifica effettiva dell'initramfs, mantenendo log diagnostici completi.

Non registrare il desktop come completato né l'ISO come avviabile: il primo boot reale ha dimostrato il contrario.


## 23 novembre 2026 — Primo nucleo operativo del Desktop

Dopo aver definito l'architettura, è iniziata l'implementazione concreta del desktop. La priorità è costruire prima una superficie desktop reale; Dock, Launcher e lavoro sul puntatore restano temporaneamente esclusi.

### DesktopManager e superficie base

È stato introdotto in `components/aurea-compositor/src/desktop/` un primo sottosistema modulare:
- `manager.rs`: coordina tema e sorgente dello sfondo e produce uno snapshot immutabile per il frame;
- `theme.rs`: modalità Light/Dark con i token desktop AureaOS;
- `background.rs`: background derivato dal tema o colore uniforme personalizzato, con validazione;
- `surface.rs`: descrizione immutabile della superficie desktop;
- `snapshot.rs`: dati desktop congelati per il singolo frame;
- `mod.rs`: esportazioni del modulo.

Il compositor ora possiede il DesktopManager. La scena di rendering riceve lo snapshot e il backend Vulkan utilizza il colore selezionato in entrambe le principali situazioni: scena vuota e scena con superfici, incluse le varianti readback e rilascio delle immagini verso KMS. Il clear pass Vulkan rimane generico e non contiene logica di tema.

Il colore predefinito è il token Light `#edf4ff`; il token Dark è `#0d1320`. È inoltre predisposto un colore uniforme personalizzato, validato e sempre opaco.

### Stato delle verifiche

Sono stati aggiunti test unitari per tema, background e DesktopManager. Non sono stati eseguiti `cargo fmt`, `cargo check`, test o build: la verifica resta affidata a Fedora dell'utente. Il codice è quindi pubblicato ma non ancora dichiarato compilante o funzionante su hardware.

Wallpaper, decodifica immagini, upload texture Vulkan, scaling/crop e persistenza non sono ancora implementati. Il puntatore, Dock e Launcher restano rinviati.

Commit principali: `542f918`, `2826f2d`, `e0fb92e`.
