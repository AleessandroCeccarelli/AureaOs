<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Roadmap di AureaOS

Questa roadmap registra lo stato attuale dell’implementazione e le priorità di sviluppo previste. È un piano di lavoro, non un calendario di consegna né una promessa di date di rilascio.

AureaOS Roma 1.0 è destinato a Debian GNU/Linux 13 (Trixie), architettura `amd64` (x86-64).

Debian costituisce la fondazione di livello inferiore. Ambiente desktop, esperienza di sistema, applicazioni fondamentali e identità del prodotto sono sviluppati e mantenuti dal progetto AureaOS.

## Direzione del prodotto

La prima versione punta a offrire un’esperienza desktop AureaOS completa e coerente, senza tentare di sviluppare da zero ogni applicazione.

La direzione di prodotto per Roma 1.0 comprende:

- Desktop grafico AureaOS e sessione Wayland, sviluppati e mantenuti dal progetto.
- AureaOS Human Interfaces e un sistema visivo coerente.
- Aurea Wallet come applicazione di sistema sviluppata dal progetto e pilastro del prodotto.
- Fides, gestore file AureaOS.
- Quaero, esperienza di ricerca globale.
- Aurea Store, negozio ufficiale delle applicazioni.
- Aurea Update per i componenti di sistema AureaOS.
- Aurea Time per backup e ripristino del sistema.
- Strumenti di sistema essenziali e procedure grafiche di configurazione.

Categorie comuni come browser web, client di posta e suite per ufficio saranno inizialmente coperte da applicazioni di terze parti consolidate.

Aurea Store si integrerà con Flathub e Flatpak per individuare, installare, eseguire, rimuovere e aggiornare applicazioni di terze parti compatibili nei rispettivi ambienti isolati.

Sviluppare alternative di prima parte per queste categorie potrà essere valutato nelle versioni future. Non è un requisito per Roma 1.0.

## Stato attuale

**Ultima esecuzione GitHub Actions completamente verde verificata il 18 novembre 2026:** commit `12cddaa8fe35dff6d8b5e1eece5c5f70ce267424`, esecuzione [44290834407](https://github.com/AleessandroCeccarelli/AureaOs/actions/runs/44290834407). Formattazione Rust, compilazione e test del workspace e job di audit delle dipendenze sono terminati con successo.

Questa esecuzione verifica i controlli configurati nella CI. Non dimostra il funzionamento dell’uscita DRM/KMS su hardware fisico, un desktop grafico completo o un’immagine di installazione avviabile. L’audit di sicurezza del progetto documenta inoltre rischi e attività ancora aperte, compreso l’avviso relativo alla dipendenza transitiva `cgmath 0.18.0`: il successo del job non deve essere interpretato come un rapporto dipendenze privo di avvisi.

### Implementato nel repository

- Documentazione di visione, contributi, direzione tecnologica e architettura iniziale.
- Manifest dei pacchetti Debian e script di bootstrap per un root filesystem Debian minimo, solo terminale.
- Risorse visive AureaOS: token di design, temi, icone, illustrazioni e sfondi.
- Workspace Rust con quattro crate in `components/`: `aurea-desktop`, `aurea-shell`, `aurea-desktop-api` e `aurea-compositor`.
- Prima radice di composizione `aurea-desktop`, che collega i componenti; non è una sessione desktop completa né un’interfaccia grafica finita.
- Contratti desktop condivisi in `aurea-desktop-api`, incluso un valore di regolazione del colore indipendente dalla tecnologia per una futura integrazione nella politica delle uscite.
- Coordinamento del ciclo di vita della sessione e gestione dello spegnimento della shell tramite bus di eventi.
- Server Wayland basato su Smithay con integrazione iniziale della shell XDG.
- Supporto iniziale a seat Wayland e tastiera XKB, con instradamento degli eventi libinput quando è attiva la funzionalità DRM.
- Scorciatoie globali Super + Ctrl + Freccia sinistra/destra per la navigazione tra workspace.
- Tracciamento iniziale delle superfici toplevel e strutture per la gestione delle finestre.
- Modello iniziale con quattro workspace, assegnazione delle finestre e cambio workspace.
- Filtraggio della scena di rendering per disegnare solo le finestre mappate nel workspace attivo.
- Conversione degli alberi di superfici XDG mappate in elementi di rendering Smithay, con posizioni logiche e scala delle uscite.
- Rilevamento e gestione del ciclo di vita dei dispositivi DRM tramite Udev e libseat.
- Integrazione degli eventi dei dispositivi DRM nel ciclo eventi del compositor.
- Gestione della pausa e riattivazione della sessione per i dispositivi DRM.
- Modelli per inventario GPU e preferenze di rendering.
- Rilevamento dei connettori DRM e comunicazione delle modalità video supportate.
- Generazione di identità stabili per le uscite.
- Registrazione, rimozione, configurazione e disposizione logica delle uscite.
- Riconciliazione delle uscite quando cambia la configurazione dei display rilevati.
- Pubblicazione e rimozione delle uscite registrate come global Wayland.
- Modello iniziale di preferenza e pianificazione locale per Night Light nella shell, con validazione e test unitari.
- Controller condiviso e lock-free per il colore del display, collegato dal runtime della shell tramite API desktop allo stato del server Wayland.
- Percorso iniziale Vulkan per Night Light: pipeline colore memorizzata in cache per dispositivo e formato, immagine intermedia composta su richiesta, composizione dell’intera scena (sfondo incluso), passaggio colore finale e ripiego sul direct scanout quando il percorso opzionale di formato o immagine non è disponibile. La sorgente supera i controlli CI di compilazione workspace/DRM, test automatizzati, compilazione shader e smoke test offscreen lavapipe; la validazione fisica DRM/Vulkan è ancora in sospeso. Servizio di attivazione degli orari, persistenza, comunicazione delle capacità e controlli visibili sono ancora da completare.

Questi componenti costituiscono una base iniziale per desktop e dispositivi grafici. Non sono ancora un desktop grafico utilizzabile né una pipeline grafica completa.

Il profilo di bootstrap Debian richiede ancora la validazione della risoluzione dei pacchetti e non produce un’immagine di installazione avviabile.

### In sviluppo e validazione

- Mantenere il compositor privo di warning oltre che compilabile; continuare a verificare shell e compositor con compilazione e test automatizzati.
- Proseguire i test indipendenti dall’hardware sulla gestione delle uscite e sul ciclo di vita DRM.
- Validare acquisizione dei dispositivi DRM, rilevamento delle uscite, hotplug e gestione della sessione su hardware fisico rappresentativo.
- Definire e validare identità del sistema, servizi predefiniti, layout di archiviazione, configurazione di avvio e politiche di sicurezza.
- Validare profilo pacchetti Debian 13 e processo di bootstrap rispetto ai repository previsti.

## Sviluppo pianificato

L’architettura dei servizi desktop è progettata secondo il principio **local-first**: servizi nativi e API per applicazioni con permessi controllati dovranno funzionare senza account cloud.

Controllo ortografico e correzione automatica globali, dizionari personali e sostituzioni di testo sono obiettivi progettuali, non codice attualmente disponibile. La sincronizzazione futura opzionale di contatti, posta, calendari, note e impostazioni non dovrà essere un prerequisito per utilizzare il desktop.

L’obiettivo Wayland-only/Vulkan-only richiede inoltre un programma misurato di verifica della compatibilità delle applicazioni Flatpak. Non viene dichiarata compatibilità universale con Flathub.

### 1. Fondamenta desktop e grafiche

- Completare, compilare e validare la composizione Vulkan SHM esistente e la presentazione fisica dei fotogrammi DRM/KMS.
- Completare impostazione delle modalità DRM/KMS e configurazione fisica dei display.
- Completare gestione di puntatore, touch e gesti, inclusi i gesti del touchpad per i workspace.
- Sviluppare gestione completa delle finestre e interazioni desktop.
- Integrare rendering, presentazione e ciclo di vita delle uscite del compositor.
- Validare la pipeline grafica su hardware rappresentativo.

Vulkan è l’API esclusiva scelta per il rendering AureaOS. Workspace e funzionalità DRM/Vulkan compilano in CI e un test offscreen lavapipe passa. Composizione SHM end-to-end e uscita fisica DRM/KMS restano da verificare.

### 2. Display Intelligence

- Sviluppare Aurea Display Intelligence per rilevamento delle capacità e politica dei display.
- Implementare strategie a refresh fisso e supporto VRR dove l’intera catena hardware e driver lo consente.
- Integrare la politica display con le informazioni di presentazione del compositor e il backend DRM/KMS.
- Fornire modalità Automatica — AureaOS e configurazione manuale in Impostazioni → Display e video.
- Implementare ripieghi sicuri per capacità non supportate o non disponibili.
- Implementare Night Light come regolazione della temperatura colore delle uscite a livello di sistema, con intensità manuale, orari alba/tramonto e pianificazione locale personalizzata.
- Validare su hardware DRM/Vulkan fisico il primo stadio Vulkan di Night Light; nel codice è collegato dopo la composizione completa della scena, con immagine intermedia opzionale e ripiego sul direct scanout quando il formato non è disponibile.
- Completare attivazione degli orari, persistenza, comunicazione delle capacità e utilizzo della politica condivisa shell-compositor tramite API desktop e stato del server Wayland.
- Considerare gestione di spazio colore e funzioni di trasferimento, formati di uscita, HDR e preferenze di accessibilità prima di attivare il filtro come impostazione predefinita.

I controlli Night Light appartengono a Impostazioni → Aspetto e al pannello Impostazioni rapide sul lato destro. La pianificazione alba/tramonto richiede un servizio di posizione e fuso orario con gestione dei permessi; deve restare disponibile una pianificazione locale personalizzata senza geolocalizzazione.

La regolazione software è un’approssimazione della temperatura colore del display, non una dichiarazione di filtraggio spettrale hardware della luce blu.

L’architettura e le responsabilità sono descritte in [Display Intelligence](../architecture/DISPLAY-INTELLIGENCE.md).

### 3. Esperienza desktop AureaOS

- Sviluppare la shell grafica desktop.
- Sviluppare la libreria condivisa AureaOS Human Interfaces e componenti UI riutilizzabili.
- Implementare esperienza di gestione finestre e interazioni desktop.
- Sviluppare impostazioni di sistema e procedure grafiche essenziali.
- Integrare notifiche, navigazione da tastiera, accessibilità, temi e fondamenta per la localizzazione.
- Realizzare l’esperienza grafica di installazione.

Il desktop dovrà avere un’identità AureaOS coerente e non basarsi su un ambiente desktop di terze parti.

### 4. Applicazioni fondamentali sviluppate dal progetto

- **Aurea Wallet:** definire ambito funzionale, modello delle credenziali, architettura di sicurezza e requisiti di archiviazione prima dell’implementazione.
- **Fides:** sviluppare gestore file grafico e flussi comuni di gestione dei file.
- **Quaero:** ricerca globale di applicazioni, file, cartelle, impostazioni e azioni di sistema supportate.
- **Aurea Store:** sviluppare interfaccia del negozio ufficiale e integrazione Flathub/Flatpak.
- **Aurea Update:** implementare esperienza di aggiornamento dei componenti di sistema specifici AureaOS.
- **Aurea Time:** sviluppare backup e ripristino del sistema, incluso il recupero completo.
- **Strumenti di sistema:** sviluppare interfacce previste per gestione stampanti, scanner e monitoraggio risorse.

Queste applicazioni dovranno seguire il contratto condiviso AureaOS Human Interfaces e rispettare la separazione delle responsabilità di sistema.

### 5. Integrazione delle applicazioni di terze parti

- Integrare Flathub come sorgente applicativa di Aurea Store.
- Integrare flussi Flatpak di installazione, esecuzione, rimozione e aggiornamento.
- Presentare chiaramente informazioni, provenienza, permessi e limiti dell’isolamento delle applicazioni.
- Fornire browser, client di posta, suite per ufficio e altre applicazioni desktop utili di terze parti.
- Mantenere separati gli aggiornamenti delle applicazioni di terze parti da quelli Debian e AureaOS.

Le applicazioni di terze parti non sono prodotti sviluppati direttamente da AureaOS. Il loro utilizzo permette al progetto di concentrarsi sul desktop, sull’integrazione di sistema e sulle applicazioni fondamentali.

### 6. Fondazione di sistema e distribuzione

- Completare e validare la fondazione minima Debian 13 `amd64`.
- Definire identità del sistema, servizi predefiniti, layout di archiviazione, configurazione di avvio e confini di sicurezza.
- Validare il layout Btrfs previsto per l’installazione.
- Definire canali di aggiornamento separati per pacchetti base Debian e componenti AureaOS.
- Stabilire firma dei pacchetti, verifica d’integrità e procedure di aggiornamento sicure.
- Produrre e validare un’immagine di sviluppo avviabile.
- Documentare hardware supportato e requisiti di installazione.

### 7. Preparazione al rilascio finale

- Validare la sessione desktop completa su hardware fisico rappresentativo.
- Testare installazione, primo avvio, sessione utente, aggiornamenti e ripristino.
- Validare Aurea Store e i flussi delle applicazioni Flatpak di terze parti.
- Rivedere sicurezza, permessi, accessibilità e localizzazione.
- Completare documentazione e criteri di rilascio.
- Produrre e validare l’immagine di installazione AureaOS Roma 1.0.

## Priorità di sviluppo

L’ordine seguente descrive le priorità tecniche attuali. Alcuni filoni potranno procedere in parallelo quando le dipendenze lo consentiranno.

1. Continuare la verifica di shell e compositor, inclusi i test automatizzati.
2. Implementare un backend Vulkan concreto e stabilire una presentazione fisica affidabile dei fotogrammi.
3. Completare impostazione delle modalità DRM/KMS e configurazione dei display fisici.
4. Implementare gestione input e proseguire con il ciclo di vita di finestre e superfici del compositor.
5. Validare rilevamento delle uscite, hotplug, cambi di sessione e gestione display su hardware fisico.
6. Sviluppare Aurea Display Intelligence su una pipeline display affidabile, incluso lo stadio di uscita Night Light e i ripieghi sicuri in base alle capacità.
7. Completare e validare la fondazione minima Debian 13 `amd64`.
8. Definire confini di immagine, avvio, installazione, sicurezza, archiviazione e aggiornamenti.
9. Costruire e testare un’immagine di sviluppo avviabile.
10. Sviluppare fondamenta GUI condivise ed esperienza desktop AureaOS, incluse Impostazioni e Impostazioni rapide.
11. Sviluppare e integrare le applicazioni di prima parte previste per Roma 1.0.
12. Integrare Flathub e Flatpak attraverso Aurea Store.
13. Validare il prodotto completo e definire i criteri di rilascio Roma 1.0.

## Stato di applicazioni e componenti

Il [Catalogo delle applicazioni e dei componenti](../product/APPLICATION-CATALOG.md) è l’inventario di riferimento per applicazioni sviluppate dal progetto, strumenti di sistema, componenti desktop, risorse di progettazione condivise, sottosistemi e categorie di applicazioni di terze parti selezionate.

Il catalogo distingue prototipi implementati, componenti pianificati e concetti esplorativi. La presenza di un elemento nel catalogo non implica che sia completo o disponibile agli utenti.

La direzione di prodotto Roma 1.0 comprende Aurea Desktop, AureaOS Human Interfaces, Aurea Wallet, Fides, Quaero, Aurea Store, Aurea Update, Aurea Time e gli strumenti di sistema previsti per la prima versione.

Ambito e implementazione restano soggetti a sviluppo e validazione. L’inclusione nella direzione Roma 1.0 non significa che i componenti siano già implementati o completi.

Aurea Display Intelligence resta un sottosistema pianificato. Nel compositor esistono fondamenta iniziali per dispositivi DRM e ciclo di vita delle uscite. Il percorso iniziale di elaborazione Vulkan Night Light è collegato nel codice, ma non è ancora validato su hardware. Impostazione delle modalità video, controllo VRR, politica di refresh adattivo, servizio e persistenza degli orari, comunicazione delle capacità e interfaccia delle impostazioni restano incompleti.

Sonus, Visio ed Explora restano candidati. Explora, se sviluppato, potrebbe essere un futuro browser di prima parte AureaOS, ma non è un requisito per Roma 1.0. Si veda [Applicazioni di prima parte pianificate](../product/PLANNED-APPLICATIONS.md).

Passkey a livello di sistema, integrazione Aurea AI/ChatGPT e Aurea Media API restano concetti esplorativi o candidati. Non sono impegni per Roma 1.0.

Browser, client di posta, suite per ufficio e altre applicazioni selezionate di terze parti dovrebbero coprire le esigenze comuni in Roma 1.0. Restano separate dal catalogo delle applicazioni sviluppate da AureaOS.

## Domini di aggiornamento

AureaOS distingue tre domini indipendenti:

1. **Base Debian:** pacchetti e aggiornamenti di sicurezza forniti dalla fondazione Debian.
2. **Sistema AureaOS:** componenti desktop, applicazioni, servizi e risorse specifiche mantenuti da AureaOS.
3. **Applicazioni Flatpak:** applicazioni di terze parti installate e gestite tramite Aurea Store, Flatpak e sorgenti supportate come Flathub.

Ogni dominio richiede responsabilità chiare, verifica d’integrità e un processo di aggiornamento adeguato alle proprie funzioni.

Il canale di aggiornamento del sistema AureaOS e l’integrazione completa del negozio non sono ancora implementati.

## Principi guida

1. Affidabilità prima del numero di funzionalità.
2. Esperienza desktop AureaOS coerente e con identità propria.
3. Flussi grafici per le attività quotidiane.
4. Aurea Wallet e applicazioni fondamentali come priorità della prima versione.
5. Confini chiari tra Debian, componenti AureaOS e applicazioni di terze parti.
6. Installazione sicura delle applicazioni e permessi di isolamento trasparenti.
7. Comportamento consapevole dell’hardware, ripieghi sicuri e rispetto delle preferenze dell’utente.
8. Architettura modulare e responsabilità chiare tra componenti di sistema.
9. Design coerente e accessibile nelle applicazioni sviluppate dal progetto.
10. Comunicazione onesta di stato, limitazioni e licenze.


## Aggiornamento operativo — 22 novembre 2026

La priorità immediata concordata è la costruzione del desktop AureaOS. Il lavoro sulla generazione ISO è sospeso fino alla sessione di verifica su Fedora prevista per venerdì 27 novembre 2026. Non avviare nuove build ISO né modificare gli script ISO durante le sessioni desktop, salvo richiesta esplicita.

La prima immagine Live Debian Trixie è stata generata e verificata come file ISO con strutture di avvio BIOS/UEFI, ma il primo avvio reale tramite Ventoy sul portatile Lenovo si è fermato nell'initramfs:
`can't open '/scripts/live': No such file or directory`, seguito da kernel panic di init. Pertanto l'immagine non è attualmente considerata avviabile come sistema Live.

Venerdì la verifica ripartirà dall'ispezione read-only dell'initramfs della ISO già esistente. Solo dopo aver individuato la causa si interverrà sulla pipeline. La correzione dovrà anche preservare workspace e cache dopo una build riuscita e introdurre controlli sul contenuto dell'initramfs, non soltanto sull'esistenza del file ISO.

Il dettaglio cronologico e i riferimenti Git sono nel [Diario di bordo](docs/it/DIARIO-DI-BORDO.md).
