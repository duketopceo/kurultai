---
title: "Plan: Heavier Science Theme & Logo Integration"
date: 2026-07-25
status: proposed
author: Antigravity
epic: #77
requirements:
  - "Transition the landing page and brain explorer to a heavier science theme (neural synapses, bioinformatics, bio-tech panels)."
  - "Integrate the generated Kurultai logo in place of placeholder symbols/text."
  - "Refine style aesthetics following no-slop design directives."
---

# Plan: Heavier Science Theme & Logo Integration

Update the frontend presentation layer of Kurultai to lean heavily into biological and neural science aesthetics. This replaces generic SaaS structures with dense biotech panels, cellular network animations, and standardizes branding around the generated neural K-logo.

## Scoping Synthesis

* **Stated:** Update landing page and brain explorer with a heavier science theme and the generated logo.
* **Inferred:** Transition canvas animation from standard particles to a neural synapse/axon electrical impulse model; restyle cards as high-density biometric panel monitors with monospaced data readouts.
* **Out of scope:** Modifying backend Rust CLI or SQLite query engine schemas.

---

## 1. Technical Decisions & Style Mapping

### 1.1 Science Palette & Materiality
* **Surfaces:** Cosmic Obsidian dark-mode baseline. Deep space `#090d16` canvas backgrounds with glowing bioluminescent backdrops.
* **Bioluminescent Accents:**
  * Primary: Phosphor Green/Teal (`#5eead4` / `#10b981`) representing active axon terminals.
  * Secondary: Synaptic Violet (`#a78bfa`) representing neurotransmitter junctions.
  * Links/Bridges: Electric Cobalt Blue (`#3b82f6`) representing transmission channels.
* **Biometric Borders:** Custom `1px solid rgba(16, 185, 129, 0.12)` to resemble medical instrument monitors.

### 1.2 Layout Density & Bento Grid
* Increase `VISUAL_DENSITY` dial to `7` (dense biotech telemetry panel style).
* Group technical metrics in `brain.html` using thin green scope-lines with oscilloscope crosshairs (`+`) in the corners.

---

## 2. Implementation Units

### Unit 1: Logo Asset Deployment & Navbar Integration
* **Files:** `website/index.html`, `website/brain.html`
* **Changes:**
  * Replace inline SVGs in header logo blocks with `<img src="kurultai_logo.jpg">` sized to `24px` height.
  * Replace the hero section's simple tag badge with a glowing circular monogram emblem.

### Unit 2: Heavier Science Canvas Animation
* **Files:** `website/index.js`
* **Changes:**
  * Update the background `<canvas>` animation to simulate biological synaptic transmission.
  * Render circular "cell bodies" (neurons) that branch axon connections to nearby nodes, with electrical pulses (electrons) traveling down the lines in waves.

### Unit 3: Biometric UI & High-Density Panels
* **Files:** `website/index.css`, `website/brain.html`
* **Changes:**
  * Restyle all `.feature-card` and `.glass-panel` components to use bioluminescent green borders (`rgba(16, 185, 129, 0.15)`) and dark green/teal drop-shadow glows.
  * Add scope crosshair indicators `::before` / `::after` on database panels.
  * Update typography sizes to favor `'Share Tech Mono'` on tables, databases, and stats readouts.

---

## 3. Verification & Test Scenarios

### Scenario 1: Desktop Viewport Fit Check
* Load both `index.html` and `brain.html` at `1920x1080` resolution.
* Verify the logo displays cleanly without stretching and headers remain on a single line.

### Scenario 2: WebGL/Canvas Animation Performance
* Verify the synapse animation runs at a consistent 60fps on mobile and desktop viewports.
* Check that reduced-motion query disables canvas physics and collapses to a static background.
