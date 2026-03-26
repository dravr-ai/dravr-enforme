# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.0.x   | Yes       |

## Reporting a Vulnerability

If you discover a security vulnerability in dravr-enforme, please report it responsibly:

1. **Do not** open a public GitHub issue
2. Email **security@dravr.ai** with a description of the vulnerability
3. Include steps to reproduce, if possible
4. You will receive an acknowledgment within 48 hours

We will work with you to understand the issue and coordinate a fix before any public disclosure.

## Security Model

dravr-enforme orchestrates health data synchronization between provider APIs and local stores. The security boundary is:

- **HMAC-SHA256 webhook validation** — all incoming webhooks are cryptographically verified
- **No secrets in core** — API keys and tokens are handled by CredentialStore implementations
- **Constant-time signature comparison** — prevents timing attacks on webhook validation
- **Rate limiting** — token bucket per provider prevents abuse
- **Soft delete by default** — data is never permanently lost without explicit configuration
