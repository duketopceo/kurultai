document.addEventListener("DOMContentLoaded", () => {
    // 1. Copy-to-clipboard functionality
    const copyButtons = document.querySelectorAll(".btn-copy");
    copyButtons.forEach(button => {
        button.addEventListener("click", (e) => {
            e.preventDefault();
            const targetId = button.getAttribute("data-target");
            const textToCopy = document.getElementById(targetId).textContent.replace(/^\s*❯\s*/, '').trim();
            
            navigator.clipboard.writeText(textToCopy).then(() => {
                const originalText = button.innerHTML;
                button.innerHTML = "✓ Copied";
                button.style.color = "#c084fc";
                setTimeout(() => {
                    button.innerHTML = originalText;
                    button.style.color = "";
                }, 2000);
            });
        });
    });

    // 2. Simulated Terminal Typing Effect
    const terminalInput = document.getElementById("terminal-input");
    const terminalOutput = document.getElementById("terminal-output");
    
    const steps = [
        {
            input: "kurultai init --agent cursor",
            output: "Config: ~/.config/kurultai/config.toml\nMCP wired: ~/Library/Application Support/Cursor/User/globalStorage/cursor-mcp/config.json\n\nRestart Cursor to load the kurultai MCP server.",
            delay: 2000
        },
        {
            input: "kurultai status",
            output: "Kurultai status\n  Environment: dev\n  Storage: ~/.local/share/kurultai/dev/store.db\n  Embedder: openai/text-embedding-3-large (3072-dim)\n  Synthesizer: openrouter\n  Atoms:   12\n  Sources:\n    - test-vault [enabled]",
            delay: 2500
        },
        {
            input: "kurultai search \"festina lente\"",
            output: "[0.033] test-vault — agent guidelines — Core Mottos\n  - **Festina lente**: Make haste slowly. Move intentionally, write robust tests.\n[0.016] test-vault — lorem ipsum\n  - Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
            delay: 3000
        }
    ];

    let currentStep = 0;

    function runTerminal() {
        if (!terminalInput || !terminalOutput) return;
        if (currentStep >= steps.length) {
            currentStep = 0; // Loop forever
        }

        const step = steps[currentStep];
        terminalInput.textContent = "";
        terminalOutput.textContent = "";

        let charIndex = 0;
        function typeChar() {
            if (charIndex < step.input.length) {
                terminalInput.textContent += step.input[charIndex];
                charIndex++;
                setTimeout(typeChar, 50 + Math.random() * 40); // Natural typing variation
            } else {
                setTimeout(() => {
                    terminalOutput.textContent = step.output;
                    currentStep++;
                    setTimeout(runTerminal, step.delay);
                }, 400);
            }
        }

        setTimeout(typeChar, 800);
    }

    if (terminalInput && terminalOutput) {
        runTerminal();
    }

    // 3. Neurons & Electrons Canvas Particle Animation
    const canvas = document.getElementById("neural-canvas");
    if (canvas) {
        const ctx = canvas.getContext("2d");
        let particles = [];
        let width = (canvas.width = window.innerWidth);
        let height = (canvas.height = window.innerHeight);

        window.addEventListener("resize", () => {
            width = canvas.width = window.innerWidth;
            height = canvas.height = window.innerHeight;
        });

        // Mouse tracking
        const mouse = { x: null, y: null, radius: 150 };
        window.addEventListener("mousemove", (e) => {
            mouse.x = e.x;
            mouse.y = e.y;
        });
        window.addEventListener("mouseleave", () => {
            mouse.x = null;
            mouse.y = null;
        });

        class Particle {
            constructor() {
                this.x = Math.random() * width;
                this.y = Math.random() * height;
                this.vx = (Math.random() - 0.5) * 0.4;
                this.vy = (Math.random() - 0.5) * 0.4;
                this.radius = Math.random() * 2 + 1;
                // Electrons glow properties — white + purple only
                this.color = Math.random() > 0.5 ? "rgba(255, 255, 255, " : "rgba(168, 85, 247, ";
                this.alpha = Math.random() * 0.5 + 0.2;
            }

            draw() {
                ctx.beginPath();
                ctx.arc(this.x, this.y, this.radius, 0, Math.PI * 2);
                ctx.fillStyle = this.color + this.alpha + ")";
                ctx.shadowBlur = this.radius * 3;
                ctx.shadowColor = this.color + "0.8)";
                ctx.fill();
                ctx.shadowBlur = 0; // Reset shadow for line performance
            }

            update() {
                this.x += this.vx;
                this.y += this.vy;

                // Wall collisions
                if (this.x < 0 || this.x > width) this.vx *= -1;
                if (this.y < 0 || this.y > height) this.vy *= -1;

                // Mouse interaction (repelled slightly like charge)
                if (mouse.x != null && mouse.y != null) {
                    let dx = this.x - mouse.x;
                    let dy = this.y - mouse.y;
                    let dist = Math.hypot(dx, dy);
                    if (dist < mouse.radius) {
                        let force = (mouse.radius - dist) / mouse.radius;
                        this.x += (dx / dist) * force * 2;
                        this.y += (dy / dist) * force * 2;
                    }
                }
            }
        }

        // Initialize synapses
        const numParticles = Math.min(100, Math.floor((width * height) / 15000));
        for (let i = 0; i < numParticles; i++) {
            particles.push(new Particle());
        }

        function drawLines() {
            for (let i = 0; i < particles.length; i++) {
                for (let j = i + 1; j < particles.length; j++) {
                    let dx = particles[i].x - particles[j].x;
                    let dy = particles[i].y - particles[j].y;
                    let dist = Math.hypot(dx, dy);

                    if (dist < 120) {
                        let alpha = (1 - dist / 120) * 0.15;
                        ctx.beginPath();
                        ctx.moveTo(particles[i].x, particles[i].y);
                        ctx.lineTo(particles[j].x, particles[j].y);
                        // Gradient line between cyan and purple
                        let grad = ctx.createLinearGradient(particles[i].x, particles[i].y, particles[j].x, particles[j].y);
                        grad.addColorStop(0, particles[i].color + alpha + ")");
                        grad.addColorStop(1, particles[j].color + alpha + ")");
                        ctx.strokeStyle = grad;
                        ctx.lineWidth = 0.5;
                        ctx.stroke();
                    }
                }
            }
        }

        function animate() {
            ctx.clearRect(0, 0, width, height);
            
            particles.forEach((p) => {
                p.update();
                p.draw();
            });
            
            drawLines();
            requestAnimationFrame(animate);
        }

        animate();
    }
});
