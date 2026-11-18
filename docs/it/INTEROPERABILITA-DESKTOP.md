<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Contratto di interoperabilità del desktop

## Scopo

AureaOS è responsabile della propria esperienza desktop e della relativa implementazione, ma non deve inventare un sostituto privato per ogni convenzione desktop. Le applicazioni devono comportarsi in modo prevedibile quando vengono installate, avviate, aperte da un file, autorizzate ad accedere a risorse o quando devono mostrare notifiche.

Questo documento definisce la direzione di interoperabilità di AureaOS. È un contratto architetturale e una roadmap, non la dichiarazione che ogni elemento sia già implementato.

## Principio di compatibilità

- Mantenere identità visiva, shell, impostazioni e applicazioni di prima parte progettate in modo indipendente da AureaOS.
- Preferire specifiche consolidate del desktop Linux per metadati delle applicazioni, gestione MIME, notifiche, integrazione sandbox e cartelle utente.
- Utilizzare API documentate e versionate per le funzionalità specifiche AureaOS.
- Non obbligare le applicazioni a dipendere da dettagli interni e privati della shell.
- Non dichiarare compatibilità X11 o XWayland: AureaOS punta a una sessione nativa Wayland.
- Valutare la compatibilità applicativa in base ai requisiti reali di esecuzione e grafica, non alla sola presenza di un’applicazione su Flathub.

## Interoperabilità con il desktop Linux

Il progetto dovrebbe implementare e testare le specifiche freedesktop.org/XDG pertinenti al prodotto.

| Ambito | Obiettivo di interoperabilità | Responsabilità AureaOS |
|---|---|---|
| Avvio applicazioni | Specifica Desktop Entry | Individuare, validare, avviare e gestire le applicazioni installate |
| Associazioni file | Database Shared MIME-info e dichiarazioni MIME nei desktop entry | Selezione prevedibile dell’applicazione e gestione delle applicazioni predefinite |
| Cartelle utente | Convenzioni XDG Base Directory e user-dirs | Rispettare percorsi di dati, configurazione e cache e cartelle standard localizzate |
| Notifiche | Specifica Desktop Notifications | Presentare notifiche coerenti con azioni e semantica di sostituzione |
| Applicazioni isolate | Interfacce XDG Desktop Portal | Accesso mediato ai file, schermate, notifiche e altre capacità supportate |
| Metadati applicazioni | Metadati AppStream | Popolare il negozio con nomi, icone, riepiloghi, categorie, licenze e informazioni di versione accurate |
| Avvio automatico | Specifica XDG Autostart | Applicare una politica di avvio esplicita per utente e sistema |
| Icone | Convenzioni freedesktop per nomi e temi delle icone, dove applicabili | Fornire grafica AureaOS mantenendo ricerca prevedibile e ripieghi |

Prima di implementare ogni integrazione, AureaOS deve consultare la specifica aggiornata e verificare il comportamento effettivo di toolkit e runtime. Le specifiche freedesktop sono standard di interoperabilità, non la garanzia che ogni ambiente desktop implementi ogni interfaccia in modo identico.

## Flatpak e portali

Le applicazioni Flatpak dovrebbero utilizzare il modello dei portali per l’integrazione desktop supportata. AureaOS dovrà fornire un backend dei portali adeguato a shell e compositor oppure dichiarare chiaramente le interfacce non supportate.

Il confine dei portali è anche un confine di sicurezza:

- Concedere la capacità più limitata necessaria all’operazione richiesta.
- Richiedere un consenso esplicito dell’utente per gli accessi sensibili.
- Non trasformare silenziosamente richieste dei portali in accesso illimitato al filesystem host.
- Conservare descrittori di file e autorizzazioni del document portal secondo il contratto previsto.
- Trattare metadati e parametri forniti dalle applicazioni come input non attendibili.
- Testare rifiuto, annullamento, revoca e uscita dell’applicazione, non solo i casi di accesso riuscito.

AureaOS non deve dichiarare supportata un’interfaccia portal finché comportamento e proprietà di sicurezza non siano implementati e verificati.

### Requisiti dei portali per screenshot e registrazione

Il backend dei portali è il confine d'integrazione previsto per le applicazioni isolate che richiedono screenshot o registrazione dello schermo.

- Implementare le interfacce XDG Screenshot e ScreenCast secondo le versioni pubblicate e i relativi contratti di richiesta e risposta.
- Mostrare un selettore delle sorgenti controllato dal sistema; le applicazioni non devono poter scegliere silenziosamente schermi o finestre arbitrari.
- Limitare ogni autorizzazione all'operazione, alla sorgente e alla sessione richieste. Screenshot, video dello schermo, microfono e audio di sistema sono capacità distinte.
- Restituire solo flussi, descrittori o risultati autorizzati dal contratto del portale. Non esporre buffer arbitrari del compositor o percorsi illimitati del filesystem host.
- Gestire annullamento, rifiuto, chiusura della sessione, disconnessione dell'applicazione e revoca.
- Dichiarare onestamente sorgenti e opzioni non supportate. Non pubblicizzare selezione area, modalità cursore, persistenza o acquisizione audio finché non siano implementate e verificate.
- Utilizzare il document portal o un flusso equivalente mediato dall'utente per i file di destinazione delle applicazioni isolate.

Il flusso video ScreenCast non concede automaticamente accesso all'audio di sistema o al microfono. L'acquisizione audio deve seguire un percorso PipeWire separato, con autorizzazioni e instradamento dedicati.

## Familiarità tra sistemi operativi

La continuità con Windows e macOS è un obiettivo di comportamento del prodotto, non una promessa di riprodurne API private o progettazione interna.

AureaOS dovrebbe offrire flussi familiari per:

- Navigazione da tastiera, focus, scorciatoie, copia/incolla, selezione e trascinamento.
- Spostamento, ridimensionamento, minimizzazione, massimizzazione, chiusura e cambio di finestre e workspace.
- Apertura, salvataggio, rinomina, spostamento, cestino e scelta delle applicazioni predefinite per i file.
- Connessione dei dispositivi, disposizione dei display, selezione audio e supporti rimovibili.
- Installazione, aggiornamento, permessi e rimozione delle applicazioni.
- Accessibilità, alternative da tastiera, dimensione del testo e feedback di sistema chiaro.

Quando Linux dispone già di una convenzione consolidata, AureaOS dovrebbe preferirla a un comportamento proprietario che renda più difficile l’integrazione delle applicazioni di terze parti.

## Confine Wayland e grafica

Wayland nativo è il protocollo desktop supportato. Le applicazioni che richiedono X11/XWayland sono fuori dal bersaglio iniziale di compatibilità e non devono essere presentate come supportate.

Vulkan resta l’API esclusiva di rendering per il compositor sviluppato da AureaOS. Questo non significa che ogni applicazione Linux disponibile tramite Flatpak utilizzi Vulkan. Le applicazioni di terze parti possono avere requisiti grafici propri: la compatibilità va valutata e documentata per ciascuna applicazione e runtime.

AureaOS non deve aggiungere silenziosamente un backend OpenGL del compositor per sostenere una dichiarazione di compatibilità commerciale.

## Ordine di implementazione

1. Individuazione delle applicazioni e validazione dei Desktop Entry.
2. Integrazione del database MIME e selezione delle applicazioni predefinite.
3. Cartelle utente XDG e integrazione con il gestore file.
4. Notifiche e gestione delle relative azioni.
5. Integrazione Flatpak e interfacce portal richieste dalla shell di prima parte.
6. Acquisizione dei metadati AppStream per il negozio.
7. Politica di avvio automatico e controlli visibili all’utente.
8. Verifiche di accessibilità e flussi da tastiera nella shell.

Ogni integrazione deve avere un modulo responsabile di dimensioni contenute, un contratto documentato, test mirati e un’etichetta esplicita dello stato di implementazione. Evitare un unico grande modulo di «integrazione desktop» che possieda specifiche non correlate.

## Criteri di accettazione

Un’integrazione non è completa solo perché compila. Deve dimostrare:

- Gestione di input validi e non validi.
- Comportamento in caso di rifiuto dei permessi e di errore.
- Interazione prevedibile con client di terze parti.
- Gestione corretta del ciclo di vita, inclusi disconnessione, riavvio e rimozione.
- Test del contratto pubblico.
- Comportamento chiaro per l’utente e documentazione.
- Nessuna dipendenza nascosta da X11, XWayland, OpenGL o da un altro ambiente desktop.

## Riferimenti

- [Repository delle specifiche freedesktop.org](https://specifications.freedesktop.org/)
- [Panoramica delle specifiche di interoperabilità freedesktop.org](https://wiki.freedesktop.org/www/Specifications/)
- [Documentazione Flatpak](https://docs.flatpak.org/)
- [Documentazione XDG Desktop Portal](https://flatpak.github.io/xdg-desktop-portal/)
