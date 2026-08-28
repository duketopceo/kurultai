/* Registers the Kurultai Brain surface with the right canvas.
 *
 * The canvas store discovers surfaces only through its JS extension points:
 *   await callJsExtensions("surfaces_register", store)
 * Without this registration, $store.rightCanvas.open('kurultai') finds no
 * surface and silently no-ops (no tab, no rail button, no panel).
 *
 * The panel markup itself is provided by the right-canvas-panels HTML
 * extension (webui/right-canvas-panels/_10_kurultai_panel.html).
 */
export default function registerKurultaiSurface(canvasStore) {
  try {
    canvasStore?.registerSurface?.({
      id: "kurultai",
      title: "Kurultai Brain",
      icon: "psychology",
      order: 40,
    });
  } catch (error) {
    console.error("Kurultai surface registration failed", error);
  }
}
