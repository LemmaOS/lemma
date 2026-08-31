---
version: alpha
name: Lemma-web-design-system
description: "wait to be filled in"

# Token names mirror theme.css semantic variables; values copied verbatim (code is the source of truth)
colors:
  # Shared by both themes; warning/success TBD (per-theme values when the amber banner is adopted)
  shared:
    primary: "#60b1ff"
    primary-foreground: "#0b1220"
    ring: "color-mix(in srgb, primary 40%, transparent)"
    warning: TBD
    success: TBD
  light:
    background: "#fdfbfd"
    foreground: "oklch(0.21 0.01 260)"
    card: "oklch(1 0 0)"
    card-foreground: "oklch(0.21 0.01 260)"
    popover: "oklch(1 0 0)"
    popover-foreground: "oklch(0.21 0.01 260)"
    secondary: "oklch(0.96 0.003 260)"
    secondary-foreground: "oklch(0.3 0.01 260)"
    muted: "oklch(0.965 0.003 260)"
    muted-foreground: "oklch(0.5 0.012 260)"
    accent: "oklch(0.955 0.005 262)"
    accent-foreground: "oklch(0.28 0.02 262)"
    destructive: "oklch(0.55 0.18 25)"
    destructive-foreground: "oklch(0.98 0 0)"
    border: "oklch(0.91 0.005 260)"
    input: "oklch(0.91 0.005 260)"
    sidebar: "#fcf8fb"
    sidebar-foreground: "oklch(0.21 0.01 260)"
    sidebar-border: "oklch(0.91 0.005 260)"
    sidebar-accent: "oklch(0.965 0.002 95)"
    sidebar-accent-foreground: "oklch(0.28 0.02 262)"
    code: "oklch(0.96 0.004 260)"
    code-foreground: "oklch(0.28 0.02 260)"
    code-border: "oklch(0.91 0.005 260)"
    composer: "#ffffff"
  dark:
    canvas-from: "#0e0d0f"
    canvas-to: "#1c1e1b"
    background: "#151615"
    foreground: "oklch(0.92 0.005 260)"
    card: "oklch(0.24 0.008 260)"
    card-foreground: "oklch(0.92 0.005 260)"
    popover: "oklch(0.24 0.008 260)"
    popover-foreground: "oklch(0.92 0.005 260)"
    secondary: "oklch(0.27 0.009 260)"
    secondary-foreground: "oklch(0.85 0.005 260)"
    muted: "oklch(0.26 0.008 260)"
    muted-foreground: "oklch(0.67 0.012 260)"
    accent: "oklch(0.29 0.01 262)"
    accent-foreground: "oklch(0.92 0.01 262)"
    destructive: "oklch(0.62 0.17 25)"
    destructive-foreground: "oklch(0.97 0 0)"
    border: "oklch(0.31 0.009 260)"
    input: "oklch(0.34 0.01 260)"
    sidebar: "#000000"
    sidebar-foreground: "oklch(0.92 0.005 260)"
    sidebar-border: "oklch(0.31 0.009 260)"
    sidebar-accent: "oklch(0.26 0.01 262)"
    sidebar-accent-foreground: "oklch(0.92 0.01 262)"
    code: "oklch(0.25 0.008 260)"
    code-foreground: "oklch(0.87 0.01 260)"
    code-border: "oklch(0.31 0.009 260)"
    composer: "#252528"

# Both fonts self-hosted (woff2 from GitHub releases, no CDN): sans ships CJK, mono ships CJK + NF icons
typography:
  fonts:
    sans: "Sarasa UI SC"        # 400/500/600; Latin from Iosevka, CJK included
    mono: "Maple Mono NF CN"    # 400/700; CJK 2:1 alignment, renders Nerd Font icons
  scale:
    display-xl:
      fontFamily: Sarasa UI SC
      fontSize: 80px
      fontWeight: 600
      lineHeight: 1.05
      letterSpacing: -3.0px
    display-lg:
      fontFamily: Sarasa UI SC
      fontSize: 56px
      fontWeight: 600
      lineHeight: 1.10
      letterSpacing: -1.8px
    display-md:
      fontFamily: Sarasa UI SC
      fontSize: 40px
      fontWeight: 600
      lineHeight: 1.15
      letterSpacing: -1.0px
    headline:
      fontFamily: Sarasa UI SC
      fontSize: 28px
      fontWeight: 600
      lineHeight: 1.20
      letterSpacing: -0.6px
    card-title:
      fontFamily: Sarasa UI SC
      fontSize: 22px
      fontWeight: 500
      lineHeight: 1.25
      letterSpacing: -0.4px
    subhead:
      fontFamily: Sarasa UI SC
      fontSize: 20px
      fontWeight: 400
      lineHeight: 1.40
      letterSpacing: -0.2px
    body-lg:
      fontFamily: Sarasa UI SC
      fontSize: 18px
      fontWeight: 400
      lineHeight: 1.50
      letterSpacing: -0.1px
    body:
      fontFamily: Sarasa UI SC
      fontSize: 16px
      fontWeight: 400
      lineHeight: 1.50
      letterSpacing: -0.05px
    body-sm:
      fontFamily: Sarasa UI SC
      fontSize: 14px
      fontWeight: 400
      lineHeight: 1.50
      letterSpacing: 0
    caption:
      fontFamily: Sarasa UI SC
      fontSize: 12px
      fontWeight: 400
      lineHeight: 1.40
      letterSpacing: 0
    button:
      fontFamily: Sarasa UI SC
      fontSize: 14px
      fontWeight: 500
      lineHeight: 1.20
      letterSpacing: 0
    eyebrow:
      fontFamily: Sarasa UI SC
      fontSize: 13px
      fontWeight: 500
      lineHeight: 1.30
      letterSpacing: 0.4px
    mono:
      fontFamily: Maple Mono NF CN
      fontSize: 13px
      fontWeight: 400
      lineHeight: 1.50
      letterSpacing: 0
    mono-sm:
      fontFamily: Maple Mono NF CN
      fontSize: 12px
      fontWeight: 400
      lineHeight: 1.40
      letterSpacing: 0

rounded:
  xs: 4px
  sm: 6px
  md: 8px
  lg: 12px
  xl: 16px
  xxl: 24px
  pill: 9999px
  full: 9999px

spacing:
  xxs: 4px
  xs: 8px
  sm: 12px
  md: 16px
  lg: 24px
  xl: 32px
  xxl: 48px
  section: 96px

# Component recipes. Reference convention: colors without light/dark prefix are
# theme-adaptive semantic tokens; typography refs omit the scale prefix;
# hover/pressed states fold into comments, never separate tokens
components:
  # ---- primitives ----
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.primary-foreground}"
    typography: "{typography.button}"
    rounded: "{rounded.md}"
    padding: 8px 16px             # h-9; hover: primary/90; focus: 3px ring
  button-secondary:
    backgroundColor: "{colors.secondary}"
    textColor: "{colors.secondary-foreground}"
    typography: "{typography.button}"
    rounded: "{rounded.md}"
    padding: 8px 16px             # hover: secondary/80
  button-outline:
    backgroundColor: "{colors.background}"
    textColor: "{colors.foreground}"
    typography: "{typography.button}"
    rounded: "{rounded.md}"
    padding: 8px 16px
    border: "{colors.border}"     # hover: bg accent
  button-ghost:
    backgroundColor: transparent
    textColor: "{colors.foreground}"
    typography: "{typography.button}"
    rounded: "{rounded.md}"
    padding: 8px 16px             # hover: bg accent; icon buttons share this recipe
  button-destructive:
    backgroundColor: "{colors.destructive}"
    textColor: "{colors.destructive-foreground}"
    typography: "{typography.button}"
    rounded: "{rounded.md}"
    padding: 8px 16px             # hover: destructive/90
  text-input:
    backgroundColor: transparent  # hairline style: transparent bg + thin border (textarea identical)
    textColor: "{colors.foreground}"
    typography: "{typography.body-sm}"
    rounded: "{rounded.md}"
    padding: 8px 12px
    border: "{colors.input}"      # placeholder: muted-foreground; focus: ring
  card:
    backgroundColor: "{colors.card}"
    textColor: "{colors.card-foreground}"
    typography: "{typography.body-sm}"
    rounded: "{rounded.xl}"       # currently 14px, becomes 16px once code aligns with template radii
    padding: 24px
    border: "{colors.border}"
  popover:
    backgroundColor: "{colors.popover}"   # shared by dropdown-menu / select overlays
    textColor: "{colors.popover-foreground}"
    typography: "{typography.body-sm}"
    rounded: "{rounded.md}"
    padding: 4px
    border: "{colors.border}"     # only overlays may use shadow-md; everything else stays flat
  tooltip:
    backgroundColor: "{colors.foreground}"  # inverse-color mini overlay
    textColor: "{colors.background}"
    typography: "{typography.caption}"
    rounded: "{rounded.md}"
    padding: 6px 12px
  switch:
    backgroundColor: "{colors.input}"     # off state; on state: primary
    rounded: "{rounded.full}"
    height: 18px                          # 16px round thumb
  tabs:
    backgroundColor: "{colors.muted}"     # track; selected: background + shadow-xs
    textColor: "{colors.muted-foreground}"
    typography: "{typography.button}"
    rounded: "{rounded.lg}"
    padding: 4px                          # selected text: foreground
  # ---- product components ----
  sidebar:
    backgroundColor: "{colors.sidebar}"
    textColor: "{colors.sidebar-foreground}"
    typography: "{typography.body-sm}"
    width: 260px                          # group headers: caption + muted-foreground
  sidebar-session-row:
    backgroundColor: transparent  # hover: accent/60; active: sidebar-accent + weight 500
    textColor: "{colors.sidebar-foreground}"
    typography: "{typography.body-sm}"
    rounded: "{rounded.md}"
    padding: 6px 12px             # inline action buttons fade in on hover (rename/archive)
  message-bubble-user:
    backgroundColor: "{colors.muted}"
    textColor: "{colors.foreground}"
    typography: "{typography.body-sm}"
    rounded: "{rounded.xl}"       # right-aligned, max-width 75%
    padding: 10px 16px
  message-assistant:
    backgroundColor: transparent  # no bubble: 28px round inverse avatar + plain flow
    textColor: "{colors.foreground}"
    typography: "{typography.body-sm}"    # errors: destructive; source/stop notices: caption + muted-foreground
  composer:
    backgroundColor: "{colors.composer}"
    textColor: "{colors.foreground}"
    typography: "{typography.body-sm}"
    rounded: "{rounded.xl}"       # 16px
    padding: 12px
    border: "{colors.input}"      # 32px round inverse send button (bg-foreground); input area transparent, borderless
  model-switcher:
    backgroundColor: transparent  # ghost trigger + popover overlay
    textColor: "{colors.muted-foreground}"
    typography: "{typography.mono-sm}"    # model IDs in mono
    rounded: "{rounded.md}"
  code-block:
    backgroundColor: "{colors.code}"      # currently overlaid at 60% opacity
    textColor: "{colors.code-foreground}"
    typography: "{typography.mono}"
    rounded: "{rounded.md}"
    padding: 16px                 # inline code same family, padding 2px 4px
    border: "{colors.code-border}"
  banner-warning:
    backgroundColor: "{colors.warning}"   # TBD: pending-migration banner currently hard-coded amber-50/300/900
    textColor: "{colors.foreground}"
    typography: "{typography.body-sm}"
    rounded: "{rounded.md}"
    padding: 8px 12px
    border: "{colors.warning}"
  auth-card:
    backgroundColor: "{colors.card}"      # centered card on the login/signup page
    textColor: "{colors.card-foreground}"
    typography: "{typography.body-sm}"
    rounded: "{rounded.xl}"
    padding: 32px
    border: "{colors.border}"
---

## Overview

Lemma is a self-hosted AI chat workbench with three theme states (light / dark / system). The design system is **semantic-token-only**: components reference semantic names like `{colors.card}` and `{typography.button}`; raw values live exclusively in `theme.css`, with both theme sets registered in the colors block of this document.

Surfaces are organized by **role**, not ladder: `{colors.background}` main canvas, `{colors.sidebar}` sidebar zone, `{colors.composer}` input zone, `{colors.card}` / `{colors.popover}` panels and overlays, `{colors.muted}` / `{colors.secondary}` / `{colors.accent}` fills. Light mode is all solid color — near-white canvas #fdfbfd, faintly warm sidebar #fcf8fb, pure white composer. Dark mode is a **pure black sidebar #000000 plus a vertical gradient canvas** (bottom `{colors.canvas-from}` #0e0d0f → top `{colors.canvas-to}` #1c1e1b, fixed to the viewport), composer #252528. Hierarchy comes from surface roles + hairline borders; shadows are reserved for overlays.

The single chromatic accent is sky blue `{colors.primary}` #60b1ff — primary buttons, focus rings, link emphasis — topped with dark ink text `{colors.primary-foreground}` #0b1220 for contrast. The focus ring has no standalone value: it is derived from primary via `color-mix`; hovers use opacity modifiers (`primary/90`); swapping the primary cascades everywhere automatically. The only semantic color today is `{colors.destructive}`; warning / success are TBD (decided when the amber migration banner is adopted).

Fonts are dual self-hosted (woff2 from GitHub releases, no CDN): **Sarasa UI SC** (400/500/600) carries UI and body text — Latin from Iosevka, CJK included, identical rendering across platforms; **Maple Mono NF CN** (400/700) carries code — CJK 2:1 alignment, renders Nerd Font icons, so pasted terminal output no longer degrades to tofu boxes. The 13-step type scale is kept as-is, including negative display tracking (-3.0px @ 80px down to 0 at body).

The page rhythm is a **workbench, not a marketing narrative**: a 260px session sidebar on the left, the conversation flow in the middle (user messages right-aligned in `{colors.muted}` bubbles; assistant messages as inverse round avatar + plain flow), and the composer at the bottom (`{colors.composer}` panel + inverse round send button), with the content column capped at max-w-3xl. The language serves long reading and typing sessions, not presentation.

**Key Characteristics:**
- **Three-state theme** — solid light / gradient dark / follow system; semantic tokens defined once, valued per theme.
- **Single sky-blue accent** `{colors.primary}` #60b1ff — derived focus ring, opacity hovers, no second chromatic color.
- **Role-based surfaces** + hairline borders; components stay flat, shadows only on overlays.
- **Dual self-hosted fonts** — Sarasa UI SC + Maple Mono NF CN; CJK is a first-class citizen.
- **Workbench rhythm** — sidebar + conversation flow + composer; content column max-w-3xl.
- Radius ladder 4/6/8/12/16/24px — 8px for buttons and inputs, 16px for cards and bubbles.

## Colors

> Raw values live only in `theme.css`; this chapter explains each token's role and intent. Per-theme values are registered in the front matter `colors:` block.

### Brand & Accent
- **Sky Blue** (`{colors.primary}`): The single chromatic accent #60b1ff — primary buttons, focus rings, link emphasis. Shared by both themes.
- **On Primary** (`{colors.primary-foreground}`): Dark ink #0b1220 on the primary color, for contrast. Shared by both themes.
- **Ring** (`{colors.ring}`): Focus ring with no standalone value — derived from primary via `color-mix` at 40% opacity, follows the primary automatically.
- No hover/pressed tokens: always opacity modifiers (`primary/90`, `secondary/80`).

### Surface
- **Background** (`{colors.background}`): Main canvas. Solid #fdfbfd in light; #151615 in dark as the gradient's midpoint fallback.
- **Canvas From / To** (`{colors.canvas-from}` / `{colors.canvas-to}`): Dark-only — the two ends of the main-area vertical gradient (bottom #0e0d0f → top #1c1e1b), anchored to the viewport via `background-attachment: fixed` so inner panel scrolling never stretches it.
- **Sidebar** (`{colors.sidebar}`): Sidebar zone. Faintly warm #fcf8fb in light; pure black #000000 in dark.
- **Composer** (`{colors.composer}`): Input-area panel. #ffffff in light; #252528 in dark.
- **Card / Popover** (`{colors.card}` / `{colors.popover}`): Panels and overlays. Both pure white in light; both first-step charcoal in dark.
- **Secondary / Muted / Accent**: The fill trio — secondary button background, subdued fill (user bubble), hover-state background.
- **Code** (`{colors.code}` + `{colors.code-border}`): Code block background and border; blocks overlay the canvas at 60% opacity.

### Text
- **Foreground** (`{colors.foreground}`): All headlines and primary body text.
- **Muted Foreground** (`{colors.muted-foreground}`): Secondary text — captions, meta info, group headers, placeholders.
- Every surface ships a matching text token with the same suffix (`card-foreground`, `sidebar-foreground`, `code-foreground`, ...).
- **Two text levels are deliberate**: finer hierarchy is carried by font weight (400/500/600), not by additional grays.

### Border
- **Border** (`{colors.border}`): Default hairline on cards and dividers.
- **Input** (`{colors.input}`): Input border, one step lighter than border in dark.
- Zone variants: `sidebar-border`, `code-border`, paired with their surfaces.

### Semantic
- **Destructive** (`{colors.destructive}`): Delete and dangerous actions; one lightness step per theme.
- **Warning** (TBD): The pending-migration banner is currently hard-coded amber-50/300/900; per-theme values land when it is adopted.
- **Success** (TBD): Positive feedback such as a successful storage connection test.
- Overlay scrim: no dialog component yet; start at `black/60` when introduced, no standalone token.

## Typography

### Font Family

- **Sarasa UI SC** — self-hosted sans (OFL). Latin glyphs derive from Iosevka; CJK glyphs are bundled, so Chinese UI text renders identically on every platform. Weights: 400 / 500 / 600. Fallback stack: `ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, "PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", sans-serif`. Carries display-xl through eyebrow.
- **Maple Mono NF CN** — self-hosted mono (OFL). CJK aligns 2:1 with Latin; the Nerd Font patch renders icons in pasted terminal output instead of tofu boxes. Weights: 400 / 700. Fallback stack: `ui-monospace, "SF Mono", Menlo, Consolas, monospace`. Carries mono and mono-sm.

One sans family spans display to body; the family change is silent, hierarchy comes from weight and tracking.

### Hierarchy

| Token                     | Size | Weight | Line Height | Letter Spacing | Use                                        |
| ------------------------- | ---- | ------ | ----------- | -------------- | ------------------------------------------ |
| `{typography.display-xl}` | 80px | 600    | 1.05        | -3.0px         | Reserved: landing / marketing hero         |
| `{typography.display-lg}` | 56px | 600    | 1.10        | -1.8px         | Reserved: landing section openers          |
| `{typography.display-md}` | 40px | 600    | 1.15        | -1.0px         | Reserved: landing sub-sections             |
| `{typography.headline}`   | 28px | 600    | 1.20        | -0.6px         | Page-level titles                          |
| `{typography.card-title}` | 22px | 500    | 1.25        | -0.4px         | Card titles (auth card, settings panels)   |
| `{typography.subhead}`    | 20px | 400    | 1.40        | -0.2px         | Lead paragraphs (empty state)              |
| `{typography.body-lg}`    | 18px | 400    | 1.50        | -0.1px         | Emphasized body, section titles            |
| `{typography.body}`       | 16px | 400    | 1.50        | -0.05px        | Default body                               |
| `{typography.body-sm}`    | 14px | 400    | 1.50        | 0              | UI workhorse: lists, forms, chat body      |
| `{typography.caption}`    | 12px | 400    | 1.40        | 0              | Captions, meta, group headers              |
| `{typography.button}`     | 14px | 500    | 1.20        | 0              | All button and tab labels                  |
| `{typography.eyebrow}`    | 13px | 500    | 1.30        | 0.4px          | Uppercase taxonomy labels                  |
| `{typography.mono}`       | 13px | 400    | 1.50        | 0              | Code blocks                                |
| `{typography.mono-sm}`    | 12px | 400    | 1.40        | 0              | Model IDs, inline code                     |

### Principles

- **Negative tracking belongs to display sizes** — -3.0px at 80px tapering to 0 at body; nothing at body size or below gets negative tracking.
- **Single voice from display to body.** Display-xl at 600 → body at 400 — one family (Sarasa), hierarchy via weight.
- **Eyebrow uses positive tracking** (+0.4px) — contrast against the negative-tracked display marks the eyebrow as taxonomy.
- **Mono marks machine text** — code blocks, inline code, model IDs. Never for prose.

### Self-hosting

- Both families are downloaded from their GitHub releases (Sarasa-Gothic, maple-font), converted to woff2 when the release ships TTF, vendored under the project, and registered via `@font-face` in `theme.css`. OFL license files ship alongside.
- No CDN, no font-service dependency; loading uses `font-display: swap` so system fonts render first.
- Mono loads on demand — pages without code never fetch it.

## Layout

### Spacing System

- **Base unit**: 4px — identical to Tailwind's scale, so every token maps 1:1 to a utility (`{spacing.md}` 16px = `p-4`).
- **Tokens (front matter)**: `{spacing.xxs}` 4px · `{spacing.xs}` 8px · `{spacing.sm}` 12px · `{spacing.md}` 16px · `{spacing.lg}` 24px · `{spacing.xl}` 32px · `{spacing.xxl}` 48px · `{spacing.section}` 96px.
- Component conventions: buttons pad 8px 16px (h-9); form inputs 8px 12px; card interiors `{spacing.lg}` 24px; auth card `{spacing.xl}` 32px; composer `{spacing.sm}` 12px.
- `{spacing.section}` 96px is reserved for landing pages — currently unused in the app.

### Grid & Container

- **Workbench layout**: fixed 260px sidebar + flexible main area. No marketing card grids.
- The chat content column is capped at max-w-3xl and centered; the composer follows the same column.
- Settings pages use a two-pane layout: nav rail + detail panel.
- User bubbles cap at 75% width, right-aligned; assistant messages span the content column.

### Whitespace Philosophy

Density serves long reading and typing sessions. Separation comes from **surface roles and hairline borders**, not from shadows or large gaps. The composer anchors to the bottom with breathing room around it (`{spacing.lg}` page padding). Empty states center their content instead of filling the canvas.

## Elevation & Depth

| Level          | Treatment                                                        | Use                                       |
| -------------- | ---------------------------------------------------------------- | ----------------------------------------- |
| 0 (flat)       | No shadow, no border                                             | Body text, assistant messages, sidebar rows |
| 1 (panel)      | `{colors.card}` background, 1px `{colors.border}`                | Cards, settings panels, auth card         |
| 2 (fill shift) | `{colors.accent}` / `{colors.muted}` fill                        | Hovered rows, ghost buttons, selected tabs |
| 3 (overlay)    | `{colors.popover}` background, 1px `{colors.border}`, shadow-md  | Dropdown menus, selects, tooltips         |
| 4 (focus ring) | 3px `{colors.ring}`, derived from primary                        | Focused input, focused button             |

Depth is carried by surface roles + hairline borders. Components stay flat; drop shadows are granted to overlays only. The focus ring is the highest attention layer.

### Decorative Depth

- **The dark canvas gradient** (`{colors.canvas-from}` → `{colors.canvas-to}`) is the single atmospheric element — anchored to the viewport, calm, non-interactive.
- **Streaming cursor** — the pulsing caret on in-flight assistant messages is the only motion used as depth.
- No product screenshots, no edge highlights, no spotlight cards.

## Shapes

### Border Radius Scale

| Token            | Value  | Use                                                   |
| ---------------- | ------ | ----------------------------------------------------- |
| `{rounded.xs}`   | 4px    | Inline code, sidebar inline action buttons            |
| `{rounded.sm}`   | 6px    | Spare step — nothing assigned today                   |
| `{rounded.md}`   | 8px    | All buttons, form inputs, session rows, code blocks, dropdown items |
| `{rounded.lg}`   | 12px   | Tabs track, new-chat button                           |
| `{rounded.xl}`   | 16px   | Cards, user bubbles, composer                         |
| `{rounded.xxl}`  | 24px   | Reserved (landing banners)                            |
| `{rounded.pill}` | 9999px | Status pills                                          |
| `{rounded.full}` | 9999px | Avatars, switch, round icon buttons (send / stop)     |

Code currently derives radii from a 10px base (card at 14px); aligning the code to this scale is a pending change tracked outside this document.

### Iconography & Avatars

- Icons are **Lucide**: 16px (`size-4`) default, 14px (`size-3.5`) in dense contexts, 12px (`size-3`) only for sidebar inline actions.
- Avatars are round initials / symbols, never photos: the 28px assistant avatar (inverse `bg-foreground` + Sparkles icon) and the 28px sidebar user avatar (username initial).
- No photography, no product screenshots, no logo walls anywhere in the app.

## Components

### Buttons

**`button-primary`** — Lavender CTA. The default primary CTA across all pages.
- Background `{colors.primary}`, text `{colors.on-primary}`, type `{typography.button}`, padding 8px 14px, rounded `{rounded.md}`.
- Pressed state lives in `button-primary-pressed` (background shifts to `{colors.primary-focus}`).
- Hover state lives in `button-primary-hover` (background shifts to `{colors.primary-hover}` lighter lavender).

**`button-secondary`** — Charcoal button. Used for secondary CTAs ("Sign in", "Read changelog").
- Background `{colors.surface-1}`, text `{colors.ink}`, type `{typography.button}`, padding 8px 14px, rounded `{rounded.md}`. 1px `{colors.hairline}` border.

**`button-tertiary`** — Plain text button.
- Background `{colors.canvas}`, text `{colors.ink}`, type `{typography.button}`, rounded `{rounded.md}`, padding 8px 14px.

**`button-inverse`** — White-on-dark inverse CTA.
- Background `{colors.inverse-canvas}`, text `{colors.inverse-ink}`, type `{typography.button}`, rounded `{rounded.md}`, padding 8px 14px.

### Pricing Tabs

**`pricing-tab-default`** + **`pricing-tab-selected`** — Pill-toggle on `/pricing`.
- Default: `{colors.canvas}` background, `{colors.ink-subtle}` text, rounded `{rounded.pill}`, padding 6px 14px.
- Selected: `{colors.surface-2}` background, `{colors.ink}` text — selected = surface lift.

### Cards & Containers

**`pricing-card`** — Each tier on `/pricing`.
- Background `{colors.surface-1}`, text `{colors.ink}`, type `{typography.body}`, rounded `{rounded.lg}`, padding 24px. 1px `{colors.hairline}` border.

**`pricing-card-featured`** — Recommended tier — surface lift to surface-2.
- Background `{colors.surface-2}`, otherwise identical structure.

**`feature-card`** — Generic feature highlight tile.
- Background `{colors.surface-1}`, text `{colors.ink}`, type `{typography.body}`, rounded `{rounded.lg}`, padding 24px.

**`product-screenshot-card`** — The dominant card type — frames a high-fidelity Linear app UI screenshot.
- Background `{colors.surface-1}`, text `{colors.ink}`, type `{typography.body}`, rounded `{rounded.xl}`, padding 24px.

**`testimonial-card`** — Customer quote with avatar + name + role.
- Background `{colors.surface-1}`, text `{colors.ink}`, type `{typography.body-lg}`, rounded `{rounded.lg}`, padding 32px.

**`customer-logo-tile`** — Small tile in the customer marquee.
- Background `{colors.canvas}`, text `{colors.ink-subtle}`, type `{typography.caption}`, rounded `{rounded.xs}`, padding 16px.

**`cta-banner`** — Closing CTA panel near page bottom.
- Background `{colors.surface-1}`, text `{colors.ink}`, type `{typography.headline}`, rounded `{rounded.lg}`, padding 48px.

### Inputs & Forms

**`text-input`** + **`text-input-focused`** — Form fields on `/contact/sales` and signup overlays.
- Background `{colors.surface-1}`, text `{colors.ink}`, type `{typography.body}`, rounded `{rounded.md}`, padding 8px 12px.
- Focused state retains the same surface; the focus ring is a 2px `{colors.primary-focus}` outline at 50% opacity.

### Status & Build Page

**`changelog-row`** — Each row in `/build` (changelog page) listing version, date, and changes.
- Background `{colors.canvas}`, text `{colors.ink}`, type `{typography.body}`, rounded `{rounded.xs}`, padding 24px 0. 1px `{colors.hairline}` bottom rule.

**`status-badge`** — Small status pill.
- Background `{colors.surface-2}`, text `{colors.ink-muted}`, type `{typography.caption}`, rounded `{rounded.pill}`, padding 2px 8px.

### Navigation

**`top-nav`** — Sticky dark bar with the Linear wordmark left, primary nav links centered, and a `button-secondary` ("Sign in") + `button-primary` ("Get started") pair right.
- Background `{colors.canvas}`, text `{colors.ink}`, type `{typography.body-sm}`, height 56px.

### Footer

**`footer`** — Dense link grid on `{colors.canvas}` with the Linear wordmark left.
- Background `{colors.canvas}`, text `{colors.ink-subtle}`, type `{typography.caption}`, padding 64px 32px.

## Do's and Don'ts

### Do

- Reserve `{colors.canvas}` (#010102) as the system's anchor surface — the faint blue tint is intentional.
- Use `{colors.primary}` lavender ONLY for: brand mark, primary CTA, focus ring, link emphasis.
- Use the four-step surface ladder for hierarchy. Avoid skipping levels.
- Pair display weight 600 with body weight 400 — Linear resists 700+ display weights.
- Apply negative letter-spacing aggressively on display.
- Use product UI screenshots as the protagonist of every section.
- Compose CTAs as `{rounded.md}` 8px corners.

### Don't

- Don't ship a light-mode marketing page.
- Don't use lavender as a section background or card fill.
- Don't introduce a second chromatic accent (orange, pink, green for marketing).
- Don't add atmospheric gradients or spotlight cards.
- Don't pill-round CTAs.
- Don't use `#000000` true black as the canvas.
- Don't combine multiple bright accents in product screenshot mockups.

## Responsive Behavior

### Breakpoints

| Name       | Width  | Key Changes                                         |
| ---------- | ------ | --------------------------------------------------- |
| Desktop-XL | 1440px | Default desktop layout                              |
| Desktop    | 1280px | Card grid 3-up maintained                           |
| Tablet     | 1024px | Card grid 3-up → 2-up                               |
| Mobile-Lg  | 768px  | Pricing comparison becomes accordion; nav hamburger |
| Mobile     | 480px  | Single-column; display-xl scales 80px → ~36px       |

### Touch Targets

- CTAs hold ≥40px tap height across viewports.
- Pricing tab pills hold ≥36px tap height; touch viewports grow to ≥44px.
- Form inputs hold ≥44px tap target on touch.

### Collapsing Strategy

- **Top nav**: links collapse to hamburger below 768px.
- **Card grids**: 3-up → 2-up at 1024px → 1-up below 768px.
- **Pricing comparison**: per-tier accordion below 768px.
- **Display type**: `{typography.display-xl}` 80px scales toward `{typography.display-md}` 40px on mobile.

### Image Behavior

- Product UI screenshots maintain aspect ratio and never crop.
- Customer logos in the marquee may collapse from 6-up to 3-up below 768px.

## Iteration Guide

1. Focus on ONE component at a time and reference it by its `components:` token name.
2. When introducing a section, decide first which surface lift it lives on.
3. Default body to `{typography.body}` at weight 400.
4. Run `npx @google/design.md lint DESIGN.md` after edits.
5. Add new variants as separate component entries.
6. Treat lavender as scarce: brand mark, primary CTA, focus, link emphasis.
7. Lead every section with a product UI screenshot.

## Known Gaps

- The four-step surface ladder values are extracted directly from Linear's `--color-bg-level-3`, `--color-line-tint`, etc. CSS variables; they are Linear's canonical surface spec.
- Form-field error and validation styling is not visible on the inspected pages.
- Light mode is not documented because the marketing site does not ship a light theme.
- Linear's actual product UI uses a richer color-tag palette (red, orange, yellow, green, blue, purple) for issue priorities and project labels — those colors live in the in-product surfaces shown in mockups.
- The custom display, text, and mono families are proprietary; an open-source substitute is acceptable.
