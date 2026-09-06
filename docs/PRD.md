# Product Requirements Document: MediOSS Wellness App

## Executive Summary

MediOSS is a self-hosted wellness application designed to support human mood regulation through four integrated pillars: authentication and user management, a meditation library, white noise player, and mood tracking. The application prioritizes user privacy, data sovereignty, and accessibility while providing practical, self-directed tools for mental wellness.

### Key Differentiators

- **Self-hosted architecture** - Complete user control over data
- **Integrated wellness ecosystem** - Three complementary mood regulation tools (meditation, white noise, mood tracking) behind a single account
- **Privacy-first design** - No third-party tracking or data sharing
- **Offline functionality** - Core features work without internet connectivity

---

## Product Vision

> To provide individuals with a private, self-hosted sanctuary for mental wellness through accessible meditation, ambient soundscapes, and mood tracking.

### Success Metrics

Because MediOSS collects no telemetry (see Non-Goals), success is measured only through what is verifiable locally or at the project level:

- **Privacy Trust:** 100% data locality (no external service dependencies), verifiable by network inspection of a running instance
- **Accessibility:** WCAG 2.1 AA compliance for all interfaces, verified by audit
- **Deployability:** A first-time operator reaches a working instance from the docs alone in under 5 minutes
- **Instance Insight:** Operators can see aggregate, non-content usage of their own instance locally (active user counts, storage use); admins never see another user's content, mood data, or journal entries, and nothing is reported outward
- **Project Health:** Community adoption signals (stars, forks, deployment reports, contributions) on the public repository

---

## Target Audience

### Primary Users

1. **Privacy-Conscious Individuals** (25-45 years)
   - Tech-savvy users concerned about data privacy
   - Prefer self-hosted solutions over commercial wellness apps
   - Value mental health tools without subscription costs

2. **Wellness Enthusiasts** (20-50 years)
   - Regular meditation practitioners
   - Interested in mood tracking and emotional awareness
   - Seek comprehensive wellness solutions

### Secondary Users

1. **Mental Health Professionals**
   - Therapists recommending a private, self-hosted tool for clients' personal use outside of treatment
   - Need recommendations that respect client privacy (no vendor accounts, no data sharing)

2. **Digital Minimalists**
   - Prefer local applications over cloud services
   - Value simplicity and focused functionality

---

## Core Functional Pillars

### 1. Authentication & User Management

#### User Stories

- As a user, I want to create an account so I can have a personal wellness space
- As a user, I want to log in securely so my data is protected
- As an admin, I want to manage user accounts so I can control access to the instance

#### Functional Requirements

- **Registration & Login**
  - Email/password registration with validation
  - Secure login with session management
  - Admin-driven password reset (admin issues a temporary password; no email infrastructure required)
  - Optional: SMTP-based password reset flow, only when an operator explicitly configures an outgoing mail server
- **User Profiles**
  - Display name and avatar
  - Personal preferences (theme, notification settings)
  - Account settings (change password, update email)
- **Multi-User Support**
  - Instance-level user limit configuration
  - Role-based access: admin, regular user
  - Admin dashboard for user management (list, disable, remove users)
  - Per-user data isolation (each user sees only their own content)
- **Account & Data Deletion**
  - User-initiated account deletion with explicit confirmation
  - Deletion removes all database records and all uploaded media files for that user (`media/{user_id}/` is wiped)
  - Admin can delete any non-admin account with the same guarantees
  - Deletion is immediate and irreversible; no soft-delete retention of personal data
- **Security**
  - Password hashing with bcrypt/scrypt
  - Session token management with expiry
  - Rate limiting on login attempts
  - Optional: two-factor authentication (TOTP)
- **Operator Recovery**
  - CLI recovery command runnable by whoever has host access (for example `medioss admin reset-password <email>`), covering the case where the only admin loses their password on a single-user instance
  - Works offline against the local database; requires no running web server and no email infrastructure

#### Technical Requirements

- Secure password storage (never plaintext)
- JWT or session-based authentication
- CSRF protection on state-changing requests
- Secure HTTP-only cookies for session tokens
- Configurable session timeout

#### User Onboarding Workflow

1. Admin deploys instance and creates admin account
2. New user navigates to instance URL
3. User registers with email and password
4. User logs in and is guided through initial preferences setup

---

### 2. Meditation Library

#### User Stories

- As a user, I want to upload my own meditation audio files so I can use content that resonates with me
- As a user, I want to organize my meditations with categories and tags so I can find them easily
- As a user, I want to track my meditation streak so I can maintain consistent practice

#### Functional Requirements

- **Content Management**
  - User upload of audio files (MP3, WAV, OGG)
  - Manual categorization by type (mindfulness, breathing, body scan, loving-kindness)
  - Custom tagging system for organization
  - Duration auto-detection from audio files
  - Search and filter by title, category, tags, duration
- **Playback Features**
  - Play/pause/seek controls
  - Resume from last position (a partially played meditation continues where it left off, per user per meditation)
  - Background playback support
  - Sleep timer with fade-out option
  - Playback speed adjustment (0.5x - 2.0x)
  - Auto-play next in category option
- **Progress Tracking**
  - Session completion tracking
  - Streak counting (consecutive days)
  - Total meditation time statistics
  - Calendar view of practice history
  - Playback history for each meditation

#### Technical Requirements

- Audio file upload with size limits (max 100MB per file)
- Audio metadata extraction (duration, format, bitrate)
- Efficient audio streaming and buffering
- Support for media session API (hardware controls)
- Responsive design for mobile and desktop
- Offline access to downloaded content

#### User Upload Workflow

1. User uploads audio file through web interface
2. System extracts metadata and auto-detects duration
3. User adds title, description, category, and tags
4. File is stored locally and organized in database
5. Meditation appears in user's personal library
6. User can play, track progress, and manage their content

---

### 3. White Noise Player

#### User Stories

- As a user, I want to mix multiple ambient sounds so I can create my ideal soundscape
- As a user, I want to save custom sound mixes so I can quickly access preferred combinations
- As a user, I want to set fade-out timers so I can fall asleep to ambient sounds

#### Functional Requirements

- **Sound Library**
  - Nature sounds (rain, ocean, forest, birds)
  - Mechanical sounds (fan, white noise, brown noise)
  - Ambient textures (cafe, fireplace, wind)
  - Support for custom user-uploaded sounds
- **Mixing Capabilities**
  - Individual volume control per sound
  - Support for 3-5 simultaneous sounds
  - Visual mixing interface
  - Volume balance presets
- **Playback Controls**
  - Fade-in/fade-out timers (5min - 120min)
  - Background playback
  - Lock screen controls
  - Emergency stop functionality

#### Technical Requirements

- Real-time audio mixing
- Memory-efficient sound buffer management
- Support for continuous playback (6+ hours)
- Low-latency volume adjustments

---

### 4. Mood Tracking

#### User Stories

- As a user, I want to log my mood so I can build emotional self-awareness
- As a user, I want to add context and journal entries with my mood so I can understand what influences it
- As a user, I want to view my mood history over time so I can identify patterns and trends

#### Functional Requirements

- **Mood Logging**
  - Quick mood entry with a 1-10 scale
  - Optional emotion tags (happy, anxious, calm, stressed, etc.)
  - Optional journal text for additional context
  - Optional activity tagging (meditation, exercise, social, work)
  - Timestamped entries with timezone awareness
- **Reminders**
  - Optional daily mood check-in reminder at a user-chosen time
  - Delivered as in-app/PWA local notifications only; no external push services
- **Mood History**
  - Calendar view of mood entries
  - List view with search and filter by date, mood score, emotions
  - Edit and delete existing entries
- **Analytics & Insights**
  - Mood trend charts (daily, weekly, monthly averages)
  - Emotion frequency breakdown
  - Correlation view between activities and mood scores
  - Streak tracking for consistent mood logging
- **Data Management**
  - Data export in standard formats (CSV, JSON)
  - Data import of previously exported files, enabling instance migration and restore (export/import round-trips losslessly)
  - Per-user data isolation

#### Technical Requirements

- Efficient querying for date-range mood history
- Responsive charts and visualizations on mobile and desktop
- Offline mood entry with sync on reconnect

#### Mood Logging Workflow

1. User opens mood tracker or is prompted after a meditation/white noise session
2. User selects mood score on a 1-10 scale
3. User optionally adds emotion tags, journal text, and activity tags
4. Entry is saved with timestamp
5. User can view updated mood history and trends

---

## Non-Functional Requirements

### Performance

- Meditation playback starts within 2 seconds
- Sound mixing supports real-time adjustments (<100ms latency)
- Mood tracker loads within 1 second for up to 1000 entries
- Application startup time <3 seconds

### Security

- All data stored locally by default
- No third-party analytics or tracking
- Regular security updates for dependencies

### Accessibility

- WCAG 2.1 AA compliance
- Keyboard navigation for all features
- Screen reader compatibility
- High contrast mode support
- Font size customization

### Compatibility

- **Browsers:** Chrome 90+, Firefox 88+, Safari 14+, Edge 90+
- **Mobile:** iOS 14+, Android 10+ (via PWA)
- **Desktop:** Windows 10+, macOS 11+, Linux (major distributions)

### Reliability

- Graceful handling of audio interruptions
- Automatic data backup prompts
- Recovery mode for corrupted data
- Offline functionality for core features

---

## Non-Goals

Explicit boundaries that protect the privacy-first, self-hosted vision. These are decisions, not omissions:

- **No third-party authentication.** No social login (Google, GitHub, etc.); identity never leaves the instance.
- **No cloud sync or vendor accounts.** There is no hosted MediOSS service; data moves between instances only via user-controlled export/import.
- **No third-party integrations for core features.** No Fitbit, Apple Health, or similar cloud-API integrations; nothing in the product depends on an external service being reachable.
- **No external push notification services.** Reminders are in-app/PWA local notifications only.
- **No telemetry, analytics, or crash reporting** that leaves the instance.
- **No social feed or cross-user sharing.** Wellness data is private to each user; there is no content feed, follower model, or public profile.
- **No AI-generated content or recommendations** in scope for this product.
- **Not a medical device.** See the disclaimer below.

### Medical Disclaimer

MediOSS is a general wellness tool. It is not a medical device, does not diagnose or treat any condition, and is not a substitute for professional mental health care. The UI must present mood data as self-reflection, never as clinical assessment, and the documentation must carry this disclaimer.

---

## Technical Architecture

### Self-Hosted Deployment Options

Three deployment sizes are supported, distinguished by expected concurrent users and network exposure. See [Deployment Guide: Resource Scaling](./DEPLOYMENT.md#resource-scaling) for the authoritative RAM, storage, and CPU figures per size — the Rust backend keeps actual memory use well under typical container limits.

| Option | Recommended for | Networking |
| --- | --- | --- |
| Single-User Docker | Personal use | Local access only (localhost) |
| Family/Home Server | 2-5 users | Local network + optional remote access |
| Community Instance | 5-20 users | Remote access with authentication |

### Technology Stack Recommendations

#### Frontend

- **Framework:** Astro with TypeScript for type safety
- **Interactivity:** Svelte 5 islands (via `@astrojs/svelte`) with shared runes/nanostores for cross-island state
- **Styling:** Tailwind CSS
- **UI Components:** shadcn-svelte (copy-in, accessible component primitives)
- **Audio:** Howler.js or Web Audio API

#### Backend

- **Runtime:** Rust with Axum web framework (performance and safety)
- **Database:** SQLite with SeaORM (type-safe query builder and migrations)
- **Authentication:** JWT-based authentication with role-based access control
- **Async Runtime:** Tokio for efficient concurrent operations

#### Infrastructure

- **Containerization:** Docker with multi-stage builds
- **Reverse Proxy:** Nginx or Caddy (for HTTPS)
- **Backup:** Rsync or restic for automated backups

### Rust Backend Advantages

**Performance & Safety:**

- Memory-safe with compile-time guarantees
- Zero-cost abstractions for high performance
- Fearless concurrency for parallel audio processing
- Single binary deployment with minimal dependencies

**Development Benefits:**

- SeaORM provides a type-safe query builder with generated entities
- Strong type system prevents runtime errors
- Excellent error handling with Result types
- Modern async/await syntax with Tokio

For layer boundaries, crate layout, and the dependency rule, see [Technical Architecture](./ARCHITECTURE.md).

---

## User Experience Design

### Design Principles

1. **Calm & Minimalist Interface**
   - Soothing color palette (blues, greens, soft neutrals)
   - Generous white space
   - Limited cognitive load per screen
   - Smooth animations (no jarring transitions)

2. **Progressive Disclosure**
   - Core features front and center
   - Advanced features behind clear menus
   - Contextual help and tooltips
   - Empty states with helpful guidance

3. **Responsive & Adaptive**
   - Mobile-first design approach
   - Touch-friendly targets (min 44x44px)
   - Adaptive layouts for tablet/desktop
   - Landscape orientation support

### Key User Flows

#### First-Time User Onboarding Flow

1. Deploy instance → Admin creates first account
2. Admin configures instance settings (user limit)
3. New users can register with email/password
4. User completes profile setup (display name, preferences)
5. User guided through initial feature tour

#### Morning Routine Flow

1. Open app → Quick mood check-in
2. Select meditation (filtered by "morning" tag)
3. Start 10-minute session with ambient background
4. Complete session → Update mood (optional)
5. View daily streak motivation

#### Evening Wind-Down Flow

1. Open app → Evening mood log
2. Launch white noise mixer
3. Select "sleep" preset or customize
4. Set 30-minute fade timer
5. Exit app with confidence sounds will fade smoothly

---

## Content Strategy

All content in MediOSS is user-owned: users upload their own meditation audio and white noise loops. Instance operators may optionally provide seed content to make a fresh instance usable on day one, but no bundled library is required for any feature to work.

### Optional Seed Meditation Content

- **Suggested starter set:** 10 sessions (5-15 min each)
- **Categories:** 2 sessions per category
- **Source:** Public domain or CC-licensed content
- **Status:** Optional — the library is fully functional with user uploads alone

### Optional Seed White Noise Content

- **Suggested starter set:** 15 high-quality loops
- **Categories:** Nature (5), Mechanical (5), Ambient (5)
- **Quality:** 44.1kHz, stereo, seamless loops
- **Status:** Optional — users can upload custom sounds

### Mood Tracking Seed Data

- **Emotion Tags**: 20-30 predefined emotions (happy, anxious, calm, stressed, energized, tired, focused, etc.)
- **Activity Tags**: Common activities (meditation, exercise, social, work, sleep, reading)
- **Future**: User-customizable emotion and activity tags

---

## Security & Privacy

### Data Protection

- **Local-First:** All data stored on user's infrastructure
- **No Telemetry:** Zero analytics or usage tracking
- **Backup:** User-controlled backup options

### Compliance Considerations

- GDPR-compatible data handling (data locality, user-initiated export and deletion)
- Clinical use is out of scope: MediOSS is a personal wellness tool, not a system for processing patient data (see Non-Goals: Medical Disclaimer)
- Clear data retention policies
- Easy data export/deletion

---

## Implementation Phases

### Phase 1: MVP (8-12 weeks)

- [ ] Authentication and user management (admin account, registration, login)
- [ ] Meditation library with audio upload and playback
- [ ] Simple white noise player (3 simultaneous sounds)
- [ ] Mood logging with basic tracking
- [ ] Single-user self-hosted deployment
- [ ] Mobile-responsive interface

### Phase 2: Enhanced Features (8 weeks)

- [ ] Full meditation categorization and search
- [ ] Advanced white noise mixing
- [ ] Mood analytics and visualizations
- [ ] Multi-user support (admin dashboard for user management)
- [ ] Account and data deletion flows
- [ ] Mood data export and import
- [ ] Offline mode improvements

### Phase 3: Polish & Expansion (8 weeks)

- [ ] Advanced analytics and insights
- [ ] Daily reminder notifications (local only)
- [ ] Two-factor authentication (TOTP)
- [ ] Accessibility audit and improvements

---

## Success Criteria

### Technical Metrics

- ✅ Application deploys successfully via Docker in <5 minutes
- ✅ Audio playback works smoothly on 95% of target devices
- ✅ Database handles 10,000+ mood entries without performance degradation
- ✅ Backup/restore functionality tested and documented

### User Experience Metrics

- ✅ New user can complete first mood log in <30 seconds
- ✅ Meditation playback starts within 2 clicks
- ✅ Support for 3+ concurrent audio streams
- ✅ UI responsive on mobile, tablet, and desktop

### Business Impact Metrics

- ✅ Zero external service dependencies for core functionality
- ✅ Clear documentation for self-hosting
- ✅ Active community engagement around the open-source repository
- ✅ Low barrier to entry for non-technical users

---

## Open Questions & Risks

### Technical Risks

- **Audio Latency:** Cross-browser audio mixing consistency
- **Storage Management:** Large media files for limited storage scenarios
- **Mobile PWA Limitations:** Background playback restrictions on iOS

### Product Risks

- **Content Sourcing:** Users must supply their own audio to get started
- **User Adoption:** Self-hosting complexity barrier for non-technical users
- **Maintenance:** Long-term support and update commitment

### Open Questions

- **Diary encryption at rest:** Should journal text be encrypted at rest (for example SQLCipher or per-field encryption)? Deferred until the threat model justifies the key-management complexity; disk-level encryption of the host is the current recommendation.

### Mitigation Strategies

- Progressive enhancement for audio features
- Compression and streaming for media management
- Comprehensive deployment documentation
- Documented sources of public domain and CC-licensed audio for new instances

---

## Appendix

### Competitor Analysis

| Feature       | Headspace | Calm | MediOSS |
| ------------- | --------- | ---- | ------- |
| Self-hosted   | ❌        | ❌   | ✅      |
| Meditation    | ✅        | ✅   | ✅      |
| White Noise   | ❌        | ✅   | ✅      |
| Mood Tracking | ❌        | ❌   | ✅      |
| Privacy Focus | ⚠️        | ⚠️   | ✅      |
| One-time Cost | ❌        | ❌   | ✅      |
