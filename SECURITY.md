# Security Policy

## Reporting a vulnerability

Please do not open a public issue for security problems. Use GitHub's private vulnerability reporting: open the repository's **Security** tab and choose **Report a vulnerability**.

Include the affected version or commit, steps to reproduce, and the impact you expect. Do not include real user data.

## What to expect

- We aim to acknowledge a report within 7 days.
- We will keep you updated while we investigate and fix it, and credit you in the fix unless you prefer otherwise.
- Please give us reasonable time to release a fix before you disclose publicly.

## Supported versions

Only the latest tagged release is supported. Once v1.0.0 is tagged, reports against it (or any later release) are in scope; until the first tag exists, only the latest commit on `main` is.

## Scope

In scope: authentication and sessions, authorization and data isolation between users, upload handling, and the Docker image. Out of scope: problems that need a malicious server administrator, or a deployment that disables the documented protections (for example `SESSION_COOKIE_SECURE=false` on a public host).
