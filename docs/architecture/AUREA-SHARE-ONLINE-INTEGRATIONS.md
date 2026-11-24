<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Aurea Share — Online service integrations

- Status: Product architecture and API research; no provider integration or dependency approved.
- Platform baseline: Debian GNU/Linux 13 (Trixie), amd64.
- Scope: share user-selected local content from AureaOS applications to online services, with explicit account authorization and user confirmation.
- Related: Aurea Link handles device-to-device transfer. This document defines the separate online-service sharing path.

## 1. Product definition

Aurea Share is a system-wide share broker. From Fides, image/PDF viewers, media applications and document applications, a user can select **Share** and choose a destination such as Telegram, WhatsApp, Facebook, Instagram, Reddit, YouTube, email or another compatible service.

The goal is to avoid manually opening an application, finding the file again and attaching it. AureaOS should pass the selected content directly to the destination integration. Depending on the provider's supported interface, the final step may be:
1. a direct API operation after explicit authorization and confirmation;
2. an official share/deep-link handoff that opens a prefilled compose screen;
3. a registered desktop application receiving a file/URL through supported OS integration.

Do not promise a fully silent/direct send when the provider does not expose that capability. The user must always see the destination/account and confirm publication or sending.

## 2. Separate product paths

- **Aurea Share Online:** provider adapters for APIs, OAuth, supported share intents/deep links and registered desktop applications.
- **Aurea Link:** local peer-to-peer file transfer, pairing and safe receive lifecycle. It is not a social-network connector.
- **Aurea Share UI:** one consistent destination chooser, preview, caption/message editor, account selector, permissions explanation and final confirmation.

Do not merge these concerns into Fides, the media decoder, compositor or GStreamer pipeline.

## 3. Provider capability research (initial, not a guarantee)

### Telegram
- Telegram documents three developer interfaces: Bot API, Telegram API/MTProto and TDLib.
- Bot API can send documents and media to chats where the bot is authorized; this is not equivalent to sending from the user's personal Telegram identity or arbitrary private chats.
- Telegram API/TDLib can support a custom Telegram client, but requires a much larger client/security/maintenance commitment and must comply with Telegram API terms.
- Share URLs/deep links may be suitable for handing off links/text; verify current supported file handoff before relying on it.
- Initial approach: registered Telegram Desktop file handoff or supported share URL; consider TDLib only after a separate security and product review. Never build a bot that silently messages contacts as if it were the user.

### WhatsApp
- WhatsApp Cloud API is a Business Platform API, not a general-purpose personal-account API.
- It supports authorized business messaging and media flows subject to account onboarding, permissions, conversation/template rules and provider policy.
- wa.me click-to-chat can open a conversation and prefill text; it does not itself provide a generic arbitrary local-file attachment API.
- Initial approach: supported desktop application handoff or click-to-chat for text/link where appropriate. Cloud API is an optional business-account adapter, not the personal WhatsApp path.

### Facebook
- Meta Graph API supports publishing to Facebook Pages with appropriate Page access token, role/task and permissions (including pages_manage_posts and related permissions).
- Do not assume equivalent API access to personal profile timelines.
- Initial approach: Page publishing adapter after OAuth/app review/permissions validation; otherwise supported application handoff.

### Instagram
- Instagram Graph API publishing is for eligible Professional (Business/Creator) accounts, with required account linkage and permissions; consumer personal accounts are not generally covered by the same publishing API.
- Publishing formats and features vary; verify current support for feed, reels and stories before exposing a destination.
- Initial approach: Professional-account adapter only when authorized and supported; otherwise native app handoff if available. Do not promise personal-account direct publishing.

### Reddit
- Reddit Data API requires OAuth and is subject to current Developer Terms, Data API Terms and Responsible Builder Policy. Access may require explicit approval.
- Commercial use requires explicit written permission; this is a release gate for AureaOS distribution/monetization and must be reviewed with counsel/provider.
- Posting/media submission capabilities and scopes must be checked against current official API docs at implementation time.
- Initial approach: link/text handoff; direct post/media adapter only after access and commercial-use approval.

### YouTube
- YouTube Data API videos.insert supports video upload and metadata for the authenticated channel, with OAuth scope such as youtube.upload, quota and policy requirements.
- This is channel publishing, not sending a video to another user as a private chat attachment.
- Initial approach: optional creator workflow in a dedicated upload dialog; never publish by default. Require title, visibility, channel and final confirmation.

### Email and other services
- Email should use a standard mail client handoff or a separately authorized mail connector; attachment size and account security policies apply.
- Add providers through capability discovery and explicit adapters, not hardcoded assumptions about a universal API.

## Account model and user profiles

AureaOS must not assume whether the person using it is a child, older adult, casual user, creator, professional or business. Aurea Share offers the same coherent system capability to everyone; provider eligibility and available actions are determined by the connected account and the provider's current rules, not by a presumed user persona.

- Account connection is optional and initiated by the user when they choose a provider.
- Provide an account manager listing connected provider accounts, account type, granted permissions, connection state and available actions.
- Support multiple accounts for a provider where its official authentication/API model allows it (for example, multiple Telegram identities or multiple eligible Meta Pages/accounts). Do not promise multi-account support where the provider does not permit it.
- Keep account credentials and tokens isolated per provider and account. Allow disconnect/revoke and explain the effect.
- Personal, Creator, Professional, Page, Business and Channel account classes must be represented explicitly where relevant.
- A connected account does not imply every capability is available. Re-evaluate capabilities using provider account type, scopes, app review, region, content kind and current limits.
- Never force account linking during OS setup or make online accounts mandatory for core desktop use.
- The share chooser must be useful with zero connected accounts: show supported desktop handoff destinations and a clear route to connect an account only when the user chooses an API-backed action.
- Accessibility and clarity are system requirements; do not simplify by hiding important permission, recipient or publication details.


## 4. Adapter contract

Define a provider-neutral capability model before implementing providers. A provider adapter should declare:
- supported content kinds: file, image, video, audio, PDF/document, URL, text;
- supported actions: attach-to-compose, send, publish, upload, share-link;
- account type requirements: personal, professional, business, page/channel;
- authentication model and OAuth scopes;
- maximum file size, accepted MIME types, format/duration constraints and quota/rate limits;
- whether the final compose/confirmation screen is mandatory;
- provider review/approval and commercial-use constraints;
- capability version, source URL and last verification date.

A provider may support only some content types/actions. UI must show only capabilities actually supported for the selected account and content. For example, sharing a PDF to YouTube is invalid; uploading a video to a channel is a different action from sending a video in a chat.

## 5. Security, credentials and privacy

- Use OAuth 2.0 Authorization Code with PKCE where supported; follow each provider's exact current flow.
- Never ask users to paste passwords into AureaOS and never store provider passwords.
- Store tokens in the system credential store/keyring with least privilege; encrypt at rest where supported.
- Keep client secrets out of desktop binaries. A public desktop OAuth client cannot safely embed a confidential secret.
- Separate tokens by provider/account and support disconnect/revoke.
- Request only scopes required for the selected capability; explain permissions in plain language.
- Never upload, post, send or publish without a clear user action and destination summary.
- Show account, recipient/Page/channel/community, file name, size, visibility and caption before final action.
- Keep an audit record of action outcome without storing content or private message bodies unnecessarily.
- Respect provider rate limits, API terms, branding rules, retention rules and app review requirements.
- Do not scrape websites, reverse engineer private endpoints, automate browser UI to evade API limits, or impersonate the user.
- If an API is unavailable, use a supported official handoff or mark the provider unsupported; never silently fall back to an unofficial/private endpoint.

## 6. Proposed modular architecture

Logical Rust components; do not create a large monolith:
- aurea-share-core: content descriptors, action model, capability registry and stable contracts.
- aurea-share-auth: OAuth flow orchestration, token lifecycle and credential-store adapter.
- aurea-share-provider-telegram: Telegram capability adapter, only after chosen integration path is validated.
- aurea-share-provider-meta: separate Facebook Page and Instagram Professional capabilities; never assume same permissions.
- aurea-share-provider-whatsapp: business API adapter only if product requirements justify it.
- aurea-share-provider-reddit: gated on access and commercial-use approval.
- aurea-share-provider-youtube: channel upload workflow.
- aurea-share-desktop: registered application/deep-link handoff.
- aurea-share-ui: accessible share chooser and confirmation.

Start with a small core and one provider at a time. Avoid a daemon unless token/session lifecycle or sandbox boundaries demonstrate a need. Keep OAuth browser callbacks, secrets and file access out of the compositor.

## 7. Delivery plan

1. Finalize provider capability matrix and legal/terms review for AureaOS distribution.
2. Define provider-neutral content/action contract and secure OAuth/token storage.
3. Implement the share chooser for Aurea first-party applications with desktop handoff.
4. Prototype one low-risk provider and validate end-to-end file handling, account selection and confirmation.
5. Add Telegram/Meta/WhatsApp/Reddit/YouTube adapters only within each provider's permitted scope and after app registration/review.
6. Add automated conformance tests for MIME/size boundaries, token expiry/revocation, cancellation, failed upload, duplicate submission and user confirmation.

## 8. Non-goals

- Bypassing provider login, consent, API review, business eligibility or rate limits.
- Sending from a personal account through a bot/business API while pretending it is the user's personal session.
- Scraping websites or using undocumented/private endpoints.
- Silent publishing or sending.
- Treating all social services as if they support the same API or file types.
- Making network sharing, social API integrations or cloud accounts mandatory for the desktop.

The product objective is one coherent Share experience over a truthful, permission-aware set of provider capabilities. AureaOS owns the UX and adapter contracts; providers retain control over what their APIs permit.
