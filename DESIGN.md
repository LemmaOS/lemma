---
version: alpha
name: Lemma
description: "Lemma's design system: a three-state (light / dark / system) self-hosted AI chat workbench. Sky blue #60b1ff is the single chromatic accent, with the focus ring derived from it and hovers expressed as opacity. Surfaces are role-based (canvas / sidebar / composer / card / popover) with hairline borders; shadows belong to overlays only. Dark mode pairs a pure-black sidebar with a viewport-fixed vertical gradient canvas. Fonts are self-hosted: Sarasa UI SC for UI text (CJK included) and Maple Mono NF CN for code (Nerd Font icons render). The rhythm is a workbench — 260px session sidebar, centered max-w-3xl conversation column, bottom composer."

# Canonical flat schema: unprefixed tokens = light theme (mirrors :root in theme.css),
# dark-* = dark override (mirrors [data-theme="dark"]). theme.css is the source of truth.
colors:
  primary: "#60b1ff"
  primary-foreground: "#0b1220"
  ring: "#60b1ff66"            # derived: color-mix(in srgb, primary 40%, transparent)
  warning: "#b45309"           # strong: banner text / icon
  warning-soft: "#fffbeb"      # banner background
  warning-border: "#fcd34d"
  success: "#27a644"
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
  dark-canvas-from: "#0e0d0f"
  dark-canvas-to: "#1c1e1b"
  dark-background: "#151615"
  dark-foreground: "oklch(0.92 0.005 260)"
  dark-card: "oklch(0.24 0.008 260)"
  dark-card-foreground: "oklch(0.92 0.005 260)"
  dark-popover: "oklch(0.24 0.008 260)"
  dark-popover-foreground: "oklch(0.92 0.005 260)"
  dark-secondary: "oklch(0.27 0.009 260)"
  dark-secondary-foreground: "oklch(0.85 0.005 260)"
  dark-muted: "oklch(0.26 0.008 260)"
  dark-muted-foreground: "oklch(0.67 0.012 260)"
  dark-accent: "oklch(0.29 0.01 262)"
  dark-accent-foreground: "oklch(0.92 0.01 262)"
  dark-destructive: "oklch(0.62 0.17 25)"
  dark-destructive-foreground: "oklch(0.97 0 0)"
  dark-border: "oklch(0.31 0.009 260)"
  dark-input: "oklch(0.34 0.01 260)"
  dark-sidebar: "#000000"
  dark-sidebar-foreground: "oklch(0.92 0.005 260)"
  dark-sidebar-border: "oklch(0.31 0.009 260)"
  dark-sidebar-accent: "oklch(0.26 0.01 262)"
  dark-sidebar-accent-foreground: "oklch(0.92 0.01 262)"
  dark-code: "oklch(0.25 0.008 260)"
  dark-code-foreground: "oklch(0.87 0.01 260)"
  dark-code-border: "oklch(0.31 0.009 260)"
  dark-composer: "#252528"
  dark-warning: "#fbbf24"
  dark-warning-soft: "#451a03"
  dark-warning-border: "#92400e"
  dark-success: "#3fb950"

# Font metadata (weights, fallbacks, self-hosting) lives in the Typography chapter
typography:
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
    lineHeight: 1.60
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

# Component recipes. Reference convention: unprefixed color tokens are
# theme-adaptive (dark-* holds the dark override); states (hover/pressed)
# fold into comments, never separate tokens
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
    backgroundColor: "{colors.warning-soft}"
    textColor: "{colors.warning}"
    typography: "{typography.body-sm}"
    rounded: "{rounded.md}"
    padding: 8px 12px
    border: "{colors.warning-border}"
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

Surfaces are organized by **role**, not ladder: `{colors.background}` main canvas, `{colors.sidebar}` sidebar zone, `{colors.composer}` input zone, `{colors.card}` / `{colors.popover}` panels and overlays, `{colors.muted}` / `{colors.secondary}` / `{colors.accent}` fills. Light mode is all solid color — near-white canvas #fdfbfd, faintly warm sidebar #fcf8fb, pure white composer. Dark mode is a **pure black sidebar #000000 plus a vertical gradient canvas** (bottom `{colors.dark-canvas-from}` #0e0d0f → top `{colors.dark-canvas-to}` #1c1e1b, fixed to the viewport), composer #252528. Hierarchy comes from surface roles + hairline borders; shadows are reserved for overlays.

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

> Raw values live only in `theme.css`; this chapter explains each token's role and intent. Light values are unprefixed, dark overrides carry the `dark-` prefix — mirroring `:root` / `[data-theme="dark"]` in `theme.css`.

### Brand & Accent
- **Sky Blue** (`{colors.primary}`): The single chromatic accent #60b1ff — primary buttons, focus rings, link emphasis. Shared by both themes.
- **On Primary** (`{colors.primary-foreground}`): Dark ink #0b1220 on the primary color, for contrast. Shared by both themes.
- **Ring** (`{colors.ring}`): Focus ring with no standalone value — derived from primary via `color-mix` at 40% opacity, follows the primary automatically.
- No hover/pressed tokens: always opacity modifiers (`primary/90`, `secondary/80`).

### Surface
- **Background** (`{colors.background}`): Main canvas. Solid #fdfbfd in light; #151615 in dark as the gradient's midpoint fallback.
- **Canvas From / To** (`{colors.dark-canvas-from}` / `{colors.dark-canvas-to}`): Dark-only — the two ends of the main-area vertical gradient (bottom #0e0d0f → top #1c1e1b), anchored to the viewport via `background-attachment: fixed` so inner panel scrolling never stretches it.
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
- **Warning** (`{colors.warning}` / `{colors.warning-soft}` / `{colors.warning-border}`): Banner trio — strong text, soft background, border. Adopted from the amber palette the migration banner shipped with.
- **Success** (`{colors.success}`): Positive feedback — storage test "Connected", migration finished.
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
| `{typography.body-sm}`    | 14px | 400    | 1.60        | 0              | UI workhorse: lists, forms, chat body      |
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

- **The dark canvas gradient** (`{colors.dark-canvas-from}` → `{colors.dark-canvas-to}`) is the single atmospheric element — anchored to the viewport, calm, non-interactive.
- **Streaming cursor** — the pulsing caret on in-flight assistant messages is the only motion used as depth.
- No product screenshots, no edge highlights, no spotlight cards.

## Shapes

### Border Radius Scale

| Token            | Value  | Use                                                   |
| ---------------- | ------ | ----------------------------------------------------- |
| `{rounded.xs}`   | 4px    | Inline code, sidebar inline action buttons            |
| `{rounded.sm}`   | 6px    | Spare step — nothing assigned today                   |
| `{rounded.md}`   | 8px    | All buttons, form inputs, session rows, code blocks, dropdown items |
| `{rounded.lg}`   | 12px   | Tabs track, new-chat button, page canvas panels             |
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

**`button-primary`** — Sky-blue CTA. Used sparingly: sign in, save changes, send-level actions.
- Background `{colors.primary}`, text `{colors.primary-foreground}`, type `{typography.button}`, padding 8px 16px, rounded `{rounded.md}`.
- Hover is an opacity modifier (`primary/90`); focus is the derived 3px `{colors.ring}`.

**`button-secondary`** — Quiet filled button for secondary actions.
- Background `{colors.secondary}`, text `{colors.secondary-foreground}`; hover `secondary/80`.

**`button-outline`** — Bordered button on the page canvas.
- Background `{colors.background}`, 1px `{colors.border}` border; hover fills `{colors.accent}`.

**`button-ghost`** — Borderless action for dense toolbars and icon buttons.
- Transparent; hover fills `{colors.accent}`. Icon buttons (copy, regenerate, collapse) are the same recipe, square or round.

**`button-destructive`** — Danger actions (delete provider, delete storage).
- Background `{colors.destructive}`, text `{colors.destructive-foreground}`; hover `destructive/90`.

### Forms

**`text-input`** — Hairline field: transparent background, 1px `{colors.input}` border. Textareas share the recipe.
- Type `{typography.body-sm}`, rounded `{rounded.md}`, padding 8px 12px; placeholder in `{colors.muted-foreground}`.
- Focus keeps the surface; the 3px `{colors.ring}` ring carries the state — no separate focus color exists.

### Overlays

**`popover`** — Floating panel shared by dropdown menus and selects.
- Background `{colors.popover}`, 1px `{colors.border}`, rounded `{rounded.md}`, padding 4px.
- The only component family allowed a drop shadow (shadow-md); everything else stays flat.

**`tooltip`** — Inverse mini overlay.
- Background `{colors.foreground}`, text `{colors.background}`, type `{typography.caption}`, padding 6px 12px.

### Toggles

**`switch`** — Round-thumb toggle.
- Off: `{colors.input}`; on: `{colors.primary}`. Rounded `{rounded.full}`, 18px tall with a 16px thumb.

**`tabs`** — Segmented control (settings groups).
- Track `{colors.muted}` with `{colors.muted-foreground}` labels; the selected segment lifts to `{colors.background}` with shadow-xs and `{colors.foreground}` text. Selected = lift, not color.

### Sidebar

**`sidebar`** — The session rail, a dedicated surface zone.
- Background `{colors.sidebar}`, text `{colors.sidebar-foreground}`, width 260px; group headers in `{typography.caption}` + `{colors.muted-foreground}`.

**`sidebar-session-row`** — One conversation in the list.
- Transparent, rounded `{rounded.md}`, padding 6px 12px; hover tints `accent/60`; the active row fills `{colors.sidebar-accent}` and bumps to weight 500.
- Inline actions (rename / archive) fade in on hover — the row stays text-pure at rest.

### Messages

**`message-bubble-user`** — Right-aligned bubble, capped at 75% width.
- Background `{colors.muted}`, type `{typography.body-sm}`, rounded `{rounded.xl}`, padding 10px 16px.

**`message-assistant`** — Deliberately bubble-less: a 28px round inverse avatar (Sparkles) followed by plain-flowing markdown.
- Long answers read as documents, not chat frames. Errors render in `{colors.destructive}`; source and stop notices in `{typography.caption}` + `{colors.muted-foreground}`; action buttons are ghost icons.

### Composer

**`composer`** — The input panel, its own surface token.
- Background `{colors.composer}`, 1px `{colors.input}` border, rounded `{rounded.xl}`, padding 12px; the textarea inside is transparent and borderless.
- The send button is a 32px round **inverse** button (`{colors.foreground}` on `{colors.background}`), not primary — blue is reserved for system-level emphasis.

**`model-switcher`** — Ghost trigger + popover overlay.
- Model IDs render in `{typography.mono-sm}` — machine text in machine type.

### Code

**`code-block`** — Fenced code in chat.
- Background `{colors.code}` overlaid at 60% opacity, 1px `{colors.code-border}`, type `{typography.mono}`, padding 16px. Inline code uses the same family at 2px 4px padding.

### Feedback

**`banner-warning`** — Inline warning banner (pending migration notice).
- Background/border `{colors.warning}` (TBD — currently hard-coded amber-50/300/900), type `{typography.body-sm}`, rounded `{rounded.md}`, padding 8px 12px.

### Auth

**`auth-card`** — The centered login / signup card.
- Background `{colors.card}`, 1px `{colors.border}`, rounded `{rounded.xl}`, padding 32px.

## Do's and Don'ts

### Do

- Reference **semantic tokens only** — raw values live exclusively in `theme.css`.
- Reserve `{colors.primary}` sky blue for system-level emphasis: primary buttons, focus ring, link emphasis.
- Use **surface roles** for hierarchy — canvas, card, popover, plus the dedicated sidebar and composer zones.
- Pair display weight 600 with body weight 400 — resist 700+ display weights.
- Apply negative letter-spacing on display sizes only.
- Derive the focus ring from primary (`color-mix`); express hover as opacity modifiers — swapping the primary cascades everywhere automatically.
- Compose buttons and inputs at `{rounded.md}` 8px corners; reserve `{rounded.full}` for avatars, switch, and round icon buttons (send / stop).

### Don't

- Don't hard-code raw color values in components — the amber migration banner is the known violation, pending adoption of `{colors.warning}`.
- Don't introduce a second chromatic accent.
- Don't add drop shadows outside overlays (popover / dropdown / tooltip).
- Don't use display-size type inside the app — display-xl through display-md are reserved for landing pages.
- Don't load fonts from a CDN — both families are self-hosted woff2.
- Don't add more text grays — finer hierarchy comes from weight (400 / 500 / 600), not new shades.
- Don't pill-round rectangular CTAs.

## Responsive Behavior

### Breakpoints

| Name    | Width   | Key Changes                                                  |
| ------- | ------- | ------------------------------------------------------------ |
| Desktop | ≥1024px | Default workbench: 260px sidebar + main canvas               |
| Tablet  | 768px   | Content column shrinks; sidebar manually collapsible         |
| Mobile  | <768px  | Not yet adapted (see Known Gaps)                             |

### Touch Targets

- Buttons are h-9 (36px), icon buttons 28–36px — below the 44px touch guidance. Desktop-first; revisit when mobile ships.
- Form inputs are h-9 (36px).

### Collapsing Strategy

- **Sidebar**: manual collapse, leaving an edge expand button; the session list scrolls independently.
- **Chat column**: capped at max-w-3xl and centered; user bubbles keep the 75% width cap.
- **Display type**: display sizes are reserved for landing pages, so no in-app scaling rules exist.

### Image Behavior

No images in the app — icons are Lucide, avatars are initial/symbol circles. Nothing to scale.

## Iteration Guide

1. Focus on ONE component at a time and reference it by its `components:` token name.
2. Introducing a surface: pick an existing role (`background` / `card` / `popover` / `sidebar` / `composer`); do not invent new ones casually.
3. Default UI text to `{typography.body-sm}` at weight 400; reach hierarchy through weight (500 / 600) before new sizes.
4. Run `npx @google/design.md lint DESIGN.md` after edits — 0 errors required. Warning baseline (by design): unreferenced `dark-*` / display tokens, the `border` sub-token, transparent-background contrast false positives.
5. Add new component variants as separate entries; interaction states (hover / pressed) fold into comments.
6. Raw values live only in `theme.css`; this document mirrors the code, never the reverse.
7. Treat sky blue as scarce: primary buttons, focus ring, link emphasis.

## Known Gaps

- Form-field validation styling (`aria-invalid` rings exist in the primitives) is not designed into any flow yet.
- Mobile layout (<768px) is not adapted — sidebar and composer need a dedicated treatment.
- Open follow-ups from the color refactor: text shade calibration, chat bubble / avatar review, whether the login page joins the dark gradient.
- The mono font is not subset — the first code block on a slow network may swap in late.
