<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Visione di AureaOS

## Idea guida

**«La tecnologia è bella quando c’è e non si vede.»**

AureaOS vuole rendere Linux più semplice da usare senza nasconderne la flessibilità. Un sistema operativo dovrebbe aiutare le persone a fare ciò che desiderano, senza costringerle a gestire complessità non necessarie.

La tecnologia deve sostenere l’esperienza, non diventare un ostacolo.

## In cosa crediamo

- La tecnologia deve adattarsi alle persone.
- Semplicità e capacità avanzate possono convivere.
- Desktop, applicazioni, impostazioni e strumenti di sistema devono offrire un’esperienza coerente.
- Per usare il sistema non deve essere necessario un account cloud.
- Progettazione visiva e animazioni devono favorire chiarezza e usabilità.
- Installazione, aggiornamenti, backup e ripristino devono essere comprensibili e affidabili.
- Un’architettura modulare deve rendere il sistema più semplice da mantenere, estendere e migliorare.
- È preferibile riutilizzare software maturo quando farlo è più sensato che reinventarlo.
- Il sistema operativo deve avere una propria identità, senza obbligare il progetto a sviluppare ogni applicazione.

## Il nostro approccio

AureaOS è una distribuzione Linux orientata all’interfaccia grafica e a Wayland, basata su Debian GNU/Linux 13 (Trixie), destinata ad architettura `amd64` (x86-64). La prima versione prevista è **AureaOS Roma 1.0**.

Debian fornisce le fondamenta del sistema. AureaOS sviluppa il proprio ambiente desktop, l’integrazione di sistema, l’identità visiva e le applicazioni fondamentali.

L’obiettivo è costruire un’esperienza coerente, non limitarsi a riconfezionare Debian cambiandone l’aspetto.

AureaOS riutilizzerà infrastrutture Linux mature e software di terze parti selezionato quando ciò sarà tecnicamente e praticamente appropriato.

Il repository ha superato la sola fase iniziale di bootstrap Debian. Include prime implementazioni Rust di `aurea-shell` e `aurea-compositor`, insieme alle fondamenta per Wayland, rilevamento dei dispositivi DRM e gestione delle uscite video.

Questi componenti costituiscono una base tecnica iniziale, non un desktop grafico completo. Interfaccia desktop, pipeline Vulkan completa di rendering e presentazione, programma di installazione, immagine avviabile e applicazioni di uso finale sviluppate dal progetto sono ancora in sviluppo o pianificate.

Per lo stato effettivo e le prossime attività si veda la [roadmap](ROADMAP.md).

## Un desktop naturale da usare

Il desktop previsto dovrà essere moderno, accessibile e coerente.

Accenti caldi, tipografia curata, superfici traslucide e animazioni misurate sono indirizzi progettuali: dovranno sempre essere al servizio della leggibilità e dell’usabilità.

Le attività quotidiane dovranno poter essere svolte tramite interfacce grafiche. Il terminale resterà disponibile per utenti avanzati, sviluppatori e amministratori, ma le normali operazioni desktop non dovranno richiedere comandi da terminale.

Il desktop dovrà apparire come un ambiente integrato, non come una raccolta di applicazioni e strumenti scollegati.

AureaOS dovrà offrire impostazioni predefinite sensate, lasciando comunque all’utente il controllo del proprio sistema.

## Una nostra identità di prodotto

AureaOS è responsabile della progettazione e dello sviluppo della propria esperienza desktop e delle applicazioni di sistema che ne definiscono l’identità.

La prima versione è incentrata su:

- **Aurea Desktop:** ambiente desktop grafico progettato per AureaOS ed esperienza di gestione delle finestre.
- **AureaOS Human Interfaces:** sistema di progettazione condiviso e futura libreria di interfaccia nativa.
- **Aurea Wallet:** applicazione di sistema sviluppata dal progetto e pilastro importante del prodotto.
- **Fides:** gestore file di AureaOS.
- **Quaero:** ricerca globale di applicazioni, file, cartelle e impostazioni.
- **Aurea Store:** negozio ufficiale delle applicazioni.
- **Aurea Update:** aggiornamenti dei componenti di sistema AureaOS.
- **Aurea Time:** backup e ripristino del sistema.

Questi elementi descrivono la direzione di prodotto prevista per Roma 1.0. La loro presenza nella visione non significa che siano già implementati o completati.

## Un ecosistema applicativo pragmatico

AureaOS non deve reinventare ogni applicazione desktop per offrire un sistema operativo completo.

Per Roma 1.0, applicazioni di terze parti già consolidate copriranno inizialmente esigenze comuni come:

- Navigazione web.
- Posta elettronica.
- Ufficio e produttività.
- Altre attività quotidiane.

Aurea Store si integrerà con Flathub come sorgente applicativa e con Flatpak per installazione, esecuzione, isolamento e aggiornamenti.

Questo approccio permette ad AureaOS di concentrare le proprie risorse sull’ambiente desktop, Wallet, gestore file, ricerca globale, negozio e integrazione di sistema.

Le applicazioni di terze parti resteranno distinte dai prodotti sviluppati direttamente da AureaOS.

In versioni future si potranno valutare alternative sviluppate dal progetto per alcune categorie, quando risorse e priorità tecniche lo renderanno ragionevole.

## Tecnologia e architettura

AureaOS segue un’architettura modulare e utilizza Rust come linguaggio principale per lo sviluppo dei componenti proprietari del progetto, dove appropriato.

La direzione tecnologica comprende:

- Debian GNU/Linux 13 (Trixie).
- Wayland e Smithay per l’architettura grafica desktop.
- Vulkan come API esclusiva scelta per il rendering grafico.
- PipeWire per l’infrastruttura multimediale.
- NetworkManager per la gestione della rete.
- systemd per sistema e servizi.
- GRUB2 come bootloader previsto.
- Btrfs come filesystem predefinito selezionato per il sistema installato.

La scelta di queste tecnologie non significa che la loro integrazione completa sia già stata realizzata.

AureaOS vuole mantenere responsabilità ben separate tra base Debian, componenti di sistema AureaOS, infrastruttura desktop, applicazioni sviluppate dal progetto e software di terze parti.

L’architettura dovrà restare modulare, verificabile e manutenibile durante la crescita del progetto.

## Un sistema affidabile

Una buona esperienza desktop dipende da molto più dell’aspetto grafico.

AureaOS dovrà offrire:

- Un processo di installazione chiaro e affidabile.
- Aggiornamenti prevedibili.
- Responsabilità di aggiornamento separate per Debian, AureaOS e applicazioni Flatpak.
- Procedure comprensibili di backup e ripristino tramite Aurea Time.
- Confini di sicurezza adeguati e permessi applicativi trasparenti.
- Interfacce accessibili e comportamento coerente.
- Documentazione chiara su hardware supportato e limitazioni.

Affidabilità, manutenibilità e fiducia degli utenti devono avere priorità rispetto al numero di funzionalità.

## Prima versione

**AureaOS Roma 1.0** è il primo obiettivo di rilascio del progetto.

La piattaforma iniziale è Debian 13 Trixie su `amd64`.

La prima versione dovrà stabilire una base Debian affidabile e offrire un desktop AureaOS coerente, con le applicazioni fondamentali e l’integrazione di sistema essenziale.

Dovrà inoltre consentire l’accesso ad applicazioni di terze parti consolidate attraverso Aurea Store.

L’obiettivo non è sviluppare ogni applicazione possibile. È realizzare un sistema operativo utilizzabile, con un’identità riconoscibile, un’esperienza coerente e solide fondamenta tecniche.

## Motto

**Prendere la semplicità che le persone apprezzano nei sistemi operativi e darle una forma adatta a Linux.**

AureaOS vuole rendere più accessibili le capacità di Linux, rispettando apertura e flessibilità della piattaforma.

## Licenza del progetto

Il lavoro originale di AureaOS è attualmente distribuito con licenza **GPL-3.0-or-later**. La titolarità dell’identità di prodotto, della direzione progettuale, dell’esperienza desktop e delle applicazioni sviluppate dal progetto non implica software a codice chiuso. I componenti di terze parti restano soggetti alle rispettive licenze. Ogni eventuale modifica futura a questa politica è ancora da decidere e sarà valutata separatamente.
