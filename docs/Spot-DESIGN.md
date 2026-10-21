# Spot — Mobile Design System (Authoritative V1)

**Updated:** 21 October 2026  
**Scope:** Spot mobile application — iOS / Android  
**Recommended implementation:** Expo / React Native  
**Status:** Product + design implementation reference

> **Source of truth for Spot mobile UX.** This document controls navigation, hierarchy, layout, interaction patterns, visual language, social behavior, workout-session UX, accessibility, state design and design QA. Where a generated mockup conflicts with this document, this document wins unless a deliberate design-system change is recorded.

---

## 1. Product definition

Spot is a **guided social fitness app** for people who are starting the gym, returning after time away, or exploring a new training program.

Spot should feel less like a spreadsheet or bodybuilding tracker and more like **the knowledgeable friend beside you at the gym** — telling you what to do, how to do it, how to adjust when reality gets in the way, and helping you train with other people when you want to.

### Core promise

> **You always know what to do next — and you do not have to do it alone.**

### Brand line

> **Never train lost.**

### Product pillars

1. **Guide** — tell the user what to do and how to do it well.
2. **Progress** — make improvement visible without turning every screen into analytics.
3. **Connect** — make training with friends and nearby people natural.
4. **Belong** — make beginners feel that the gym is a place they understand and can participate in.

### Primary audience

- roughly 18–35
- beginners and early-stage gym users
- inconsistent gym users trying to build a habit
- people changing programs or returning after a break
- users who do not want or cannot justify a full-time personal trainer
- users who are comfortable using their phone during a workout

Spot is **not** initially designed around competitive bodybuilding, powerlifting meet prep, elite endurance coaching or highly technical sports programming.

---

## 2. Product principles and boundaries

### 2.1 One obvious next action

Every major screen should answer one of these questions quickly:

- What should I do now?
- How do I do this exercise?
- How am I progressing?
- Who can I train with?

Avoid presenting 6–8 equally important actions.

### 2.2 Guidance before data density

Spot may collect detailed training data, but the interface should translate it into useful decisions.

Prefer:

> **Try 12 kg again — aim for 10 clean reps.**

over:

> Volume +8.2% · RPE 8.4 · intensity 67% · estimated 1RM 31.4 kg

Detailed metrics can live in drill-down views.

### 2.3 Beginner without being childish

The app should be easy for a first-time gym user without looking juvenile or patronizing.

Use plain language, visual instruction and contextual help. Do not dumb down the brand.

### 2.4 Social should lead to real training

Spot is not primarily an entertainment feed.

Social features should encourage:

- training together
- accountability
- encouragement
- discovering people at the same gym
- sharing meaningful progress

Avoid designing the product around passive scrolling.

### 2.5 Real-world gym conditions matter

The ideal training plan is often interrupted by:

- occupied equipment
- limited time
- low energy
- soreness
- missing equipment
- changing gym location
- training with a friend

The UI should make adjustment feel normal rather than like failure.

### 2.6 Fitness safety over gamification

Never encourage the user to ignore sharp pain, unsafe technique, exhaustion or recovery needs to preserve a streak.

Spot is not a medical diagnostic tool. Health-related language must be careful and appropriately bounded.

### 2.7 Privacy by default for location and social data

Exact live location should not be publicly exposed.

Use gym membership, district/area and opt-in presence rather than continuously visible coordinates.

### 2.8 Finite experiences

Workout sessions, challenges and social surfaces should have clear stopping points.

Do not build infinite-scroll behavior into every surface simply because content exists.

---

## 3. Brand personality

Spot should feel like:

- knowledgeable
- energetic
- supportive
- contemporary
- confident
- social
- non-judgmental
- clean
- precise

Spot should **not** feel like:

- macho “beast mode” gym culture
- aggressive supplement advertising
- clinical hospital software
- corporate SaaS dashboards
- a meditation/wellness cliché
- a TikTok clone with workouts added
- a technical strength-training spreadsheet

### Voice examples

Good:

- **Ready for Push Day?**
- **That machine is busy? Swap it.**
- **Nice — all 10 reps were cleaner than last week.**
- **Only 25 minutes? We’ll keep the important work.**
- **You missed Monday. Continue from here.**

Avoid:

- **NO EXCUSES. DESTROY YOUR LIMITS.**
- **You failed your weekly goal.**
- **Optimal hypertrophy protocol recalculated.**
- **Your wellness journey awaits.**

---

## 4. Logo and identity

The Spot mark consists of **two simplified human forms flowing into an infinity / linked-motion shape**.

It communicates:

- people
- partnership
- continuity
- movement
- mutual support

The mark should remain simple and should not gain dumbbells, hearts, lightning bolts or other literal fitness symbols.

### Logo use

- Use the full mark on onboarding, splash, sign-in and marketing moments.
- Use the symbol alone for app icon and compact brand moments.
- Maintain generous clear space around the symbol.
- Do not place the mark over noisy photography unless contrast is guaranteed.
- Do not stretch, outline, skew or add glow effects.
- Do not recreate the mark with system icons.

---

## 5. Visual identity — color tokens

The logo establishes a cool, energetic blue-to-violet identity.

The following tokens are based on the uploaded logo and are the recommended V1 system palette.

| Token | Value | Usage |
|---|---|---|
| `spotBlue` | `#063DB2` | Core brand, primary actions, active states |
| `spotBlueDeep` | `#052E86` | Pressed states, dark panels, strong contrast |
| `spotIndigo` | `#5568FD` | Secondary brand accent, social/live states |
| `spotIndigoSoft` | `#EEF0FF` | Soft highlighted backgrounds |
| `canvas` | `#F8F9FC` | Default app background |
| `surface` | `#FFFFFF` | Primary cards/sheets |
| `surfaceSoft` | `#F1F4F9` | Grouped secondary surfaces |
| `ink` | `#111827` | Primary text |
| `inkMuted` | `#667085` | Secondary text |
| `hairline` | `#E4E7EC` | Dividers/borders |
| `success` | `#168A5B` | Completed/positive states |
| `warning` | `#B86E00` | Caution / recovery / attention |
| `danger` | `#C83C4A` | Errors/destructive actions |
| `live` | `#6B5CFF` | Live/pair-session presence |

### Gradient

Primary brand gradient:

```text
#063DB2 → #5568FD
```

Use it sparingly for:

- logo-driven hero areas
- workout start moments
- completion moments
- selected onboarding art
- live pair-workout accents

Do **not** use gradient text for body copy or make every primary button gradient.

### Color discipline

Default functional screens should mostly use:

`canvas + surface + ink + spotBlue`

`spotIndigo` adds energy, not noise.

One strong accent area per screen is usually enough.

Color must never be the only indicator of:

- exercise completion
- active set
- PR
- warning
- selected option
- friend presence

Always pair color with labels, icons or state changes.

---

## 6. Dark mode

Dark mode is useful for gym environments and may be supported in V1 if engineering scope allows.

Recommended dark tokens:

| Token | Value |
|---|---|
| `canvasDark` | `#090D16` |
| `surfaceDark` | `#111827` |
| `surfaceDarkRaised` | `#182131` |
| `inkDark` | `#F8FAFC` |
| `inkMutedDark` | `#98A2B3` |
| `hairlineDark` | `#273244` |
| `spotBlueDarkMode` | `#4D7CFF` |
| `spotIndigoDarkMode` | `#7B83FF` |

Dark mode should be intentionally designed — never a naive color inversion.

Workout execution screens are the highest priority dark-mode surfaces.

---

## 7. Typography

Recommended pair:

- **Sora** — expressive headings, onboarding moments, major workout/completion numbers
- **Inter** — body copy, navigation, forms, metrics, timers, labels and dense UI

If implementation simplicity matters, **Inter alone is acceptable**.

### Type scale

| Role | Size | Use |
|---|---:|---|
| Display | 34–40sp | onboarding / major completion moment |
| Page title | 26–30sp | top-level screens |
| Section heading | 20–24sp | grouped content |
| Card title | 17–19sp | workout/program/friend cards |
| Body | 15–17sp | normal UI copy |
| Supporting | 13–14sp | metadata |
| Label | 11–12sp | compact status/eyebrow labels |
| Workout metric | 28–44sp | reps, timer, weight where relevant |

### Typography rules

- Use no more than 3 major text sizes on a typical screen.
- Keep workout metrics highly legible at arm’s length.
- Use tabular numerals for timers and changing counters.
- Avoid heavy all-caps except tiny state labels where justified.
- Do not center long paragraphs.
- Support dynamic type / font scaling.
- Never allow long exercise names or usernames to break the layout.

---

## 8. Shape, depth and iconography

### Shape

- cards: `18–24` radius
- large workout panels: `24–28`
- inputs: `12–16`
- buttons: `14–18`, or pill for compact actions
- avatar: circular
- chips: pill

The rounded geometry should echo the Spot mark without becoming cartoonish.

### Depth

Use hierarchy in this order:

1. whitespace
2. background/surface contrast
3. divider
4. border
5. shadow

**Card inflation is a defect.**

Do not nest card inside card without a meaningful reason.

Use subtle elevation mainly for:

- bottom sheets
- sticky workout controls
- floating live-session elements
- overlays

### Icons

Use one clean line-icon family throughout the product.

Icons should be:

- visually simple
- recognizable at 20–24pt
- paired with labels for important navigation/actions
- filled only for selected navigation states where appropriate

Avoid bodybuilding clichés as navigation icons.

---

## 9. Photography, illustration and 3D exercise media

Spot should use **realistic, useful fitness imagery** rather than aspirational body-transformation advertising.

### Photography

Prefer:

- normal people training
- varied body types
- real gym environments
- partner workouts
- preparation / rest / setup moments

Avoid:

- exaggerated bodybuilder physiques dominating beginner screens
- sexualized fitness photography
- impossible transformation imagery
- excessive stock-photo posing

### 3D exercise demonstrations

Exercise demos are functional content, not decoration.

Each demo should prioritize:

- clear silhouette
- obvious joint movement
- unobstructed equipment
- stable camera angle
- loopability
- correct start/end posture

Recommended visual treatment:

- neutral simplified avatar
- neutral or very soft background
- highlighted active muscle areas only when informative
- no unnecessary gym scene clutter
- front / side / 3-quarter camera depending on exercise

The demo card must include accessible textual cues; animation alone is not enough.

---

## 10. Layout system

Reference frames:

- compact phone: `360 × 780`
- common phone: `390 × 844`
- large phone: `430 × 932`
- small tablet: `768 × 1024`

These are design references, not fixed dimensions.

### Spacing scale

`4, 8, 12, 16, 20, 24, 32, 40, 48`

Typical use:

| Relationship | Gap |
|---|---:|
| icon + label | 8 |
| title + metadata | 4–8 |
| controls in a group | 8–12 |
| card content | 12–16 |
| card padding | 16–20 |
| independent sections | 24–32 |
| major screen sections | 32–40 |
| horizontal screen padding | 20–24 |

### Layout rules

- Respect safe areas.
- Keep primary controls reachable with one hand where practical.
- Use max-widths on tablets rather than stretching phone cards.
- Keep workout logging controls large enough to use while standing or fatigued.
- Avoid important actions under the keyboard.
- Do not hide essential workout actions behind horizontal swipes.
- Portrait first; exercise media may expand landscape if useful.

---

## 11. Presentation modes

Spot has four presentation modes within one visual system.

| Mode | Screens | Behavior |
|---|---|---|
| **Guide** | Today, program, exercise detail, onboarding | calm, explanatory, one next step |
| **Workout** | active session, set logging, rest timer | high contrast, large controls, minimal distraction |
| **Progress** | history, PRs, program progress | clear charts and meaningful summaries, restrained density |
| **Social** | friends, gym, chat, pair workout | warmer/person-led, avatars and presence, still task-oriented |

The modes should feel related, not like four different applications.

---

## 12. Navigation architecture

Recommended V1 bottom navigation:

1. **Today**
2. **Train**
3. **Social**
4. **Progress**
5. **Profile**

### Today

Answers: **What should I do today?**

Contains:

- recommended workout
- recovery/readiness prompt when useful
- active program state
- upcoming workout
- compact friend/gym activity

### Train

Contains:

- current program
- programs
- exercise library
- workout history entry points
- start custom workout

### Social

Contains:

- friends
- gym community
- pair workout invites
- accountability
- chat
- limited activity feed

### Progress

Contains:

- consistency
- PRs
- program progress
- workout history
- body metrics if user enables them

### Profile

Contains:

- public/private profile
- preferences
- gyms
- privacy
- notifications
- account/settings

### Focused-session rule

When a workout is active, Spot enters a **focused workout shell**.

The normal tab bar should not compete with the session.

Provide explicit:

- minimize
- pause
- finish
- exit

behavior.

---

## 13. Navigation schematic

```text
(app)
├── (auth)
│   ├── welcome
│   ├── sign-in
│   ├── verify-code
│   └── provider-return
├── (onboarding)
│   ├── basics
│   ├── goals
│   ├── experience
│   ├── schedule
│   ├── location-gym
│   ├── equipment
│   ├── limitations
│   └── recommendation
├── (tabs)
│   ├── today
│   ├── train
│   ├── social
│   ├── progress
│   └── profile
├── workout/[sessionId]
│   ├── overview
│   ├── exercise/[exerciseId]
│   ├── demo
│   ├── substitute
│   ├── rest
│   ├── pair
│   └── complete
├── programs/[programId]
├── exercises/[exerciseId]
├── friends/[userId]
├── gyms/[gymId]
├── chats/[conversationId]
├── accountability/[relationshipId]
├── notifications
└── settings/*
```

Route names are design guidance, not an implementation mandate.

---

## 14. Screen inventory — canonical V1

The following screens define the major product experience. Stitch/Figma generations should be derived from these rather than inventing unrelated screen structures.

### A. Authentication and onboarding

**01 · Welcome / sign in**  
Logo, concise statement, Google/Apple where implemented, email/phone or code flow, terms/privacy. Keep marketing minimal.

**02 · Verification code**  
Large clear code fields, resend timer, change address, loading/error/expired states.

**03 · Basic profile**  
Name, username, avatar, optional age range and basic identity fields actually needed by the product.

**04 · Fitness goal**  
Large multi-select or ranked cards: general fitness, build muscle, get stronger, lose fat, improve mobility, return to training, etc. Avoid promising guaranteed outcomes.

**05 · Experience**  
New / some experience / consistent lifter. Explain choices in plain language rather than technical classifications.

**06 · Training schedule**  
Days per week, typical duration, preferred days. Show realistic recommended ranges without forcing them.

**07 · Training environment**  
Gym, home, mixed. Search/select primary gym or district; allow skip.

**08 · Equipment**  
Pre-populate from selected gym if known. User can confirm available equipment.

**09 · Preferences and limitations**  
Preferred/disliked exercises, optional limitations or areas requiring care. Sensitive inputs should be optional and clearly explained.

**10 · Program recommendation**  
One strong recommendation plus 1–2 alternatives, with why it fits. CTA: **Start this program**.

---

### B. Daily guidance

**11 · Today**  
The most important non-workout screen. Show one dominant card: today’s session / rest / recovery recommendation. Secondary content: weekly progress, friend activity, short wellness check-in.

**12 · Readiness check-in**  
Fast inputs: energy, soreness, sleep quality, available time. Should take under ~15 seconds. Do not imply medical precision.

**13 · Today adjusted**  
If input changes the plan, show a concise explanation: “You only have 25 minutes, so we kept the three highest-priority movements.”

---

### C. Programs and exercise discovery

**14 · Current program**  
Program overview, schedule, current week/session, goal, equipment, progress and edit/swap controls.

**15 · Program library**  
Goal filters, experience, equipment, days/week, duration. Avoid giant card grids. One clear recommendation section first.

**16 · Program detail**  
Who it is for, schedule, session structure, expected duration, equipment, example exercises, progression approach.

**17 · Exercise library**  
Search, muscle group, movement, equipment and difficulty filters. Favor high-quality library over endless generated variants.

**18 · Exercise detail**  
3D demo/video, setup, execution, cues, common mistakes, muscles, alternatives and history.

**19 · Exercise substitution**  
Reason-aware suggestions: machine busy, equipment unavailable, discomfort, too difficult, preference. Explain why each replacement is suitable.

---

### D. Active workout

**20 · Workout start**  
Session name, approximate duration, exercise count, friend/pair status, short readiness note and Start CTA.

**21 · Active exercise**  
This is the core product screen.

Must foreground:

- exercise name
- demo preview
- current set
- target reps/load
- previous performance
- large weight/reps controls
- Complete Set
- concise technique cue

Everything else is secondary.

**22 · Exercise demo full-screen**  
Looping 3D/video demo, angle controls only if actually supported, setup, cues, mistakes, close/back without losing set state.

**23 · Set feedback**  
After a set: Easy / Good / Hard / Failed, optional pain/discomfort action. Feedback must be one-tap when possible.

**24 · Rest timer**  
Large countdown, +15/+30 seconds, Skip, next-set preview. The timer must not lock the user into waiting.

**25 · Swap / skip / modify**  
Quick bottom sheet. Machine busy should be a first-class reason, not hidden in menus.

**26 · Workout minimized**  
Persistent compact state if user navigates elsewhere: session name, elapsed time, next action, resume.

**27 · Workout complete**  
Celebrate meaningful progress: completed session, PRs, progression, consistency, partner summary. Avoid confetti for ordinary actions; completion may use restrained celebratory motion.

---

### E. Progress

**28 · Progress overview**  
Lead with understandable progress: sessions this month, consistency, current program progress, recent PR. Do not present 10 charts at once.

**29 · Workout history**  
Chronological list/calendar with session summaries and filters.

**30 · Exercise history**  
Load/reps trends, best sets, recent history, estimated progression. Keep charts readable and optional.

**31 · Personal records**  
Recent/new/all PRs, contextualized by exercise and rep range.

**32 · Body progress**  
Optional weight/measurement/progress-photo tracking. Explicitly private by default.

---

### F. Friends and community

**33 · Social home**  
Friends training now/recently, pair invites, accountability status, gym activity. The feed is secondary to actions.

**34 · Find friends**  
Username/name search, contacts only with permission, mutual friends, gym/district suggestions where privacy allows.

**35 · User profile**  
Avatar, username, short bio, gym only if visible, training interests, selected stats, shared activity, Add Friend / Message / Invite.

**36 · Friend requests**  
Incoming/outgoing, simple accept/decline, context such as mutual friend/gym when allowed.

**37 · Gym community**  
Gym name, members on Spot, opt-in training presence, challenges, upcoming pair/group sessions and relevant activity.

**38 · District community**  
Fallback area-level discovery where gym is not selected. Keep exact location hidden.

**39 · Activity post**  
Workout share, PR, image or short reel, caption, visibility selector. Workout content should be easier to post than generic lifestyle content.

**40 · Post detail**  
Reactions/comments, workout context, author profile. Avoid engagement-count dominance.

---

### G. Pair workout and accountability

**41 · Pair workout invite**  
Friend, proposed workout, time/now, accept/decline, clear session context.

**42 · Pair workout lobby**  
Partner presence, selected program/session, sync status, Start Together.

**43 · Live pair workout**  
Own workout remains primary. Partner state is secondary: current exercise, set completion, quick cheer/reaction, short text.

Do not force both people to move through sets in lockstep unless they choose a synchronized mode.

**44 · Pair workout summary**  
Both users’ completion, shared streak/milestone, next invitation.

**45 · Accountability setup**  
Select partner, permission scope, planned frequency, what they can see. Make all visibility explicit.

**46 · Accountability status**  
Weekly commitments, completed/missed sessions and encouragement. Avoid shame language.

---

### H. Chat and notifications

**47 · Inbox**  
1:1 and group conversations, unread state, pair/workout context.

**48 · Chat**  
Text, images/video where supported, workout/program/exercise sharing, quick invite-to-train action.

**49 · Notifications**  
Friend request, message, workout invite, cheer, accountability, PR/social reaction. Group repetitive notifications.

---

### I. Profile and settings

**50 · Own profile**  
Preview how others see the profile, visibility controls, edit entry, activity and selected training information.

**51 · Edit profile**  
Name, username, image, bio, location/district, gyms, training interests.

**52 · Privacy and discoverability**  
Public/private profile, friend-request rules, gym visibility, district visibility, activity visibility, body metrics, training presence.

**53 · Workout preferences**  
Units, default rest timers, exercise preferences, equipment, workout sounds/haptics.

**54 · Notifications settings**  
Workout reminders, social, accountability, chat, gym/community.

**55 · Account and security**  
Auth providers, sessions, data export, account deactivation/deletion.

---

### J. System states

**56 · Offline / poor connection**  
Active workout logging should remain resilient where possible. Make sync state visible without interrupting the workout.

**57 · Empty-state gallery**  
No program, no friends, no gym, no history, no messages, no PRs, no posts.

**58 · Error / retry gallery**  
Auth failure, program load error, media unavailable, chat disconnected, workout sync conflict, location unavailable.

---

## 15. Today screen design doctrine

The Today screen must not become a dashboard dump.

Default hierarchy:

1. greeting / context
2. **primary recommended action**
3. small weekly context
4. one social/community prompt if useful
5. optional wellness check-in

Example:

```text
Good evening, Brandon

[ PUSH · WEEK 3 ]
Upper Body A
~52 min · 6 exercises

Continue your program
[ Start workout ]

This week
2 / 4 workouts

Training nearby
David plans to train at 7:00 PM
```

Avoid:

- calories, streak, weight, heart rate, social likes and 6 programs all above the fold
- giant generic motivational quote
- large charts on the default Today view

---

## 16. Active workout design doctrine

The active workout experience is Spot’s **highest-priority interface**.

### Primary goals

The user should be able to:

- identify the current exercise instantly
- understand how to execute it
- log a set in a few taps
- know what they did previously
- change the plan without losing flow
- use the screen while tired, sweaty or moving between equipment

### Active set hierarchy

```text
Exercise title
Demo / form cue

SET 2 OF 3
Previous: 12 kg × 10
Target: 12 kg × 10

[  -  ]   12 kg   [  +  ]
[  -  ]   10 reps [  +  ]

[ Complete set ]

Swap exercise · More
```

### Rules

- Completed sets collapse but remain inspectable.
- Current set is visually dominant.
- Weight/reps controls must be large.
- Keyboard entry is optional, not required for every set.
- Previous values should prefill intelligently.
- Logging must feel instant; optimistic updates are acceptable if sync state is handled safely.
- The next set should never require navigating through multiple screens.
- Rest timer should start automatically only if the user has enabled it or the program defines it clearly.

---

## 17. Exercise guidance doctrine

Every exercise should answer:

1. **What am I doing?**
2. **How do I set up?**
3. **What should it feel like?**
4. **What common mistake should I avoid?**
5. **What can I do instead?**

Recommended structure:

```text
3D demo

Incline Dumbbell Press
Chest · Shoulders · Triceps

Setup
• Bench around 30°
• Feet planted
• Shoulder blades gently back

Do
• Lower with control
• Press up without bouncing

Watch out
• Excessive elbow flare
• Losing shoulder position

[ Alternatives ]
```

Do not overload the user with anatomy or coaching theory during a live set.

---

## 18. Personalization and AI presentation

Spot may use AI/rules to recommend or adjust workouts, but the interface should not become “chatbot-first.”

### Good AI use

- suggest today’s workout
- adjust duration
- choose a substitute
- explain an exercise
- summarize progress
- answer contextual training questions
- suggest progression based on history

### Presentation

Prefer direct action:

> **You only have 25 minutes. We kept the three highest-priority exercises.**

with:

`Keep adjustment` · `Restore full workout`

instead of forcing the user into a chat conversation.

### Assistant entry points

A small contextual **Ask Spot** action may appear on:

- exercise detail
- active workout
- program detail
- progress summary

The assistant should inherit relevant context rather than asking the user to repeat basic facts.

### Safety

Do not present AI output as medical diagnosis or guaranteed injury prevention.

Sharp pain or concerning symptoms should trigger a conservative stop-and-seek-qualified-help message rather than an exercise optimization answer.

---

## 19. Social design doctrine

### Friends, not followers

V1 is based on mutual friendship.

Primary actions:

- Add friend
- Message
- Invite to workout
- Cheer
- Add accountability partner

Follower counts should not exist in V1.

### Social content hierarchy

Favor:

1. workout activity
2. PRs / milestones
3. training invitation
4. image / reel
5. generic lifestyle posting

Do not let generic content overwhelm the fitness purpose.

### Public/private profiles

Private profile:

- searchable only according to discoverability setting
- user must accept friend request before protected activity is visible

Public profile:

- selected profile/activity data may be visible
- sensitive body metrics remain separately controlled

### Social counts

Do not overemphasize likes, comment counts or popularity.

The social reward should be **connection and training**, not audience size.

---

## 20. Gym and location UX

Spot’s local graph can become a major differentiator, so location design must be useful without being invasive.

### Gym-first location model

Prefer:

`User → Gym → Community`

rather than:

`User → exact live coordinates`

### Visibility choices

Examples:

- Show my gym on profile
- Show me in gym member discovery
- Show when I plan to train
- Show when I am currently training
- Allow people from my gym to send friend requests
- Show my district

Each setting should be independently understandable.

### Nearby language

Prefer:

- **People at your gym**
- **People in your area**

Avoid displaying precise distance such as “43 m away” unless there is a future product reason and explicit consent.

---

## 21. Pair workout UX

Pair workouts should feel collaborative, not restrictive.

### Principles

- Both users own their own workout state.
- Shared session connects them; it does not make one device the source of truth for both.
- Partner progress should not cover the main logging UI.
- Quick reactions should be one tap.
- Text chat should be available but secondary during active sets.

### Partner presence examples

```text
David
Lat Pulldown · Set 2/3
[ 👏 Cheer ]
```

or:

```text
David just finished his set
Nice work 🔥
```

Keep celebratory effects brief.

---

## 22. Accountability UX

Accountability must be opt-in and permission-based.

The setup flow must explicitly show what the partner can see.

Possible scopes:

- planned workout days
- whether workout started/completed
- weekly consistency
- streak
- selected PRs
- selected program progress

Body weight, measurements, progress photos, health notes and private journal content should never be shared by default.

### Language

Prefer:

> **Brandon planned to train today but hasn’t checked in yet.**

Avoid:

> **Brandon failed his workout.**

The system should encourage, not shame.

---

## 23. Feed and reels constraints

Spot can support images and short reels, but they must not turn the product into a generic entertainment platform.

### Feed rules

- finite initial batch
- explicit refresh
- workouts and friends prioritized
- no default autoplay with sound
- visible context for why content appears
- avoid engagement-bait UI

### Reel rules

- short fitness-related posts
- tap for sound
- captions supported
- respect reduced motion / autoplay preferences
- no endless forced loop as the primary social home

---

## 24. Wellness UX

Wellness should support training decisions rather than becoming a separate lifestyle application in V1.

Recommended inputs:

- sleep quality
- energy
- soreness
- mood / motivation where useful
- available time

### Check-in design

Use quick, low-friction controls.

Example:

```text
How are you feeling?

Energy      Low  ●  High
Sleep       Poor ●  Great
Soreness    None ●  High
Time today  25 min

[ Adjust today’s workout ]
```

Do not imply that simple self-report data is a medical readiness score.

---

## 25. Core reusable components

Create reusable native components rather than styling every screen independently.

| Component | Purpose | Critical states |
|---|---|---|
| `SpotScreen` | safe-area-aware screen wrapper | scroll / fixed / keyboard-aware |
| `SpotHeader` | compact navigation header | back / close / action |
| `SpotButton` | primary/secondary action | pressed / disabled / loading / destructive |
| `SpotCard` | meaningful grouped object | idle / selected / disabled |
| `SpotAvatar` | user identity | online / training / private / loading |
| `WorkoutCard` | session recommendation | upcoming / active / complete / skipped |
| `ProgramCard` | program preview | recommended / current / saved |
| `ExerciseRow` | exercise list item | available / complete / substituted |
| `ExerciseDemo` | 3D/video preview | loading / playing / paused / unavailable |
| `SetRow` | set logging | future / current / complete / failed |
| `MetricStepper` | reps/weight control | active / disabled / manual input |
| `RestTimer` | workout countdown | active / paused / complete |
| `ProgressRing` | compact meaningful progress | empty / partial / complete |
| `FriendRow` | social identity/action | friend / request / training-now |
| `PresenceChip` | availability/status | offline / online / training |
| `GymCard` | gym/community object | joined / discoverable / private |
| `PairSessionBar` | live partner status | connecting / live / paused / ended |
| `ChatComposer` | message entry | text / media / sending / failed |
| `PrivacyRow` | visibility setting | public / friends / private |
| `SpotSheet` | contextual bottom sheet | collapsed / expanded / keyboard |
| `InlineNotice` | warning/info/sync | info / warning / error / success |
| `Skeleton` | layout-preserving loading | active |

### Component rule

A reusable component should encode:

- spacing
- typography
- interaction states
- accessibility semantics
- disabled/loading behavior

not merely visual appearance.

---

## 26. Buttons and actions

### Primary action

- one dominant primary CTA per screen where practical
- `spotBlue` fill
- white text
- 50–56pt typical height
- full-width only when hierarchy benefits from it

### Secondary

- soft surface or outline
- dark text / spot blue

### Tertiary

- text/icon action

### Destructive

- danger color only when genuinely destructive
- confirmation for account deletion, friendship/block-sensitive actions where required, or discarding meaningful workout data

Do not show confirmation modals for routine reversible actions.

---

## 27. Inputs and forms

- Minimum control height: 48–52pt.
- Labels remain visible; do not rely only on placeholders.
- Validate after meaningful interaction, not immediately on focus.
- Group onboarding into short coherent steps.
- Use segmented controls/chips for small known choice sets.
- Use searchable sheets for gyms and exercises.
- Use native date/time controls where appropriate.
- Preserve partially completed onboarding data.

Error messages should be actionable:

Good:

> **Username is already taken. Try another.**

Bad:

> **Validation error 409.**

---

## 28. Motion and haptics

Motion should communicate state, not decorate every surface.

| Motion | Guideline |
|---|---|
| press feedback | ~100–150 ms |
| simple state change | ~150–220 ms |
| sheet/modal | ~200–300 ms |
| workout completion | ~400–700 ms |
| PR celebration | ~500–900 ms, restrained |

Recommended haptics:

- set complete: light
- workout complete: medium
- PR: success pattern
- destructive warning: warning pattern

Haptics are supplementary and can be disabled.

Honor reduced-motion settings.

Avoid:

- constant bouncing
- looping celebration
- full-screen animation after every set
- motion that delays workout logging

---

## 29. Charts and progress visualization

Charts should answer a specific question.

Examples:

- **Am I training consistently?** → sessions per week
- **Am I getting stronger at this exercise?** → best-set load/reps trend
- **How far through this program am I?** → program progress

Rules:

- show summary before chart
- avoid 3D charts
- avoid unnecessary dual axes
- do not use five colors when one series is enough
- make touch targets large enough for mobile
- use text summaries for accessibility

---

## 30. Loading, empty, success and error states

Every production screen must define the important states.

### Loading

Use skeletons that resemble the final structure.

Avoid generic spinners for entire content-heavy screens when partial skeletons are possible.

### Empty

Every empty state should answer:

1. Why is this empty?
2. Is that normal?
3. What can I do next?

Example:

> **No friends here yet**  
> Find someone you already know or people at your gym.  
> **Find friends**

### Errors

Keep current user work whenever possible.

During workouts, local logging should not vanish because a network request failed.

### Offline

Show non-blocking state such as:

> **Offline — your workout will sync when you reconnect.**

---

## 31. Interaction-state matrix

| Flow | Required states |
|---|---|
| Sign in | empty, validating, sending, code sent, expired, invalid, provider failure |
| Onboarding | incomplete, valid, saving, saved, interrupted, resumed |
| Gym search | permission unknown, searching, results, no results, manual entry, error |
| Program recommendation | generating/loading, ready, alternatives, error |
| Workout | ready, active, minimized, paused, offline, syncing, complete |
| Set logging | current, editing, complete, failed save, synced |
| Exercise media | thumbnail, loading, playing, paused, failed |
| Substitution | reason selected, loading, choices, accepted, reverted |
| Pair workout | invited, waiting, joining, live, partner disconnected, ended |
| Friendship | none, requested, incoming, accepted, blocked |
| Chat | empty, sending, sent, failed, reconnecting |
| Accountability | invited, active, permission changed, paused, removed |
| Post | composing, uploading, published, upload failed, removed |
| Profile privacy | original, edited, saving, success, error |

---

## 32. Accessibility

Accessibility is a product requirement, not polish.

### Minimum rules

- interactive targets at least 44×44pt; prefer larger in active workout UI
- sufficient color contrast
- screen-reader labels for icons and media controls
- logical focus order
- dynamic text support
- do not encode state by color only
- captions or textual equivalent for exercise video where practical
- reduced-motion support
- usable without precise gestures
- haptics never required to understand feedback

### Workout-specific accessibility

- large set controls
- high-contrast timer
- voice-over-friendly set state
- visible labels for weight/reps
- optional sound/haptic countdown
- landscape media only as enhancement

---

## 33. Safety and responsible fitness language

Spot should never visually pressure the user to continue an unsafe workout.

### Pain/discomfort flow

If the user reports sharp pain or concerning discomfort:

- suggest stopping the movement
- offer a non-diagnostic alternative path
- recommend professional help when appropriate
- do not simply recommend reducing weight and continuing

### Body image

Avoid UI that frames body weight or appearance as moral success/failure.

Body metrics are optional and private by default.

### Streaks

Rest days should not “break” a healthy consistency streak if the program expects rest.

Measure adherence to the plan rather than raw daily app usage.

---

## 34. Privacy and social safety

At minimum, Spot should support:

- block user
- report user
- remove friend
- mute user
- private profile
- restrict messages
- control gym visibility
- control district visibility
- control current-training visibility
- control post visibility

### Location safety

Never show another user’s exact home location.

Do not expose exact live gym presence unless the user explicitly opts into that experience.

### Media

Provide reporting for:

- harassment
- inappropriate media
- spam
- impersonation
- dangerous content

---

## 35. Notification design

Notifications should correspond to meaningful action.

Priority examples:

1. direct message
2. friend request
3. pair workout invite
4. accountability prompt
5. friend started planned pair session
6. program/workout reminder
7. social reaction

Avoid notification spam for every set or trivial social action.

### Notification copy

Good:

> **David invited you to Upper Body A at 7:00 PM.**

> **Ada cheered your workout.**

> **You planned to train today. Ready when you are.**

Avoid guilt:

> **You’re falling behind!**

---

## 36. Search design

Different search contexts should remain distinct.

### Exercise search

Prioritize:

- exact name
- movement
- muscle
- equipment

### Program search

Prioritize:

- goal
- days/week
- experience
- equipment
- duration

### Friend search

Prioritize:

- username/name
- mutual friends
- same gym where allowed

### Gym search

Prioritize:

- name
- district/area
- city

Do not merge every content type into one noisy global search in V1.

---

## 37. Content hierarchy and text economy

Use the shortest copy that preserves meaning.

Prefer:

```text
Incline Dumbbell Press
Chest · 3 sets
Previous 12 kg × 10
```

instead of:

```text
Exercise Information
Your next exercise is the Incline Dumbbell Press.
This movement primarily targets your chest muscles.
You have three sets remaining to complete.
```

### Rules

- Avoid eyebrow + heading + subtitle + helper text on every card.
- Do not repeat the tab name as a large heading unless needed.
- One card usually needs a title and one useful metadata line.
- Keep workout instructions skimmable.
- Privacy/safety text may be longer when clarity requires it.

---

## 38. Progressive visual simplification

When a screen feels cluttered, simplify in this order:

1. remove redundant copy
2. improve spacing/alignment
3. remove unnecessary cards
4. reduce borders
5. reduce accent colors
6. collapse secondary metrics
7. move advanced actions into a sheet/drill-down

Do **not** simplify by hiding critical workout controls or privacy information.

---

## 39. Screen-specific simplicity rules

| Screen | Foreground | Avoid |
|---|---|---|
| Welcome | logo, promise, sign-in | landing-page marketing sections |
| Onboarding | one question/task | 20-field forms |
| Today | next workout | dashboard of every metric |
| Program library | recommendation + filters | endless identical cards |
| Exercise detail | demo + cues | anatomy textbook density |
| Active workout | current set | social feed/navigation clutter |
| Rest timer | timer + next action | modal lock-in |
| Workout complete | meaningful progress | celebration overload |
| Progress | one understandable trend | 8 charts at once |
| Social | people/actions | generic content feed dominance |
| Gym | members + training/community | precise location tracking |
| Pair workout | own workout + partner status | split-screen overload |
| Chat | conversation + workout share | mini social network inside chat |
| Profile | identity + useful fitness context | every private metric |
| Settings | flat grouped rows | card per setting |

---

## 40. Responsive behavior

### Phones

Primary target. One main column.

### Tablets

Use wider compositions intentionally:

- program detail can use summary + session list
- exercise detail can place media beside cues
- progress can use chart + history
- chat can use split list/conversation

Do not simply stretch cards edge to edge.

### Foldables / wide screens

Treat as tablet-like only when enough width exists.

Preserve touch reachability and active-workout clarity.

---

## 41. Native behavior expectations

- use native safe areas
- standard back behavior
- preserve active workout state during navigation
- confirm before discarding meaningful unsaved session changes
- keyboard-aware forms/chat
- native share sheet where useful
- deep-link into friend invites, programs, gyms and pair sessions
- permission prompts only immediately before the feature needs them
- do not request contacts/location during first launch without context

---

## 42. Design-to-engineering contracts

Every screen handed to engineering should define:

- default state
- loading state
- empty state where relevant
- error state
- disabled state
- offline behavior where relevant
- primary/secondary actions
- navigation/back behavior
- data dependencies
- privacy implications

Generated designs must be implementable with standard React Native primitives and chosen libraries.

Do not approve a design that depends on:

- impossible blur/effects across platforms
- desktop hover behavior
- invisible gesture-only controls
- fake live data
- unsupported AI capability
- precise location exposure not present in product policy

---

## 43. Design tokens — suggested implementation shape

Example token structure:

```ts
export const colors = {
  canvas: '#F8F9FC',
  surface: '#FFFFFF',
  surfaceSoft: '#F1F4F9',
  ink: '#111827',
  inkMuted: '#667085',
  hairline: '#E4E7EC',
  brand: '#063DB2',
  brandPressed: '#052E86',
  accent: '#5568FD',
  accentSoft: '#EEF0FF',
  success: '#168A5B',
  warning: '#B86E00',
  danger: '#C83C4A',
};

export const radius = {
  sm: 10,
  md: 14,
  lg: 20,
  xl: 24,
  pill: 999,
};

export const space = {
  xs: 4,
  sm: 8,
  md: 12,
  lg: 16,
  xl: 24,
  xxl: 32,
  xxxl: 40,
};
```

Token names matter more than exact code shape.

---

## 44. Suggested design-generation batches

When using Stitch, Figma AI, MagicPath or another generative design tool, work in batches rather than asking for 58 unrelated screens at once.

### Batch 1 — Brand foundation

- Welcome
- Sign in
- Basic onboarding
- Program recommendation

Goal: establish typography, spacing, colors and component feel.

### Batch 2 — Core workout

- Today
- Workout start
- Active exercise
- Rest timer
- Substitute exercise
- Workout complete

**This batch is highest priority.**

### Batch 3 — Programs and exercise guidance

- Program library
- Program detail
- Exercise library
- Exercise detail

### Batch 4 — Progress

- Overview
- History
- Exercise progress
- PRs

### Batch 5 — Social

- Social home
- Find friends
- User profile
- Gym community
- Post

### Batch 6 — Live social

- Pair invite
- Lobby
- Pair workout
- Pair summary
- Accountability setup/status
- Chat

### Batch 7 — Settings and states

- own profile
- privacy
- settings
- notifications
- offline
- empty/error gallery

Do not proceed to the next large batch until the visual system from earlier batches is coherent.

---

## 45. Master prompt for AI design tools

Use the following as a base instruction when generating Spot screens:

> Design a native mobile interface for **Spot**, a guided social fitness app for people starting or rebuilding a gym habit. The core brand promise is **“Never train lost.”** Spot should feel like a knowledgeable, supportive gym friend — modern, confident, energetic and non-judgmental. Use a cool white canvas, near-black typography, primary deep blue `#063DB2` and electric indigo `#5568FD`, inspired by the linked two-person/infinity Spot logo. Use restrained rounded geometry, strong whitespace, minimal borders and very little decorative shadow. Avoid black-and-neon bodybuilding clichés, generic wellness gradients, enterprise dashboard density and card inflation. One obvious action should dominate each screen. Active workout interfaces must use large controls, highly legible metrics and minimal distraction. Social interfaces should prioritize friends, gym community, pair workouts and accountability rather than passive infinite scrolling. Exercise media should feel functional and precise. Create real native mobile states, safe-area spacing, accessible touch targets and implementable React Native behavior. Show realistic content rather than lorem ipsum.

For each screen request, additionally specify:

- screen name
- user state
- light/dark mode
- device frame
- key interaction
- default + loading + error state if applicable

---

## 46. Visual QA scorecard

Score every major screen from 1–5.

| Criterion | Pass test |
|---|---|
| Task clarity | primary action understood in ~3 seconds |
| Visual hierarchy | one obvious focal point |
| Brand cohesion | unmistakably Spot without logo dependence |
| Text economy | no redundant explanation |
| Workout usability | usable while standing/moving/tired |
| Social usefulness | connection/action over passive engagement |
| Beginner clarity | understandable without gym expertise |
| Accessibility | touch, text, contrast, semantics considered |
| Privacy | location/social exposure is explicit and safe |
| State completeness | loading/error/offline states are designed |
| Implementation realism | can be built in React Native without tricks |

### Reject/revise when

- every section is inside a card
- four CTAs have equal prominence
- the screen resembles a generic crypto/SaaS dashboard
- exercise logging requires too many taps
- important actions require hidden gestures
- social metrics dominate user identity
- exact location is exposed casually
- body transformation imagery creates the brand tone
- decorative gradients appear everywhere
- an AI mockup invents unsupported capabilities

---

## 47. Release QA checklist

### Layout

- test 360, 390, 430 and tablet width
- test long usernames/exercise names
- test large text
- test keyboard open
- verify safe-area behavior

### Workout

- start/resume/pause/finish
- background/foreground app during active session
- network loss mid-workout
- edit previous set
- rapid repeated set completion taps
- timer interruption
- substitute exercise
- pair disconnect/reconnect

### Social

- public/private profiles
- incoming/outgoing friend requests
- block/remove friend
- gym visibility
- accountability permission changes
- post upload failure
- message retry

### Accessibility

- VoiceOver/TalkBack
- dynamic type
- reduced motion
- high contrast
- target sizes
- non-color state indicators

### Privacy

- exact location never leaks into public UI
- sensitive metrics remain private unless explicitly shared
- profile visibility matches backend rules
- blocked users cannot bypass visibility through community surfaces

---

## 48. Priority / source-of-truth rules

1. **Safety and privacy beat visual novelty.**
2. **Active workout usability is the highest product-design priority.**
3. **This file controls mobile interaction and visual behavior.**
4. **The uploaded Spot logo controls the core brand direction.**
5. **Workout guidance should remain simpler than the underlying data model.**
6. **Social features should help people train, not maximize feed consumption.**
7. **Generated designs are references, not authority.** Preserve product logic when adapting them.
8. **Do not visually promise features that are not actually implemented.**
9. **Location visibility is always explicit and permission-based.**
10. **When in doubt, make the next useful action more obvious.**

---

## 49. Final design doctrine

The visual system should continuously reinforce this sequence:

> **Guide me → Help me train → Show me progress → Connect me to people.**

The user should never feel that Spot is asking them to become a fitness expert before they can use it.

The app succeeds when a beginner can walk into a gym, open Spot and immediately feel:

> **“I know what I’m doing next.”**

And over time:

> **“I have people here.”**

That is the design standard every screen should serve.
