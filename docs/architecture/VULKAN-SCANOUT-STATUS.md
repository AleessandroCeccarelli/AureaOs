<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Vulkan / DRM Scanout — Stato tecnico

Questo documento fotografa il percorso di presentazione fisica presente nel repository. Non equivale a una certificazione hardware né dichiara completato il desktop grafico.

## Stato implementato

Il percorso DRM attivo non usa il metodo generico `VulkanBackend::render_frame`, che resta intenzionalmente non disponibile finché non esiste un target di presentazione generico. Il server DRM instrada invece i frame richiesti verso `VulkanOutputScanoutBackend` in `drm/scanout_renderer.rs`.

La catena attuale è:

1. Il server apre la sessione libseat e il dispositivo DRM primario.
2. Udev individua i dispositivi; il compositor inizializza i dispositivi GBM e i backend Vulkan compatibili.
3. Per i dispositivi idonei vengono create le superfici DRM/KMS e pubblicati gli output Wayland.
4. Il ciclo Calloop individua gli output con un frame pendente.
5. Il backend negozia formato e modifier compatibili tra Vulkan e il piano primario KMS.
6. Viene allocata un'immagine Vulkan esportabile come DMA-BUF, registrata come framebuffer KMS.
7. Vulkan registra il clear pass, trasferisce la proprietà dell'immagine alla famiglia queue esterna e attende il completamento GPU.
8. KMS riceve il framebuffer tramite il backend di presentazione. Il VBlank completa la transizione di presentazione e permette di ritirare il framebuffer precedente.

Il primo contenuto presentato è un colore uniforme blu scuro (`FIRST_FRAME_CLEAR_COLOR`). Questo è un primo percorso Vulkan → DMA-BUF/GBM → KMS, non ancora il rendering delle finestre o dell'interfaccia AureaOS.

## Limiti ancora aperti

- `RenderScene` viene costruita dal compositor, ma il backend di scanout attuale non la usa.
- Le superfici XDG mappate non sono ancora importate/composite nell'immagine Vulkan di scanout.
- Il frame viene allocato per il percorso iniziale e ritirato dopo il rilascio KMS; il riuso efficiente delle immagini e la gestione delle dimensioni/formati al cambio modalità sono lavoro successivo.
- Il percorso fisico deve ancora essere verificato su hardware reale, inclusi formati/modifier, DMA-BUF export, ownership delle queue, atomic KMS, VBlank, hotplug e teardown.
- Il successo di una build o dei test unitari non dimostra che il monitor mostri correttamente il frame.

## Prossimi passi, in ordine

1. Validare su Fedora il frame uniforme su un output reale, senza avviare test KMS distruttivi sulla sessione grafica in uso.
2. Rendere esplicito il contratto di frame e di ownership tra rendering Vulkan e presentazione KMS; mantenere la risorsa viva fino al rilascio confermato.
3. Introdurre un percorso di import delle superfici Wayland compatibile con i buffer supportati, con controlli rigorosi su formato, modifier, stride, offset e sincronizzazione.
4. Comporre gli elementi della scena nel target Vulkan e presentare il risultato attraverso lo stesso percorso KMS già esistente.
5. Aggiungere riuso controllato dei frame e ricreazione delle risorse quando cambiano modalità o dimensioni dell'output.
6. Verificare errori, hot-unplug, sospensione/ripresa e teardown con più output prima di considerare il percorso affidabile.

## Regole di implementazione

- Vulkan resta l'unica API di rendering; non introdurre OpenGL, EGL o un backend grafico alternativo.
- Wayland resta l'unico protocollo di sessione; non introdurre X11 o XWayland.
- Tenere separati composizione della scena, allocazione delle immagini, sincronizzazione e presentazione KMS.
- Non rilasciare immagini o framebuffer finché GPU e KMS non hanno restituito la proprietà.
- Non trasformare errori di import o sincronizzazione in fallback silenziosi che mostrino contenuti incompleti.
- Mantenere moduli piccoli e responsabilità singole.

## Verifica locale prevista

Dopo il pull su Fedora, verificare prima compilazione e test unitari. La prova fisica va eseguita solo da una sessione di test controllata, preferibilmente con un secondo VT o un ambiente dedicato e una procedura di ripristino. Registrare dispositivo DRM, GPU selezionata, output, formato/modifier scelto, risultato del modeset/present e comportamento allo shutdown.

Fino al completamento di queste verifiche, il percorso va considerato **implementazione iniziale in sviluppo**, non una pipeline desktop pronta per l'uso quotidiano.
