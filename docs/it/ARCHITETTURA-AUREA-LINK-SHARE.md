<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Aurea Link e Aurea Share — Architettura di condivisione tra dispositivi

- Stato: direzione di prodotto e proposta architetturale; nessuna implementazione o dipendenza è approvata qui.
- Destinazione: fondamenta AureaOS Roma 1.0 e successive app companion.
- Autorità di compatibilità: Debian GNU/Linux 13 (Trixie), amd64.
- Principio: semplice, consenso esplicito, locale prima di tutto, nessun account cloud obbligatorio.

## 1. Obiettivo

AureaOS deve scambiare file, contenuti multimediali e collegamenti con Android, iPhone/iPad, macOS e altri dispositivi Linux/Windows. L'utente deve poter selezionare foto, audio, video, documenti, link o testo e inviarli a un dispositivo vicino associato, oppure riceverli in una destinazione chiara.

Distinguere due flussi:
- **Passaggio tra dispositivi:** trasferimento diretto tra dispositivi dell'utente, preferibilmente sulla rete locale e senza account cloud.
- **Condivisione con applicazioni:** consegna di contenuti selezionati a un'app installata come Telegram o WhatsApp. AureaOS apre l'app/target registrato; l'utente controlla e conferma destinatario e messaggio nell'applicazione. AureaOS non automatizza API private di messaggistica.

## 2. Componenti

### Aurea Link — connessione e trasferimento

Gestisce pairing esplicito e identità peer, discovery locale, sessioni di trasferimento, consenso, avanzamento, annullamento, verifica integrità, destinazioni sicure, controlli privacy e adapter di protocollo dietro un'API Aurea stabile.

Perimetro iniziale: file, più file, directory quando sicuro, link e testo semplice. Conservare i byte originali: mai convertire silenziosamente i media. Anteprime facoltative e limitate nelle risorse.

### Aurea Share — esperienza di condivisione

Gestisce l'azione Condividi e il registro delle destinazioni, non il protocollo di trasporto. Può inviare contenuti ad Aurea Link, aprire app installate compatibili come Telegram/WhatsApp, offrire un'azione coerente in Fides e nelle app AureaOS e ricevere contenuti da target registrati/app companion.

Non presumere che ogni app desktop Linux supporti uno share sheet universale. Iniziare dalle app AureaOS e da adapter/desktop entry espliciti. XDG Desktop Portal resta il confine per accesso ai file e permessi sandbox; FileChooser non è un portale di condivisione.

## 3. Interoperabilità

### Primo obiettivo: LocalSend Protocol v2.x

Valutare e implementare l'interoperabilità con il protocollo REST pubblicato da LocalSend invece di inventare subito un protocollo proprietario. È progettato per trasferimenti sulla rete locale senza server esterni e supporta discovery e flussi HTTP(S).

- Esaminare specifica v2.2 e implementazione upstream prima del codice.
- Preferire HTTPS app-to-app con identità certificato autenticata/pinning e approvazione esplicita del destinatario.
- Verificare gli hash quando forniti; usare streaming limitato senza caricare file multimediali grandi in RAM.
- Gli annunci discovery sono indizi non attendibili, mai prova di identità o autorizzazione.
- Supportare multicast dove disponibile e pairing manuale IP/QR quando discovery è bloccato da isolamento Wi-Fi, VPN o policy.
- Versionare esplicitamente protocollo e test di compatibilità.
- Verificare licenza upstream, sicurezza, evoluzione protocollo e requisiti Debian prima di riutilizzare codice. Interoperabilità non significa copiare o includere il codice dell'app LocalSend.

LocalSend è una base di compatibilità multipiattaforma per Linux, macOS, Windows, Android e iOS, non l'interfaccia AureaOS né un'app obbligatoria da includere.

### Interoperabilità secondaria

- Valutare KDE Connect come ecosistema peer opzionale, soprattutto per share sheet iOS/Android, clipboard e controllo media. Non rendere KDE/Qt desktop una dipendenza AureaOS.
- Bluetooth OBEX o importazione USB potranno essere ripieghi, non il trasporto principale per media grandi.
- Apple AirDrop/AWDL resta ricerca. OWL e implementazioni reverse-engineered hanno limiti hardware/driver e stato sperimentale: non promettere compatibilità AirDrop né renderla dipendenza di rilascio.
- Una futura app companion Aurea Link dovrà integrarsi con Android Sharesheet (ACTION_SEND / ACTION_SEND_MULTIPLE) e iOS Share Extensions, per permettere di scegliere Aurea Link dai menu nativi di Foto, File, Telegram o WhatsApp, nel rispetto dei limiti delle estensioni e del background di ciascuna piattaforma.
- Un companion macOS potrà essere valutato in seguito. Inizialmente interoperare con client LocalSend esistenti su macOS, senza richiedere un'app Mac proprietaria.

## 4. Sicurezza e privacy

- Pairing esplicito: un dispositivo rilevato non è automaticamente attendibile.
- Prima dell'accettazione mostrare mittente, nomi file, numero, dimensione totale e destinazione.
- Chiedere conferma per dispositivi sconosciuti; policy dispositivi attendibili configurabile e revocabile.
- Mai sovrascrivere silenziosamente: nomi senza collisioni o richiesta all'utente.
- Validare nomi, percorsi, MIME e dimensioni; impedire path traversal, escape tramite symlink e file speciali.
- Scrivere in file temporaneo controllato, verificare lunghezza/hash e pubblicare atomicamente il file completo.
- Limiti per file, batch, sessioni simultanee, spazio disco e frequenza; annullamento e pulizia dopo interruzioni.
- TLS con identità peer autenticata per trasferimenti app-to-app. Mai downgrade silenzioso a HTTP non autenticato.
- Non esporre un file server LAN senza restrizioni; aprire solo interfacce necessarie, token brevi e listener chiusi quando inattivi.
- Nessun accesso a messaggi, notifiche, clipboard o contatti senza permesso separato ed esplicito.
- Non aprire automaticamente i contenuti ricevuti.

## 5. Confini architetturali

Moduli logici proposti, non necessariamente daemon separati:
- aurea-link-core: peer, pairing, macchina a stati, validazione e API.
- aurea-link-localsend: adapter protocollo e discovery/trasferimento.
- aurea-link-storage: ricezione temporanea, pubblicazione atomica e policy destinazioni.
- aurea-share: descrittori contenuto/intenti e risoluzione destinazioni.
- aurea-share-ui: share sheet Human Interfaces e UI trasferimenti.
- Companion mobile opzionale: integrazione nativa sottile sullo stesso protocollo e gestione esplicita di share extension/intent.

Iniziare con piccole librerie Rust e un componente di sessione ristretto solo se necessario. Non aggiungere subito un daemon monolitico sempre attivo. I listener si attivano solo quando la ricezione è abilitata o un trasferimento è attivo.

Aurea Link non entra nel compositor, nel render thread, nel decoder Aurea Media o nella pipeline GStreamer. I byte multimediali si trasferiscono come file; riproduzione e decodifica restano separate.

## 6. Esperienza utente

- Un'unica azione **Condividi** in Fides e nelle app proprietarie.
- Dispositivi vicini con nomi e tipo chiari; possibilità di rinominarli o rimuoverli.
- Invio: scegliere destinazione → controllare selezione → inviare → avanzamento → completamento.
- Ricezione: richiesta visibile → controllare mittente/contenuti → accettare/rifiutare → avanzamento → destinazione sicura.
- Pairing QR/manuale quando multicast non è disponibile.
- Linguaggio semplice, target ampi, tastiera, screen reader, avanzamento chiaro ed errori recuperabili.
- Nessuna necessità di conoscere IP, porte, certificati TLS o versioni protocollo.
- Trasferimento locale offline e senza cloud per impostazione predefinita.

## 7. Rilascio per fasi

1. Revisione protocollo/threat model: comportamento v2.x, licenza, sicurezza, discovery e pacchetti Debian. Nessuna dipendenza ancora.
2. Prototipo headless: invio/ricezione file piccoli e video grandi verso client LocalSend ufficiali su Fedora e Debian Trixie; test di rifiuto, annullamento, hash errato, metadati malformati, cambio rete e riavvio.
3. Core Aurea Link: pairing, identità, macchina a stati, streaming limitato, storage sicuro e diagnostica senza raccolta contenuti.
4. Integrazione Aurea Share: azione in Fides e app proprietarie; destinazioni Telegram/WhatsApp solo se installate e supportate.
5. Companion mobile: Android Sharesheet e iOS Share Extension; verificare limiti background e requisiti degli store.
6. Continuità avanzata: clipboard, notifiche, controlli remoti, passaggio riproduzione o AirDrop solo con progetti separati e revisione sicurezza indipendente.

## 8. Criteri di accettazione

- Interoperabilità tra AureaOS e almeno un client LocalSend mantenuto su Android, iOS e macOS; registrare le versioni testate.
- File singoli/multipli, video grandi, interruzioni, nomi duplicati, spazio insufficiente, rifiuto e scomparsa dispositivo gestiti correttamente.
- Integrità verificata; file parziali mai esposti come completi.
- Nessun trasferimento senza consenso; discovery da sola non crea fiducia.
- Debian 13 senza dipendenze di sessione GNOME/KDE, X11/XWayland o OpenGL.
- Nessun trasferimento blocca compositor o rendering.
- Accessibile da tastiera e screen reader, con avanzamento leggibile ed errori recuperabili.
- Nessun account cloud o Internet richiesto per trasferimenti LAN.

## 9. Fuori ambito iniziale

- Reimplementare Telegram, WhatsApp, AirDrop o tutte le funzioni KDE Connect.
- Leggere/sincronizzare messaggi privati, contatti, notifiche o clipboard per impostazione predefinita.
- Sincronizzare in background intere librerie fotografiche.
- Relay cloud, account o trasferimento Internet remoto.
- Streaming audio/video live tra dispositivi: primo ambito = file e link/testo.
- Includere codice di app terze senza revisione separata di licenza e manutenzione.

Primo traguardo: trasferimento locale affidabile, sicuro e multipiattaforma con un'esperienza AureaOS coerente.
