<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Aurea Share — Integrazioni con servizi online

- Stato: architettura prodotto e ricerca API; nessuna integrazione o dipendenza approvata.
- Base piattaforma: Debian GNU/Linux 13 (Trixie), amd64.
- Ambito: condividere contenuti locali selezionati dalle app AureaOS verso servizi online, con autorizzazione dell'account e conferma dell'utente.
- Correlato: Aurea Link gestisce i trasferimenti tra dispositivi. Questo documento definisce il percorso distinto verso i servizi online.

## 1. Definizione del prodotto

Aurea Share è un broker di condivisione di sistema. Da Fides, visualizzatori di immagini/PDF, app multimediali e documentali, l'utente sceglie **Condividi** e una destinazione come Telegram, WhatsApp, Facebook, Instagram, Reddit, YouTube, email o un altro servizio compatibile.

L'obiettivo è evitare di aprire manualmente l'applicazione, ritrovare il file e allegarlo. AureaOS deve consegnare direttamente il contenuto selezionato all'integrazione della destinazione. In base alle interfacce supportate dal provider, il passaggio finale può essere:
1. operazione API diretta dopo autorizzazione e conferma esplicite;
2. passaggio tramite share/deep link ufficiale che apre una schermata di composizione precompilata;
3. applicazione desktop registrata che riceve file/URL tramite integrazione OS supportata.

Non promettere invii silenziosi o completamente diretti quando il provider non li consente. L'utente deve sempre vedere destinazione/account e confermare invio o pubblicazione.

## 2. Percorsi distinti

- **Aurea Share Online:** adapter provider per API, OAuth, share intent/deep link supportati e applicazioni desktop registrate.
- **Aurea Link:** trasferimento locale peer-to-peer, pairing e ricezione sicura. Non è un connettore social.
- **Aurea Share UI:** selettore destinazioni coerente, anteprima, editor didascalia/messaggio, selezione account, spiegazione permessi e conferma finale.

Non inserire queste responsabilità in Fides, decoder media, compositor o pipeline GStreamer.

## 3. Ricerca iniziale capacità provider (non è una garanzia)

### Telegram
- Telegram documenta tre interfacce: Bot API, Telegram API/MTProto e TDLib.
- Bot API può inviare documenti e media alle chat in cui il bot è autorizzato; non equivale all'invio con identità personale Telegram dell'utente né consente qualsiasi chat privata.
- Telegram API/TDLib possono supportare un client Telegram personalizzato, ma comportano un impegno molto maggiore di sviluppo, sicurezza e manutenzione e devono rispettare i termini API Telegram.
- Share URL/deep link possono essere adatti al passaggio di link/testo; verificare il supporto attuale per file prima di farvi affidamento.
- Approccio iniziale: handoff file verso Telegram Desktop registrato o share URL supportato; valutare TDLib solo dopo revisione separata. Mai usare un bot per inviare messaggi ai contatti fingendo di essere l'utente.

### WhatsApp
- WhatsApp Cloud API è una API della Business Platform, non una API generale per account personali.
- Supporta messaggistica business autorizzata e flussi media soggetti a onboarding, permessi, regole conversazione/template e policy provider.
- wa.me può aprire una conversazione e precompilare testo; non fornisce da solo un'API generica per allegare file locali arbitrari.
- Approccio iniziale: handoff all'app desktop supportata o click-to-chat per testo/link quando appropriato. Cloud API è un adapter opzionale per account business, non il percorso WhatsApp personale.

### Facebook
- Graph API Meta supporta la pubblicazione sulle Pagine Facebook con token Pagina, ruolo/attività e permessi appropriati (tra cui pages_manage_posts e permessi correlati).
- Non presumere accesso API equivalente alle bacheche dei profili personali.
- Approccio iniziale: adapter pubblicazione Pagine dopo verifica OAuth/revisione app/permessi; altrimenti handoff all'applicazione supportata.

### Instagram
- Instagram Graph API permette la pubblicazione su account Professional idonei (Business/Creator), con collegamento account e permessi richiesti; gli account personali consumer non sono generalmente coperti dalla stessa API.
- Formati e funzioni variano; verificare il supporto attuale per feed, reels e stories prima di esporre una destinazione.
- Approccio iniziale: adapter per account Professional solo se autorizzato e supportato; altrimenti handoff all'app nativa se disponibile. Non promettere pubblicazione diretta su account personali.

### Reddit
- Reddit Data API richiede OAuth ed è soggetta a Developer Terms, Data API Terms e Responsible Builder Policy correnti. Può richiedere approvazione esplicita.
- L'uso commerciale richiede autorizzazione scritta esplicita: gate obbligatorio per distribuzione/monetizzazione AureaOS e da sottoporre a revisione legale/provider.
- Capacità di pubblicazione e invio media e relativi scope vanno verificati nella documentazione API ufficiale al momento dell'implementazione.
- Approccio iniziale: handoff link/testo; adapter post/media solo dopo approvazione accesso e uso commerciale.

### YouTube
- YouTube Data API videos.insert supporta caricamento video e metadati sul canale autenticato, con OAuth (ad esempio scope youtube.upload), quote e policy.
- È pubblicazione su un canale, non invio di un video a un utente come allegato di chat privata.
- Approccio iniziale: flusso creator opzionale in una finestra di caricamento dedicata; mai pubblicare per impostazione predefinita. Richiedere titolo, visibilità, canale e conferma finale.

### Email e altri servizi
- Email tramite handoff a un client standard o connettore mail autorizzato separatamente; valgono limiti allegati e policy di sicurezza account.
- Aggiungere provider tramite rilevamento capacità e adapter espliciti, non con supposizioni su una API universale.

## 4. Contratto degli adapter

Definire un modello di capacità indipendente dal provider prima di implementare i singoli servizi. Ogni adapter dichiara:
- tipi contenuto: file, immagine, video, audio, PDF/documento, URL, testo;
- azioni: allega alla composizione, invia, pubblica, carica, condividi link;
- requisiti account: personale, professionale, business, pagina/canale;
- autenticazione e scope OAuth;
- dimensione massima, MIME accettati, vincoli formato/durata e quote/rate limit;
- obbligo o meno di schermata finale di composizione/conferma;
- revisione provider/app e vincoli uso commerciale;
- versione capacità, URL fonte e data ultima verifica.

Un provider può supportare solo alcuni tipi di contenuto e azioni. L'interfaccia mostra solo capacità realmente supportate dall'account e dal contenuto selezionato. Un PDF non si condivide su YouTube come video; caricare un video su un canale è diverso dall'inviare un video in chat.

## 5. Sicurezza, credenziali e privacy

- Usare OAuth 2.0 Authorization Code con PKCE dove supportato; seguire il flusso esatto e aggiornato di ogni provider.
- Mai chiedere password agli utenti né memorizzare password provider.
- Conservare token nel credential store/keyring di sistema con privilegi minimi; cifrare a riposo dove supportato.
- Non inserire client secret nei binari desktop. Un client OAuth pubblico non può custodire in sicurezza un secret confidenziale.
- Separare token per provider/account e consentire disconnessione/revoca.
- Richiedere solo gli scope necessari e spiegare i permessi in linguaggio semplice.
- Mai caricare, pubblicare o inviare senza azione chiara dell'utente e riepilogo destinazione.
- Mostrare account, destinatario/Pagina/canale/community, nome file, dimensione, visibilità e didascalia prima dell'azione finale.
- Registrare l'esito senza conservare inutilmente contenuti o corpi di messaggi privati.
- Rispettare rate limit, termini API, regole marchio, retention e requisiti di revisione app.
- Non fare scraping, reverse engineering di endpoint privati, automazione browser per aggirare limiti API o impersonificazione.
- Se un'API non è disponibile, usare un handoff ufficiale supportato o dichiarare il provider non supportato; mai ripiegare silenziosamente su endpoint privati.

## 6. Architettura modulare proposta

Componenti Rust logici; evitare un monolite:
- aurea-share-core: descrittori contenuto, modello azioni, registro capacità e contratti stabili.
- aurea-share-auth: orchestrazione OAuth, ciclo token e adapter credential store.
- aurea-share-provider-telegram: adapter Telegram, solo dopo aver validato il percorso scelto.
- aurea-share-provider-meta: capacità separate Facebook Page e Instagram Professional; non presumere permessi uguali.
- aurea-share-provider-whatsapp: adapter Business API solo se giustificato dal prodotto.
- aurea-share-provider-reddit: bloccato finché non arrivano accesso e approvazione uso commerciale.
- aurea-share-provider-youtube: flusso upload canale.
- aurea-share-desktop: handoff applicazioni registrate/deep link.
- aurea-share-ui: selettore e conferma accessibili.

Iniziare con core piccolo e un provider per volta. Evitare un daemon salvo necessità dimostrata da ciclo token/sessione o sandbox. Callback OAuth, segreti e accesso file restano fuori dal compositor.

## 7. Piano di rilascio

1. Completare matrice capacità provider e revisione legale/termini per distribuzione AureaOS.
2. Definire contratto contenuto/azioni neutrale e storage sicuro OAuth/token.
3. Implementare share chooser per app proprietarie AureaOS con handoff desktop.
4. Prototipare un provider a basso rischio e validare file, selezione account e conferma end-to-end.
5. Aggiungere adapter Telegram/Meta/WhatsApp/Reddit/YouTube solo entro gli ambiti consentiti e dopo registrazione/revisione app.
6. Aggiungere test di conformità per MIME/dimensioni, scadenza/revoca token, annullamento, upload fallito, doppio invio e conferma utente.

## 8. Fuori ambito

- Aggirare login, consenso, revisione API, requisiti business o rate limit.
- Inviare tramite bot/API business fingendo di essere la sessione personale dell'utente.
- Scraping o endpoint privati/non documentati.
- Pubblicazione o invio silenziosi.
- Trattare tutti i social come se avessero le stesse API o supportassero gli stessi file.
- Rendere obbligatori rete, integrazioni social o account cloud per il desktop.

Obiettivo: un'unica esperienza Condividi coerente, basata su capacità provider reali e consapevoli dei permessi. AureaOS possiede UX e contratti adapter; i provider mantengono il controllo su ciò che le loro API consentono.
