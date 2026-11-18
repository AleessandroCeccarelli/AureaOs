<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Principi tecnologici e architetturali di AureaOS

## 1. Piattaforma e architettura del prodotto

**Sviluppato dal progetto non significa codice chiuso.** AureaOS progetta e mantiene il proprio ambiente desktop, la shell, l’integrazione del compositor, i servizi di sistema e le applicazioni con un nome e una funzione propri. Sono componenti sviluppati direttamente per AureaOS, ma il codice originale del progetto è distribuito con licenza GPL-3.0-or-later, salvo diversa indicazione esplicita nel singolo file.

Nella comunicazione pubblica, quando si parla di codice o architettura, preferire le espressioni *sviluppato da AureaOS*, *prima parte* o *nativo AureaOS* a *proprietario*. La titolarità del prodotto e dell’identità visiva non equivale a una licenza software proprietaria. Si veda [Identità e terminologia del progetto](../PROJECT-IDENTITY-AND-TERMINOLOGY.md).

AureaOS Roma 1.0 è destinato a Debian GNU/Linux 13 (Trixie) su amd64 (x86-64).

AureaOS è una distribuzione Linux orientata all’interfaccia grafica e destinata a una sessione desktop esclusivamente Wayland. Non è previsto un livello di compatibilità X11 o XWayland: è un vincolo di compatibilità deliberato, non la dichiarazione che tutte le applicazioni Linux esistenti funzioneranno.

L’architettura è suddivisa in due livelli principali:

- **Fondazione Debian:** fornisce kernel Linux, driver, firmware, pacchetti di sistema fondamentali e infrastruttura Linux consolidata.
- **Livello prodotto AureaOS:** fornisce ambiente desktop, integrazione di sistema, esperienza utente e applicazioni sviluppate specificamente per AureaOS.

Debian è la fondazione tecnica, non l’identità dell’esperienza desktop.

AureaOS progetta e sviluppa il proprio livello superiore. Questo non significa che ogni applicazione disponibile agli utenti debba essere sviluppata dal progetto.

Per la prima versione, AureaOS unirà un desktop integrato e applicazioni di sistema essenziali sviluppati dal progetto ad applicazioni di terze parti selezionate e distribuite attraverso Aurea Store.

## 2. Criteri di selezione delle tecnologie

Le tecnologie devono essere scelte in base alle necessità di AureaOS e dei suoi utenti.

Software e firmware open source o proprietari possono essere presi in considerazione quando la scelta è giustificata tecnicamente e la distribuzione è legalmente consentita.

Ogni componente va valutato in base a:

- Compatibilità e affidabilità sulla piattaforma di destinazione.
- Sicurezza, manutenzione e supporto del progetto upstream.
- Prestazioni e consumo di risorse.
- Usabilità, accessibilità e integrazione con il sistema.
- Licenza, attribuzione, ridistribuzione, brevetti e altri termini applicabili.
- Sostenibilità nel lungo periodo e disponibilità di alternative.

AureaOS dovrebbe riutilizzare infrastrutture Linux mature invece di riscrivere tecnologie consolidate senza una ragione concreta.

Il lavoro di sviluppo deve concentrarsi sui componenti che definiscono l’esperienza AureaOS e sull’integrazione necessaria a rendere coerente il sistema.

Il lavoro originale del progetto è attualmente distribuito con licenza **GPL-3.0-or-later**, inclusi desktop, componenti di sistema, applicazioni sviluppate dal progetto, documentazione e sorgenti del sistema di progettazione, salvo indicazioni specifiche per materiali di terze parti.

I componenti di terze parti mantengono le proprie licenze. La GPL-3.0-or-later non si applica automaticamente a software di terzi solo perché viene utilizzato da AureaOS.

Qualsiasi futura modifica al modello di licenza o distribuzione è ancora da decidere e richiede una valutazione separata, compresi titolarità del copyright, diritti dei contributori e obblighi delle licenze.

## 3. Linguaggio di programmazione

Rust è il linguaggio principale per i componenti di sistema e le applicazioni sviluppati da AureaOS, dove tecnicamente appropriato.

Questo comprende ambiente desktop, servizi di sistema e applicazioni grafiche.

Un linguaggio diverso può essere utilizzato quando esiste una motivazione tecnica chiara. L’eccezione e le sue implicazioni di manutenzione devono essere documentate.

Questa regola non obbliga applicazioni di terze parti, driver, firmware, librerie o infrastrutture esterne a utilizzare Rust.

## 4. Architettura desktop e grafica

AureaOS punta a una sessione desktop **esclusivamente Wayland**. X11 e XWayland sono intenzionalmente fuori dall’architettura prevista. È una scelta con conseguenze sulla compatibilità, non la garanzia che ogni applicazione o toolkit Linux funzioni senza modifiche.

Il desktop è sviluppato come ambiente AureaOS, con responsabilità separate per shell, compositor, servizi di sistema e applicazioni.

### Fondamenta attuali del compositor

L’implementazione attuale di `aurea-compositor` utilizza:

- **Smithay** come framework per il compositor Wayland.
- **Wayland** per la comunicazione grafica del desktop.
- **Calloop** per l’integrazione del ciclo degli eventi.
- **libseat** per l’accesso alla sessione e ai dispositivi.
- **DRM e Udev** per il rilevamento dei dispositivi grafici e le fondamenta del ciclo di vita delle uscite video.

Il workspace Rust si trova in `components/` e contiene quattro crate con responsabilità distinte:

- `aurea-desktop`: punto di ingresso del desktop e radice di composizione.
- `aurea-shell`: ciclo di vita della sessione e coordinamento degli eventi.
- `aurea-desktop-api`: contratti condivisi tra i componenti desktop.
- `aurea-compositor`: compositor Wayland e responsabilità grafiche e di gestione delle uscite.

La radice di composizione collega i componenti, ma non deve assorbirne le responsabilità interne. La shell coordina la sessione e non deve possedere rendering del compositor o gestione dei dispositivi video.

Questi componenti costituiscono una base tecnica iniziale. Non sono ancora un desktop grafico completo.

Rendering completo, presentazione fisica dei fotogrammi, configurazione DRM/KMS completa, gestione completa degli input e gestione definitiva delle finestre sono ancora in sviluppo.

### Vulkan

Vulkan è l’API grafica scelta come **unica API di rendering** per AureaOS. OpenGL non fa parte del backend di rendering previsto per il progetto.

Non verranno introdotte API di rendering alternative, incluso OpenGL, come backend grafici paralleli del compositor AureaOS. Questa scelta può escludere applicazioni che richiedono X11/XWayland o OpenGL; la compatibilità dovrà essere misurata e documentata, non presunta.

Il vincolo riguarda il rendering dei componenti grafici sviluppati da AureaOS. Le applicazioni di terze parti possono avere requisiti grafici differenti e dovranno essere valutate singolarmente.

### Confine Wayland e compatibilità

Le applicazioni native Wayland sono il bersaglio di compatibilità desktop. Le applicazioni che richiedono X11 o XWayland non rientrano nel bersaglio iniziale.

La presenza di un’applicazione su Flathub non garantisce che funzioni su AureaOS. Occorre valutare protocollo grafico, toolkit, runtime, dipendenze e API di rendering effettivamente richieste.

## 5. Modularità e responsabilità

La modularità è un requisito architetturale, non una preferenza estetica.

- Ogni componente deve avere una responsabilità primaria chiara.
- I confini tra componenti devono essere espressi tramite API e contratti documentati.
- La radice di composizione coordina, ma non ingloba la logica dei componenti.
- La shell gestisce il ciclo di vita della sessione; il compositor gestisce superfici, rendering e uscite.
- L’API desktop espone contratti condivisi e non deve duplicare il comportamento interno dei componenti.
- La gestione dei dispositivi e delle uscite deve avere un proprietario definito: non devono esistere due componenti che competono per lo stesso dispositivo o per la stessa configurazione video.
- Preferire moduli piccoli, testabili e con una responsabilità singola. Evitare moduli monolitici che accorpano specifiche o sottosistemi indipendenti.

Ogni integrazione deve avere un modulo responsabile, un contratto documentato, test mirati e uno stato esplicito: implementata, parziale, sperimentale o pianificata.

## 6. Infrastruttura Linux e servizi

AureaOS intende riutilizzare infrastrutture consolidate per le funzioni che non definiscono direttamente la propria identità:

- PipeWire per l’infrastruttura audio e multimediale.
- NetworkManager per la gestione della rete.
- systemd per avvio, servizi e gestione del sistema.
- DRM/KMS e Udev per le interfacce hardware grafiche.
- Flatpak e Flathub per applicazioni di terze parti.

L’integrazione deve rispettare standard Linux documentati e mantenere confini di sicurezza chiari. Non si devono inventare sostituti privati quando uno standard esistente offre interoperabilità e manutenzione migliori.

## 7. Distribuzione e aggiornamenti

AureaOS distingue tre domini di aggiornamento:

1. **Base Debian:** pacchetti e aggiornamenti di sicurezza forniti da Debian.
2. **Sistema AureaOS:** componenti desktop, applicazioni, servizi e risorse di sistema mantenuti dal progetto.
3. **Applicazioni Flatpak:** applicazioni di terze parti installate e gestite tramite Aurea Store e Flatpak, da sorgenti supportate come Flathub.

Ogni dominio deve avere responsabilità chiare, verifica d’integrità e procedure di aggiornamento adeguate.

I meccanismi completi del canale di aggiornamento AureaOS e l’integrazione definitiva del negozio non sono ancora implementati.

## 8. Regola di verifica

Una funzionalità non è completa solo perché compila.

Prima di dichiararla pronta occorre verificare, secondo il suo ambito:

- Gestione di input validi e non validi.
- Comportamento in caso di errore, rifiuto dei permessi e annullamento.
- Interazione prevedibile con client di terze parti.
- Gestione corretta del ciclo di vita, inclusi disconnessione, riavvio e rimozione.
- Test del contratto pubblico.
- Comportamento comprensibile per l’utente e documentazione aggiornata.
- Assenza di dipendenze nascoste da X11, XWayland, OpenGL o da un altro ambiente desktop.

La compilazione in CI, i test automatizzati e la validazione su hardware reale sono livelli di verifica distinti. Non devono essere confusi.

## 9. Licenza

Il lavoro originale AureaOS è attualmente distribuito con licenza **GPL-3.0-or-later**. Il termine *prima parte* identifica software progettato, sviluppato e mantenuto nell’ambito del progetto: non significa software chiuso o proprietario.

Dipendenze, firmware, applicazioni e risorse di terze parti restano soggetti alle rispettive licenze e ai relativi obblighi.
