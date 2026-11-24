const canvas = document.querySelector("#showcase");
const startup = document.querySelector("#startup");
const status = document.querySelector("#status");
const retry = document.querySelector("#retry");
let app;

status.textContent = "Loading Temivu Showcase…";
try {
    const { default: init, WebShowcase } = await import("./pkg/showcase.js");
    await init();
    // Egui antialiases its shapes. A single-sampled WebGL2 back buffer also
    // lets the glass renderer copy the live scene into its blur textures.
    canvas.getContext("webgl2", { antialias: false });
    app = new WebShowcase();
    await app.start(
        canvas,
        matchMedia("(prefers-reduced-motion: reduce)").matches,
        matchMedia("(pointer: coarse)").matches,
    );
    startup.hidden = true;
} catch {
    status.textContent = "Temivu Showcase could not start. Check your connection and WebGL support.";
    retry.hidden = false;
    retry.addEventListener("click", () => location.reload());
}
