document.addEventListener("DOMContentLoaded", () => {

    // ── 1. Theme toggle ──────────────────────────────────────────
    const themeToggle = document.getElementById("theme-toggle");
    function applyTheme(theme) {
        document.documentElement.dataset.theme = theme;
        try {
            localStorage.setItem("kurultai-theme", theme);
        } catch (_) { /* private mode / blocked storage */ }
        const isLight = theme === "light";
        if (themeToggle) {
            themeToggle.setAttribute("aria-pressed", String(isLight));
            themeToggle.setAttribute("aria-label", `Switch to ${isLight ? "dark" : "light"} theme`);
        }
    }
    // Sync button state with the theme already applied by the inline script
    const currentTheme = document.documentElement.dataset.theme || "dark";
    applyTheme(currentTheme);
    if (themeToggle) {
        themeToggle.addEventListener("click", () => {
            const next = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
            applyTheme(next);
        });
    }

    // ── 2. Copy-to-clipboard ─────────────────────────────────────
    document.querySelectorAll(".btn-copy").forEach(button => {
        button.addEventListener("click", (e) => {
            e.preventDefault();
            const targetId = button.getAttribute("data-target");
            const el = document.getElementById(targetId);
            if (!el) return;
            const textToCopy = el.textContent.replace(/^\s*❯\s*/, "").trim();
            navigator.clipboard.writeText(textToCopy).then(() => {
                const originalText = button.innerHTML;
                button.innerHTML = "✓ Copied";
                button.style.color = "var(--electric-strong)";
                setTimeout(() => {
                    button.innerHTML = originalText;
                    button.style.color = "";
                }, 2000);
            });
        });
    });

    // ── 3. Simulated Terminal Typing Effect ───────────────────────
    const terminalInput  = document.getElementById("terminal-input");
    const terminalOutput = document.getElementById("terminal-output");

    const steps = [
        {
            input: "kurultai init --docs",
            output: "Config: ~/.config/kurultai/config.toml\nDocs folder: ~/Documents/kurultai\n  Starter note: welcome.md (tagged so it is searchable)\n\nMCP wired: ~/.cursor/mcp.json\nRestart the agent(s) to load the kurultai MCP server.\n\nWithout an API key (FTS-only — works now):\n  kurultai index --full\n  kurultai daemon --port 8421\n  # Brain UI → http://127.0.0.1:8421/ui/",
            delay: 2200
        },
        {
            input: "kurultai status",
            output: "Kurultai status\n  Environment: dev\n  Storage: ~/.local/share/kurultai/dev/store.db\n  Embedder: openai/text-embedding-3-large (3072-dim)\n  Synthesizer: openrouter\n  Atoms:   12\n  Sources:\n    - test-vault [enabled]",
            delay: 2600
        },
        {
            input: 'kurultai search "festina lente"',
            output: "[0.033] test-vault — agent guidelines — Core Mottos\n  - **Festina lente**: Make haste slowly. Move intentionally, write robust tests.\n[0.016] test-vault — lorem ipsum\n  - Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
            delay: 3000
        }
    ];

    let currentStep = 0;

    function runTerminal() {
        if (!terminalInput || !terminalOutput) return;
        if (currentStep >= steps.length) currentStep = 0;

        const step = steps[currentStep];
        terminalInput.textContent = "";
        terminalOutput.textContent = "";

        let charIndex = 0;
        function typeChar() {
            if (charIndex < step.input.length) {
                terminalInput.textContent += step.input[charIndex];
                charIndex++;
                setTimeout(typeChar, 48 + Math.random() * 36);
            } else {
                setTimeout(() => {
                    terminalOutput.textContent = step.output;
                    currentStep++;
                    setTimeout(runTerminal, step.delay);
                }, 380);
            }
        }
        setTimeout(typeChar, 900);
    }

    if (terminalInput && terminalOutput) runTerminal();

    // ── 4. Neural Canvas Particle Animation ──────────────────────
    const canvas = document.getElementById("neural-canvas");
    if (!canvas) return;

    const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reducedMotion) { canvas.style.display = "none"; return; }

    const ctx = canvas.getContext("2d");
    let width  = (canvas.width  = window.innerWidth);
    let height = (canvas.height = window.innerHeight);

    const mouse = { x: null, y: null, radius: 140 };
    window.addEventListener("mousemove", (e) => { mouse.x = e.x; mouse.y = e.y; });
    window.addEventListener("mouseleave", () => { mouse.x = null; mouse.y = null; });

    class Particle {
        constructor() { this.reset(); }
        reset() {
            this.x     = Math.random() * width;
            this.y     = Math.random() * height;
            this.vx    = (Math.random() - 0.5) * 0.38;
            this.vy    = (Math.random() - 0.5) * 0.38;
            this.r     = Math.random() * 1.8 + 0.7;
            // electric white or purple only
            this.isPurple = Math.random() > 0.5;
            this.alpha = Math.random() * 0.45 + 0.18;
        }
        color(a) {
            return this.isPurple
                ? `rgba(168, 85, 247, ${a})`
                : `rgba(255, 255, 255, ${a})`;
        }
        draw() {
            ctx.beginPath();
            ctx.arc(this.x, this.y, this.r, 0, Math.PI * 2);
            ctx.fillStyle = this.color(this.alpha);
            ctx.shadowBlur  = this.r * 3;
            ctx.shadowColor = this.color(0.7);
            ctx.fill();
            ctx.shadowBlur  = 0;
        }
        update() {
            this.x += this.vx;
            this.y += this.vy;
            if (this.x < 0 || this.x > width)  this.vx *= -1;
            if (this.y < 0 || this.y > height) this.vy *= -1;
            if (mouse.x != null) {
                const dx   = this.x - mouse.x;
                const dy   = this.y - mouse.y;
                const dist = Math.hypot(dx, dy);
                if (dist < mouse.radius && dist > 0) {
                    const force = (mouse.radius - dist) / mouse.radius;
                    this.x += (dx / dist) * force * 1.8;
                    this.y += (dy / dist) * force * 1.8;
                }
            }
        }
    }

    const numParticles = Math.min(90, Math.floor((width * height) / 16000));
    const particles    = Array.from({ length: numParticles }, () => new Particle());

    window.addEventListener("resize", () => {
        width  = canvas.width  = window.innerWidth;
        height = canvas.height = window.innerHeight;
        particles.forEach((p) => {
            p.x = Math.min(Math.max(p.x, 0), width);
            p.y = Math.min(Math.max(p.y, 0), height);
        });
    });

    function drawLines() {
        for (let i = 0; i < particles.length; i++) {
            for (let j = i + 1; j < particles.length; j++) {
                const dx   = particles[i].x - particles[j].x;
                const dy   = particles[i].y - particles[j].y;
                const dist = Math.hypot(dx, dy);
                if (dist < 110) {
                    const a = (1 - dist / 110) * 0.13;
                    const grad = ctx.createLinearGradient(
                        particles[i].x, particles[i].y,
                        particles[j].x, particles[j].y
                    );
                    grad.addColorStop(0, particles[i].color(a));
                    grad.addColorStop(1, particles[j].color(a));
                    ctx.beginPath();
                    ctx.moveTo(particles[i].x, particles[i].y);
                    ctx.lineTo(particles[j].x, particles[j].y);
                    ctx.strokeStyle = grad;
                    ctx.lineWidth   = 0.5;
                    ctx.stroke();
                }
            }
        }
    }

    function animate() {
        ctx.clearRect(0, 0, width, height);
        particles.forEach((p) => { p.update(); p.draw(); });
        drawLines();
        requestAnimationFrame(animate);
    }

    animate();
});
