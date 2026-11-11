<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Aurea Wallet

## Purpose

Aurea Wallet is the native digital wallet and credential manager of AureaOS.

It is designed to provide users with a secure, integrated place to manage:

- Passkeys.
- Website and application passwords.
- Credit and debit card details.
- Other payment credentials supported by future integrations.

Aurea Wallet is a fundamental component of the AureaOS product and a mandatory requirement for the Roma 1.0 release.

AureaOS Roma 1.0 must not be released without a functional and security-validated Aurea Wallet.

Its purpose is not limited to storing information. It should make everyday authentication and online payments easier by allowing authorized applications, especially web browsers, to request credentials through a controlled system interface.

Aurea Wallet must remain under the user's control. Applications must never have unrestricted access to stored credentials or payment information.

This document defines the product scope, architectural principles, security requirements, and implementation direction.

The requirements describe the intended product. They do not imply that Aurea Wallet or its integrations are already implemented.

## 1. Product Scope

Aurea Wallet is a native AureaOS system application with two closely related responsibilities:

1. Securely storing and managing digital credentials.
2. Providing controlled access to those credentials when the user needs them.

The initial scope includes passkeys, passwords, and payment cards.

### 1.1 Passkeys

Aurea Wallet should support passkeys based on established authentication standards.

The intended direction includes:

- Creating and registering passkeys with compatible websites and services.
- Using stored passkeys to authenticate.
- Managing the passkeys associated with different accounts.
- Showing the user which service or website a passkey belongs to.
- Supporting secure authentication and user verification.
- Providing clear information about credential availability and recovery limitations.

The implementation should evaluate FIDO2, WebAuthn, and the relevant CTAP specifications.

Compatibility and interoperability must be tested against real services and supported authentication flows.

The project must not assume that every passkey can be exported, copied, synchronized, or recovered. Some credentials may be hardware-bound or otherwise non-exportable.

### 1.2 Password Management

Aurea Wallet should also manage traditional website and application passwords.

Expected functionality includes:

- Securely storing usernames and passwords.
- Associating credentials with the correct website or service.
- Suggesting saved credentials when the user visits a recognized login page.
- Automatically filling login fields after user authorization.
- Supporting manual credential selection.
- Creating and storing strong passwords.
- Allowing users to review, update, and delete saved credentials.
- Showing the user which credentials are associated with a particular service.

Password suggestions and automatic filling must be based on the actual website origin, not merely the visible name of a page.

Credentials must not be exposed to unrelated websites, applications, or page content.

### 1.3 Payment Cards

Aurea Wallet should securely store credit and debit card information.

The intended information may include:

- Cardholder name.
- Card number.
- Expiration date.
- Card network or type, when identifiable.
- A user-defined card name or label.

The Wallet must not retain card verification codes such as CVV/CVC as permanent stored credentials.

The purpose is to make online payments more convenient without requiring users to manually enter their card details every time.

#### Automatic Card Suggestions

When a user visits a website through a compatible browser and reaches a payment form, Aurea Wallet should be able to recognize an eligible payment-card request.

The browser should then present an appropriate Wallet suggestion.

The intended workflow is:

1. The user opens an online store.
2. The user proceeds to checkout.
3. The website displays its payment form.
4. The browser detects that card information is being requested.
5. Aurea Wallet offers the relevant saved card or available cards.
6. The user selects a card.
7. The user confirms the operation when required.
8. The browser fills the appropriate payment fields.

The user must remain in control of whether card information is provided.

Automatic filling must never happen merely because a website displays a form.

The implementation must consider website origin, secure browser communication, user consent, and protection against malicious or misleading forms.

#### Payment Card Storage and Payment Processing

Storing a payment card and performing a payment are different operations.

The initial payment-card functionality is intended to provide secure storage, selection, and controlled autofill.

It does not automatically provide a payment-processing system.

Payment tokenization and contactless payments require additional integrations and must be evaluated separately.

### 1.4 Excluded from the Initial Scope

The initial Aurea Wallet scope does not include:

- Identity documents.
- National identity cards.
- Health cards.
- Loyalty cards.
- Digital identity document issuance.
- General-purpose document storage.

These categories may be evaluated separately in the future.

## 2. Architectural Principles

Aurea Wallet belongs to the AureaOS system layer.

It must be designed as a native AureaOS application, not as a browser extension or a feature belonging exclusively to one browser.

Its architecture should separate:

- Wallet user interface.
- Credential storage.
- Credential management.
- Authentication and authorization.
- Application integration.
- Browser integration.
- Security and recovery mechanisms.

The Wallet must expose controlled interfaces rather than direct access to its internal storage.

Rust is the primary programming language for AureaOS-developed components wherever technically appropriate.

Security-driven exceptions may be considered when a mature, established implementation provides a stronger or more appropriate solution.

The native UI toolkit and internal IPC protocol remain undecided. The security architecture must nevertheless preserve separate responsibilities for the user interface, credential service, authenticator operations, and encrypted storage.

### 2.1 Passkey and Authenticator Boundary

Passkeys must be implemented against established standards, including WebAuthn and the relevant FIDO2/CTAP specifications. AureaOS should provide a platform-authenticator service that can be used by compatible browsers and, where appropriate, native applications.

The platform authenticator and the Wallet user interface are related but distinct responsibilities:

- The authenticator service performs credential creation, user verification checks, and assertion signing.
- The Wallet interface lets the user inspect and manage supported credentials and security settings.
- The secure vault stores protected credential records and key material according to the selected key-management design.
- Browser integration conveys standards-compliant requests and responses while preserving the requesting website's origin and relying-party identity.

An application must not receive unrestricted access to private keys or the vault database. Each sensitive operation must be authorized for the specific requesting application and relying party. User consent and user verification must be enforced by trusted system components, not by an ordinary application window alone.

The exact process layout, daemon names, IPC transport, consent UI integration, and hardware-provider implementation are proposed design work and have not been implemented or finalized. TPM 2.0 is a hardware capability to evaluate, not a guarantee that every device provides a complete isolated authenticator. A software-protected fallback and its security limitations must be documented where hardware-backed protection is unavailable.

## 3. Secure Storage Environment

The most important architectural requirement of Aurea Wallet is the protection of stored credentials.

The project must investigate a dedicated, encrypted, isolated storage environment designed specifically for sensitive information.

This environment is referred to in this document as the **Aurea Secure Vault**.

Aurea Secure Vault is a logical architectural component and a proposed security boundary. It is not yet an implemented hardware enclave or a selected cryptographic technology.

Its purpose is to make stored credentials inaccessible to ordinary applications and unreadable without the required authorization and cryptographic keys.

### 3.1 Security Objectives

The Secure Vault should provide:

- Encryption of sensitive information at rest.
- Protection of encryption keys.
- Separation from ordinary application data.
- Controlled access through authenticated system interfaces.
- Integrity protection for stored credentials.
- Protection against unauthorized extraction.
- Secure handling of credentials during use.
- Controlled locking and unlocking.
- Protection against accidental exposure through logs, temporary files, and diagnostic data.

Reading the vault's files directly must not reveal usable credentials.

Access to the filesystem must not be equivalent to access to the Wallet.

The architecture must also prevent ordinary applications from bypassing the Wallet interface and reading its internal database.

### 3.2 Isolation Boundary

The Secure Vault must be separated logically and, where technically possible, through operating-system security mechanisms from ordinary user applications.

The design should consider:

- Dedicated system-service ownership.
- Restricted filesystem permissions.
- Separate process boundaries.
- Least-privilege execution.
- Authenticated local communication.
- Restricted access to sensitive memory.
- Explicit authorization for credential operations.

The Wallet application must not expose its internal database or encryption keys to the desktop, browser, or unrelated applications.

Applications should interact with the Wallet through a controlled credential interface.

The security boundary must be designed to reduce the consequences of a compromised application or ordinary user process.

### 3.3 Hardware-Backed Protection

AureaOS should investigate available hardware security capabilities.

Potential technologies include:

- TPM 2.0.
- Hardware-backed key storage.
- Platform-specific secure execution capabilities.
- Other hardware security mechanisms supported by the target device.

TPM 2.0 may be used to protect or bind cryptographic keys to a device and its security state, depending on the selected design.

The project must investigate the difference between:

- Hardware-backed key protection.
- Secure key storage.
- Hardware isolation.
- Trusted execution environments.
- A complete hardware-backed credential authenticator.

These capabilities are not interchangeable.

The presence of a TPM does not automatically provide a fully isolated execution environment for the Wallet.

AureaOS must not assume that every supported computer offers identical hardware security capabilities.

Hardware-backed protection should be used when available and technically suitable.

A secure software-based fallback must be designed for supported systems without the required hardware.

The final security guarantees must be documented according to the actual capabilities of each supported configuration.

### 3.4 Software Isolation

Hardware security alone is not sufficient.

The project must also establish a software security boundary around the Wallet.

This should include:

- A dedicated credential-management service.
- Strict process and filesystem permissions.
- Restricted access to sensitive resources.
- Secure local communication.
- Application identity verification.
- Authorization checks for every sensitive operation.
- Memory protection and careful handling of secrets.

The Wallet must not depend exclusively on the graphical application for security enforcement.

Security decisions must be enforced by the trusted system-side components.

## 4. Credential Vault and Key Management

The Credential Vault is responsible for storing sensitive information.

It must provide:

- Encrypted storage.
- Credential separation.
- Integrity protection.
- Controlled access.
- Safe updates and deletion.
- Protection against unauthorized access.

The internal storage format and encryption implementation remain to be designed.

### 4.1 Encryption at Rest

All sensitive credential data must be encrypted before being written to persistent storage.

The design must establish:

- Encryption-key generation.
- Key derivation and management.
- Encryption and decryption boundaries.
- Key protection.
- Authentication requirements.
- Integrity verification.
- Secure updates and deletion.
- Recovery behavior.

The project must select established and reviewed cryptographic implementations.

It must not invent proprietary cryptographic algorithms.

The final cryptographic algorithms, parameters, key hierarchy, and vault format must be selected through a documented security design and reviewed before implementation.

### 4.2 Key Hierarchy

The project should evaluate a layered key-management architecture.

A possible conceptual model includes:

- A primary vault key protecting the credential database.
- Protected key material used to unlock or derive access to the vault key.
- Hardware-backed protection when supported.
- User authentication as part of the unlocking process.

This is a conceptual direction, not a finalized key hierarchy.

The implementation must prevent ordinary applications from obtaining the keys required to decrypt the vault.

The security design must consider key rotation, revocation, device replacement, and recovery.

### 4.3 Memory Protection

Credentials may need to be temporarily available in memory when the user accesses them.

The implementation must minimize this exposure.

The design should consider:

- Keeping secrets in memory for the shortest practical time.
- Avoiding unnecessary copies of sensitive data.
- Restricting access to processes holding secrets.
- Avoiding secrets in logs, crash reports, and diagnostics.
- Clearing sensitive buffers when no longer required, where technically possible.
- Avoiding unnecessary persistence in temporary files.
- Protecting sensitive information during inter-process communication.

The project must recognize that ordinary desktop operating systems cannot guarantee that secrets are completely inaccessible to every privileged process or to an attacker with full control of the running machine.

These limitations must be included in the threat model.

## 5. Authentication and Vault Lifecycle

Access to sensitive credentials must require an authenticated Wallet session.

The design should support:

- Initial vault creation.
- Wallet unlock.
- Wallet lock.
- Manual locking.
- Automatic locking after inactivity.
- Locking when the user session ends.
- Reauthentication for sensitive operations.

### 5.1 Unlocking

The Wallet must verify that the user is authorized before allowing access to protected credentials.

The authentication method and key-unlocking procedure remain to be selected.

The project should evaluate integration with the AureaOS login and session architecture without assuming that desktop login alone is always sufficient to unlock every credential.

### 5.2 Automatic Locking

The Wallet should support configurable security policies.

Possible locking triggers include:

- A period of inactivity.
- User-requested locking.
- Desktop session locking.
- User logout.
- System shutdown.
- Security-related failures.

The exact behavior and timing must be defined through the security design and user-experience requirements.

### 5.3 Sensitive Operations

Some operations may require additional authentication even when the Wallet is already unlocked.

Examples include:

- Revealing a complete card number.
- Exporting eligible credentials.
- Changing security settings.
- Authorizing an especially sensitive operation.
- Performing credential recovery.

The security policy must balance protection and usability.

## 6. Wallet Security Service

A dedicated security boundary should coordinate sensitive operations.

Its responsibilities include:

- Verifying requests from authorized applications.
- Enforcing access policies.
- Managing credential access sessions.
- Coordinating authentication requirements.
- Applying automatic locking rules.
- Preventing unauthorized credential extraction.

The exact service boundary and communication protocol remain to be defined.

The service must be independent of the graphical interface for security enforcement.

A compromised or closed Wallet window must not automatically disable the security controls protecting the stored credentials.

### 6.1 Credential Broker

A Credential Broker is proposed as the controlled interface between applications and the Secure Vault.

The Broker should:

- Receive credential requests.
- Identify the requesting application.
- Validate the requested operation.
- Apply authorization policies.
- Request user confirmation when required.
- Retrieve only the necessary credential information.
- Return the authorized result.
- Record security-relevant events without recording secret values.

The Broker must not expose a general-purpose operation that allows applications to enumerate or export the entire vault.

The final relationship between the Credential Broker and the Wallet Security Service remains to be designed.

## 7. Application Authorization

Applications must not receive permanent or unrestricted permission to access Wallet information.

Authorization should be limited by:

- Application identity.
- Requested operation.
- Credential type.
- Website origin, where applicable.
- User consent.
- Session and authentication state.

The user must be able to understand why an application is requesting information.

The Wallet should provide a way to review and revoke application access where persistent authorization is supported.

### 7.1 Least Privilege

An application should receive only the information necessary for the requested operation.

Examples:

- A browser requesting a matching password should not receive payment-card details.
- A browser requesting a payment card should not receive unrelated passwords.
- An application requesting passkey authentication should not receive the private key as an ordinary exportable value.

The system must distinguish between requesting an authentication operation and requesting raw credential data.

Where possible, cryptographic operations should be performed through the Wallet or an appropriate trusted component without exposing private key material.

### 7.2 User Consent

Credential access must follow a clear authorization policy.

The user should be able to:

- Accept a request.
- Reject a request.
- Select a different credential.
- Review the requesting application.
- Understand which information will be used.

Persistent permissions, where supported, must be explicit, limited, and revocable.

The project must not silently grant broad credential access to newly installed applications.

## 8. Browser Integration

Browser integration is one of the most important parts of Aurea Wallet.

AureaOS should provide a consistent credential experience even when the user installs a third-party browser.

This is particularly important for Roma 1.0 because web browsers are initially expected to be provided by established third-party projects through Aurea Store.

### 8.1 Integration Goals

Compatible browsers should be able to:

- Request password suggestions.
- Request passkey authentication.
- Request payment-card suggestions.
- Trigger authorized credential filling.
- Display Wallet authentication and consent prompts.
- Report the result of a credential operation.

The user experience should remain consistent across supported browsers.

### 8.2 Integration Architecture

The project must evaluate suitable integration mechanisms, including:

- Browser extension interfaces.
- Native messaging integration.
- A system-level credential service.
- Standard authentication interfaces where available.

A browser integration component may communicate with Aurea Wallet through an authenticated system interface.

The browser must not receive direct access to the Wallet database or encryption keys.

The final implementation must account for browser-specific integration requirements, Flatpak sandboxing, application permissions, and communication boundaries.

The chosen solution must work with the actual browser distribution and sandbox configuration used by AureaOS.

The project must not assume that every browser supports the same native integration capabilities.

### 8.3 Website Origin Verification

Credential suggestions must be associated with the correct website origin.

The system must distinguish between:

- A legitimate website requesting its own credentials.
- A different website attempting to imitate that service.
- An unrelated page attempting to request stored information.

Matching only a page title, company name, or visible text is insufficient.

The integration must use trustworthy origin information provided by the browser.

The project must evaluate how origin information is authenticated across the browser-to-Wallet communication boundary.

### 8.4 Password Autofill

When a user visits a recognized login page, the browser should be able to request matching credentials.

The intended workflow is:

1. The browser identifies a login context.
2. The browser requests a matching credential.
3. The Credential Broker validates the requesting application and website origin.
4. Aurea Wallet identifies matching saved credentials.
5. The user selects a credential or accepts an authorized suggestion.
6. The browser fills the relevant fields.
7. The operation completes without exposing unrelated credentials.

The user should also be able to select credentials manually from the Wallet interface.

### 8.5 Payment Card Autofill

When a user reaches an online payment form, a compatible browser should be able to request a saved payment card.

The intended workflow is:

1. The browser detects a supported payment form.
2. The browser requests payment-card suggestions.
3. The Credential Broker validates the request.
4. Aurea Wallet presents available matching cards.
5. The user selects a card.
6. The user authorizes the operation.
7. The browser fills the appropriate fields.

The Wallet must not send card details to the browser before the required authorization has been granted.

The implementation must consider misleading forms, malicious pages, and incorrect field identification.

### 8.6 Passkey Integration

Aurea Wallet should integrate with compatible browsers and websites for passkey creation and authentication.

The implementation must use established standards and trusted browser authentication interfaces.

Where the authentication protocol allows it, private key operations should remain within the trusted credential component.

The browser should receive the protocol result required to complete authentication, not unrestricted access to the underlying private key.

## 9. Passkey Standards and Interoperability

Passkeys must be implemented using established standards wherever appropriate.

The project should evaluate:

- FIDO2.
- WebAuthn.
- CTAP.
- Platform authenticator integration.
- Browser and system authentication interfaces.

The objective is to provide interoperable passkey functionality without inventing a proprietary authentication protocol.

The implementation must distinguish between:

- Credentials stored and managed by Aurea Wallet.
- Credentials protected by hardware.
- Credentials that cannot be exported.
- Credentials managed by external authentication providers.

Synchronization, portability, and recovery behavior must be documented according to the actual capabilities of the selected implementation.

The Wallet must not promise universal passkey synchronization or recovery.

## 10. Password Security

Passwords must be stored separately from ordinary application settings and non-sensitive user data.

The implementation should provide:

- Secure password generation.
- Secure credential storage.
- Website and service association.
- Duplicate credential handling.
- Credential replacement.
- Password deletion.
- Secure autofill.
- User-controlled credential selection.

The Wallet should avoid exposing passwords through logs, diagnostic messages, crash reports, or ordinary application interfaces.

Password values must not be included in telemetry.

The project should also evaluate warnings for weak or reused passwords, provided that such checks can be implemented without unnecessarily exposing credentials.

## 11. Payment Security and Transaction Boundaries

Storing a payment card and performing a payment are different operations.

Aurea Wallet's initial payment-card functionality is intended to provide secure storage, selection, and controlled autofill.

It does not automatically provide a payment-processing system.

### 11.1 Card Storage and Autofill

The Wallet may store card information and provide it to a compatible browser after authorization.

The browser remains responsible for communicating with the website's payment form.

Aurea Wallet must not claim that storing a card makes it possible to perform every type of online payment.

### 11.2 Payment Tokenization

Payment tokenization is a separate capability.

It may replace a card number with a payment token issued through an eligible payment network or service.

Supporting tokenized payments requires evaluation of:

- Payment network requirements.
- Issuer support.
- Token service provider integration.
- Certification and compliance obligations.
- Device and platform security requirements.

These integrations are not assumed to be available in Roma 1.0.

### 11.3 Contactless and NFC Payments

NFC contactless payments are outside the initial confirmed scope.

Supporting contactless payments would require additional hardware and software capabilities, payment-network and issuer participation, token provisioning, and applicable security and certification requirements.

Aurea Wallet must not advertise tap-to-pay functionality until these requirements have been met and the implementation has been validated.

### 11.4 Payment Data Protection

The Wallet must minimize the payment information it stores and exposes.

Card verification codes such as CVV/CVC must not be retained as permanent stored card data.

Payment security and applicable PCI DSS requirements must be evaluated before the payment-card storage and integration design is finalized.

The existence of encrypted storage does not, by itself, establish PCI DSS compliance.

## 12. AureaOS Desktop Integration

Aurea Wallet is a native part of the AureaOS product experience.

It should not feel like an unrelated application installed alongside the desktop.

Its interface must follow AureaOS Human Interfaces and the shared design system.

### 12.1 Desktop Experience

The application should provide:

- A clear overview of saved credentials.
- Separate sections for passkeys, passwords, and payment cards.
- Search and filtering.
- Credential details.
- Security settings.
- Application authorization management.
- Lock and unlock controls.
- Clear information about security status.

### 12.2 System Prompts

Aurea Wallet should be able to present system-integrated prompts when an application requests a credential.

Examples include:

- Password suggestions.
- Payment-card selection.
- Passkey authentication.
- Additional authentication requests.
- Authorization and revocation messages.

These prompts should be recognizable as AureaOS system interactions.

Their visual design and behavior must remain consistent with the desktop.

### 12.3 Accessibility

The interface must support:

- Keyboard navigation.
- Visible focus indicators.
- Screen readers.
- Clear and understandable security messages.
- Accessible contrast.
- Scalable text.
- Light and dark themes.

The native UI toolkit has not yet been selected.

## 13. Backup and Recovery

Aurea Wallet must have a clearly defined recovery strategy.

Its integration with Aurea Time should be evaluated as part of the wider AureaOS backup and restore architecture.

Wallet data must never be copied into an ordinary unprotected backup.

### 13.1 Backup Requirements

The design should consider:

- Encrypted backup data.
- Protection of backup encryption keys.
- Integrity verification.
- User authorization.
- Restoration to a supported system.
- Handling of credentials that cannot be exported.

The backup process must not silently expose the contents of the Secure Vault to ordinary backup destinations.

### 13.2 Recovery Limitations

Not every credential can necessarily be restored to another device.

Hardware-bound passkeys and non-exportable keys may require a new registration or recovery process with the original service.

The Wallet must clearly explain these limitations.

A successful restoration of the Wallet database must not be presented as proof that every stored credential is usable again.

The backup format, key recovery process, and compatibility rules remain to be designed.

## 14. Privacy

Aurea Wallet must collect and expose as little information as possible.

The initial design should avoid unnecessary external services and avoid transmitting credential data to AureaOS infrastructure.

The project should establish clear rules for:

- Credential metadata.
- Application access history.
- Security event logging.
- Diagnostic information.
- Backup metadata.
- Data retention and deletion.

Security logs must never contain passwords, complete card numbers, private keys, or other secret credential values.

The Wallet must not use credential contents for analytics or advertising.

## 15. Threat Model

The security architecture must be developed around an explicit threat model.

The project should consider at least the following threats:

### 15.1 Offline Storage Theft

An attacker obtains access to the physical storage device or copies the Wallet database.

Expected protection:

- Credential data must remain encrypted.
- Encryption keys must not be stored alongside the vault in a directly usable form.
- Integrity checks must detect unauthorized modification.

### 15.2 Unauthorized Local Application

An application attempts to access Wallet files, request credentials, or communicate with the security service without authorization.

Expected protection:

- Restricted filesystem permissions.
- Application identity verification.
- Authenticated communication.
- Least-privilege access.
- Explicit authorization policies.

### 15.3 Malicious Website

A website attempts to obtain credentials belonging to another service or imitate a legitimate payment form.

Expected protection:

- Reliable website-origin verification.
- Browser-mediated integration.
- User-visible credential selection.
- Restricted credential delivery.
- No trust based solely on visible page content.

### 15.4 Compromised Browser

A browser or browser integration component becomes compromised.

Expected protection:

- No direct access to the vault.
- No unrestricted credential enumeration.
- Operation-specific authorization.
- Minimal credential disclosure.
- Clearly defined trust boundaries.

The project must recognize that once a credential has been delivered to a compromised application, that application may be able to expose it.

This risk must be reduced through careful authorization and limited exposure.

### 15.5 Malware or Privileged System Compromise

An attacker gains elevated privileges or control over the running operating system.

Expected protection:

- Hardware-backed key protection where supported.
- Restricted service boundaries.
- Secure boot and system-integrity capabilities where available.
- Reduced exposure of secrets in memory.
- Documented limitations and recovery behavior.

No design should claim absolute protection against an attacker who has complete control of the running machine or the relevant hardware and firmware.

### 15.6 User Session Exposure

An unauthorized person gains access to an unlocked user session.

Expected protection:

- Automatic locking.
- User authentication.
- Reauthentication for sensitive operations.
- Restricted credential visibility.
- Clear session lifecycle rules.

### 15.7 Backup Theft

An attacker obtains a Wallet backup.

Expected protection:

- Encrypted backup data.
- Independent protection of backup keys.
- Integrity verification.
- A recovery process that does not expose credentials.

The final threat model must be reviewed and expanded before the Wallet is considered ready for release.

## 16. Security Validation and Testing

Security must be validated independently from ordinary functional testing.

The project should establish a dedicated test strategy covering:

- Vault encryption and decryption.
- Key lifecycle and access control.
- Unauthorized process access.
- Application identity verification.
- Credential Broker authorization.
- Browser-origin verification.
- Password autofill isolation.
- Payment-card autofill isolation.
- Passkey authentication flows.
- Lock and unlock behavior.
- Recovery and backup integrity.
- Failure handling.
- Sensitive information in logs and diagnostics.

The project should also evaluate independent security review and penetration testing before the Roma 1.0 release.

A successful build or passing unit tests must not be treated as proof of security.

Security-sensitive components require documented review, integration testing, and validation against the threat model.

## 17. Technology Evaluation

The implementation must reuse mature and well-maintained security technologies wherever appropriate.

The project should evaluate:

- Linux key-management facilities.
- TPM 2.0 integration.
- Hardware-backed key storage.
- Established cryptographic libraries.
- Secure inter-process communication.
- Browser integration mechanisms.
- FIDO2 and WebAuthn implementations.
- Platform authentication interfaces.

Linux Secret Service may be investigated as a compatibility reference or integration interface.

Its evaluation must not automatically determine the internal architecture or identity of Aurea Wallet.

Aurea Wallet remains an AureaOS-owned product with its own security requirements and user experience.

The project must not select technologies only because they are convenient to integrate.

Every security-critical dependency must be evaluated for:

- Maintenance and upstream support.
- Security history.
- Licensing.
- Compatibility with Debian 13.
- Integration with Rust.
- Hardware requirements.
- Long-term sustainability.

No cryptographic algorithm, vault format, or final hardware security architecture has been selected at this stage.

## 18. Development Principles

Aurea Wallet should follow the wider AureaOS architecture principles.

The implementation must remain modular, testable, and maintainable.

Responsibilities should be separated between:

- Wallet user interface.
- Credential management.
- Secure storage.
- Key management.
- Authentication.
- Authorization.
- Credential Broker.
- Browser integration.
- Backup and recovery.

The project follows the guiding principle:

**One file, one responsibility.**

This does not mean creating unnecessary files or fragmenting components into tiny modules.

The objective is a clear architecture with well-defined interfaces and security boundaries.

Rust is the primary language for first-party implementation wherever technically appropriate.

## 19. Implementation Stages

The development process should proceed through explicit stages.

### Stage 1 — Security and Architecture Design

- Define the threat model.
- Define the Secure Vault security boundary.
- Evaluate hardware-backed protection.
- Define the key-management requirements.
- Evaluate cryptographic libraries and storage formats.
- Define the Credential Broker responsibilities.
- Define the application authorization model.
- Evaluate passkey standards and implementation options.

No credential storage implementation should begin before the security architecture is sufficiently defined.

### Stage 2 — Secure Storage Foundation

- Implement the initial vault architecture.
- Establish encryption and integrity protection.
- Implement key-management foundations.
- Establish secure service boundaries.
- Implement locking and unlocking.
- Add security-focused automated tests.

### Stage 3 — Core Wallet

- Implement the native Wallet interface.
- Add password management.
- Add payment-card storage.
- Add credential search and management.
- Implement user authorization prompts.
- Add security settings and automatic locking.

### Stage 4 — Passkey Support

- Integrate the selected FIDO2/WebAuthn implementation.
- Implement passkey registration.
- Implement passkey authentication.
- Validate compatibility with supported browsers and services.
- Define credential recovery limitations.

### Stage 5 — Browser Integration

- Implement the browser integration interface.
- Integrate password suggestions and autofill.
- Integrate payment-card suggestions and autofill.
- Integrate passkey authentication.
- Validate origin verification.
- Test supported third-party browsers and their sandbox configurations.

### Stage 6 — Backup, Recovery and Security Validation

- Integrate with Aurea Time.
- Implement encrypted backup and recovery.
- Validate key recovery behavior.
- Complete security testing.
- Review the threat model.
- Perform independent security assessment where feasible.

### Stage 7 — Roma 1.0 Release Readiness

- Complete the required Wallet functionality.
- Validate supported hardware configurations.
- Validate browser integration.
- Complete security and recovery documentation.
- Resolve release-blocking security issues.
- Confirm that the Wallet meets its defined security requirements.

Aurea Wallet is a mandatory Roma 1.0 release component.

The operating system must not be released with the Wallet missing, non-functional, or lacking the security validation required for its intended use.

## 20. Current Project Status

Aurea Wallet is currently in the planning and architectural design phase.

The following points describe the established product direction:

- Aurea Wallet is a native AureaOS system application.
- It is a mandatory component of the Roma 1.0 release.
- Its initial scope includes passkeys, passwords, and payment cards.
- Browser integration and controlled autofill are core requirements.
- Secure, encrypted credential storage is a fundamental requirement.
- A dedicated Secure Vault architecture must be investigated.
- Hardware-backed protection should be evaluated.
- The Credential Broker and application authorization model must be designed.
- Backup and recovery must be integrated into the wider AureaOS architecture.

The following items remain unimplemented or undecided:

- Secure Vault implementation.
- Cryptographic algorithms and vault format.
- Key-management architecture.
- Hardware-backed security integration.
- Authentication and unlocking mechanisms.
- Credential Broker implementation.
- Browser integration mechanism.
- Passkey implementation.
- Password and payment-card storage.
- Native graphical interface.
- Backup and recovery implementation.
- Security testing and independent review.

No implementation or security certification is currently claimed.

## 21. Guiding Principle

Aurea Wallet must make secure credential management a natural part of using AureaOS.

Users should be able to authenticate, manage passwords, and complete online payment forms without repeatedly entering the same information.

Convenience must never come at the expense of credential security.

The Secure Vault must protect stored information, the Credential Broker must control access, and the user must remain in charge of every sensitive operation.

Aurea Wallet is not an optional extra.

**AureaOS Roma 1.0 will not ship without Aurea Wallet.**
