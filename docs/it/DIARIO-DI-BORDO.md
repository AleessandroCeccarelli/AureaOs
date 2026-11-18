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
