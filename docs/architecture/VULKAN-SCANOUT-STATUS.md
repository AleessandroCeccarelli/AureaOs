<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Vulkan / DRM Scanout — Stato tecnico

Questo documento fotografa il percorso di presentazione fisica presente nel repository. Non equivale a una certificazione hardware né dichiara completato il desktop grafico.

## Stato di verifica (GitHub Actions, Ubuntu 24.04, 17 novembre 2026)

Il commit `b7494271848213a6efdcfb1cf626773e861c08ae` ha superato il workflow [Rust CI](https://github.com/AleessandroCeccarelli/AureaOs/actions/runs/44056628316): `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo check -p aurea-compositor --features drm --locked`, `cargo test --workspace --locked`, `cargo test -p aurea-compositor --features drm --locked`, compilazione degli shader e smoke test offscreen del triangolo Vulkan con Mesa lavapipe. Anche i controlli degli script bootstrap e delle fixture di sicurezza sono passati.

I sette errori individuati il 15 novembre sono stati corretti: visibilità dei tipi di upload, doppio prestito mutabile del backend di presentazione, vincoli di lifetime del binding colore e prestito concorrente della cache texture. La correzione del lifecycle porta il frame nello stato Rendering prima della registrazione, così un errore pre-submit può ancora essere annullato dal percorso di cleanup; gli upload vengono completati prima di prendere i riferimenti immutabili alle risorse della cache.

Il workflow non equivale a una verifica su Fedora né a una validazione di un monitor fisico. Il percorso DRM/KMS reale, hotplug, formati/modifier, DMA-BUF export, VBlank e teardown devono ancora essere provati su hardware.

## Stato implementato nel codice

Il percorso DRM attivo non usa il metodo generico `VulkanBackend::render_frame`, che resta intenzionalmente non disponibile finché non esiste un target di presentazione generico. Il server DRM instrada invece i frame richiesti verso `VulkanOutputScanoutBackend` in `drm/scanout_renderer.rs`.

La catena di presentazione è:

1. Il server apre la sessione libseat e il dispositivo DRM primario.
2. Udev individua i dispositivi; il compositor inizializza i dispositivi GBM e i backend Vulkan compatibili.
3. Per i dispositivi idonei vengono create le superfici DRM/KMS e pubblicati gli output Wayland.
4. Il ciclo Calloop individua gli output con un frame pendente.
5. Il backend negozia formato e modifier compatibili tra Vulkan e il piano primario KMS.
6. Viene allocata un'immagine Vulkan esportabile come DMA-BUF, registrata come framebuffer KMS.
7. Il compositor raccoglie gli snapshot SHM delle superfici XDG mappate, calcola posizione e dimensione fisica rispetto all'origine e alla scala dell'output, e scarta le superfici completamente fuori dall'area visibile.
8. I buffer SHM supportati vengono caricati come texture RGBA8; una pipeline Vulkan con blending compone le superfici nell'ordine della scena sopra il colore di sfondo.
9. Vulkan registra il rilascio dell'immagine alla famiglia queue esterna, attende il completamento GPU e KMS riceve il framebuffer tramite il backend di presentazione.
10. Il VBlank completa la transizione di presentazione e permette di riutilizzare o ritirare il framebuffer precedente.

Gli shader per la composizione vengono compilati da `build.rs` con `glslangValidator` durante le build con feature `drm`. Su Fedora il pacchetto richiesto è `glslang`.

## Limiti ancora aperti

- La composizione iniziale supporta i buffer Wayland SHM ARGB8888 e XRGB8888 già normalizzati dal percorso di snapshot. Gli snapshot vengono aggiornati anche quando un client riutilizza lo stesso wl_buffer e invia nuovo damage; per ora viene ricopiato l'intero buffer.
- I buffer Wayland DMA-BUF non sono ancora importati come superfici; non vanno confusi con il DMA-BUF usato come destinazione di scanout.
- Le trasformazioni buffer diverse da `Transform::Normal` vengono rifiutate esplicitamente: non vengono presentate texture ruotate o specchiate in modo scorretto.
- Il codice include `VulkanPendingUpload`, upload raggruppati e una cache limitata delle texture SHM indicizzata per superficie/revisione. Questa implementazione ha superato compilazione e test CI, ma non è ancora stata validata su hardware fisico.
- Pipeline grafica SHM, descriptor-set layout e sampler sono mantenuti in cache per dispositivo. Compilazione, test automatizzati e smoke test Vulkan offscreen sono passati; non vanno comunque considerati prova di una pipeline pronta per l'uso quotidiano prima della validazione fisica.
- Il limite aggregato del frame è 256 MiB di snapshot SHM visibili e 128 superfici; i singoli snapshot hanno inoltre il proprio limite.
- Il percorso fisico deve ancora essere verificato su hardware reale, inclusi formati/modifier, DMA-BUF export, ownership delle queue, blending, atomic KMS, VBlank, hotplug e teardown.
- Il successo di una build o dei test unitari non dimostra che il monitor mostri correttamente le finestre.

## Prossimi passi, in ordine

1. Ripetere su Fedora `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo check -p aurea-compositor --features drm --locked` e i test, confrontando l'ambiente locale con il CI Ubuntu.
2. Verificare la composizione SHM con client Wayland reali e controllare ordine delle superfici, alpha, clipping, scala e multi-monitor.
3. Aggiungere import e sincronizzazione dei buffer DMA-BUF Wayland con controlli rigorosi su formato, modifier, stride, offset e ownership.
4. Implementare le trasformazioni buffer non standard e la gestione corretta di viewport/damage.
5. Validare e consolidare la cache texture per revisione e gli upload raggruppati già presenti nel sorgente; il damage incrementale resta da implementare. Non rilasciare risorse ancora usate dalla GPU.
6. Verificare errori, hot-unplug, sospensione/ripresa e teardown con più output prima di considerare il percorso affidabile.

## Regole di implementazione

- Vulkan resta l'unica API di rendering; non introdurre OpenGL, EGL o un backend grafico alternativo.
- Wayland resta l'unico protocollo di sessione; non introdurre X11 o XWayland.
- Tenere separati composizione della scena, allocazione delle immagini, sincronizzazione e presentazione KMS.
- Non rilasciare immagini o framebuffer finché GPU e KMS non hanno restituito la proprietà.
- Non trasformare errori di import o sincronizzazione in fallback silenziosi che mostrino contenuti incompleti.
- Mantenere moduli piccoli e responsabilità singole.

## Verifica locale prevista

Dopo il pull su Fedora, verificare prima compilazione e test unitari. La prova fisica va eseguita solo da una sessione di test controllata, preferibilmente con un secondo VT o un ambiente dedicato e una procedura di ripristino. Registrare dispositivo DRM, GPU selezionata, output, formato/modifier scelto, numero di superfici composte, risultato del modeset/present e comportamento allo shutdown.

Fino al completamento di queste verifiche, il percorso va considerato **implementazione SHM iniziale in sviluppo**, non una pipeline desktop pronta per l'uso quotidiano.
