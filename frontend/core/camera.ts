export class InteractiveCamera {
  public camX: number;
  public camY: number;
  public camZoom: number;

  public isDragging = false;
  public hasDragged = false;
  public startX = 0;
  public startY = 0;

  constructor(
    private canvas: HTMLCanvasElement,
    private mapWidth = 19200,
    private mapHeight = 10800,
    private leftHUDWidth = 680,
    private bottomTerminalHeight = 260
  ) {
    this.camX = mapWidth / 2;
    this.camY = mapHeight / 2;
    this.camZoom = 0.05;
  }

  // Centering look-at coordinate focal point
  public reset(visibleWidth: number, visibleHeight: number) {
    this.camZoom = Math.min(visibleWidth / this.mapWidth, visibleHeight / this.mapHeight);
    this.camX = this.mapWidth / 2;
    this.camY = this.mapHeight / 2;
  }

  public getViewportBounds() {
    const visibleWidth = window.innerWidth - this.leftHUDWidth;
    const visibleHeight = window.innerHeight - this.bottomTerminalHeight;
    const viewCenterX = 305 + visibleWidth / 2;
    const viewCenterY = 90 + visibleHeight / 2;
    return { visibleWidth, visibleHeight, viewCenterX, viewCenterY };
  }

  public screenToWorld(clientX: number, clientY: number): { x: number; y: number } {
    const rect = this.canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    const { viewCenterX, viewCenterY } = this.getViewportBounds();

    const renderWidth = this.canvas.width / dpr;
    const renderHeight = this.canvas.height / dpr;

    const screenX = (clientX - rect.left) * (renderWidth / rect.width);
    const screenY = (clientY - rect.top) * (renderHeight / rect.height);

    const worldX = this.camX + (screenX - viewCenterX) / this.camZoom;
    const worldY = this.camY + (screenY - viewCenterY) / this.camZoom;

    return { x: worldX, y: worldY };
  }

  public setupEventListeners(
    onCheckHover: (worldX: number, worldY: number) => boolean,
    onClickEmptySpace: (worldX: number, worldY: number) => void
  ) {
    this.canvas.addEventListener("mousedown", (e) => {
      this.isDragging = true;
      this.hasDragged = false;
      this.startX = e.clientX;
      this.startY = e.clientY;
      this.canvas.style.cursor = "grabbing";
    });

    this.canvas.addEventListener("mousemove", (e) => {
      if (this.isDragging) {
        const dx = e.clientX - this.startX;
        const dy = e.clientY - this.startY;

        if (Math.abs(dx) > 3 || Math.abs(dy) > 3) {
          this.hasDragged = true;
        }

        this.camX -= dx / this.camZoom;
        this.camY -= dy / this.camZoom;

        this.camX = Math.max(0, Math.min(this.mapWidth, this.camX));
        this.camY = Math.max(0, Math.min(this.mapHeight, this.camY));

        this.startX = e.clientX;
        this.startY = e.clientY;
        this.canvas.style.cursor = "grabbing";
      } else {
        const { x, y } = this.screenToWorld(e.clientX, e.clientY);
        const hovered = onCheckHover(x, y);
        this.canvas.style.cursor = hovered ? "pointer" : "grab";
      }
    });

    this.canvas.addEventListener("mouseup", () => {
      this.isDragging = false;
      this.canvas.style.cursor = "grab";
    });

    this.canvas.addEventListener("mouseleave", () => {
      this.isDragging = false;
      this.canvas.style.cursor = "default";
    });

    this.canvas.addEventListener("wheel", (e) => {
      e.preventDefault();

      const { viewCenterX, viewCenterY, visibleWidth, visibleHeight } = this.getViewportBounds();
      const rect = this.canvas.getBoundingClientRect();
      const mouseX = e.clientX - rect.left;
      const mouseY = e.clientY - rect.top;

      const logMouseX = this.camX + (mouseX - viewCenterX) / this.camZoom;
      const logMouseY = this.camY + (mouseY - viewCenterY) / this.camZoom;

      const intensity = 0.12;
      const direction = e.deltaY < 0 ? 1 : -1;
      const factor = Math.exp(direction * intensity);

      const minZoom = Math.min(visibleWidth / this.mapWidth, visibleHeight / this.mapHeight) * 0.8;
      const maxZoom = 2.0;

      this.camZoom = Math.max(minZoom, Math.min(maxZoom, this.camZoom * factor));

      this.camX = logMouseX - (mouseX - viewCenterX) / this.camZoom;
      this.camY = logMouseY - (mouseY - viewCenterY) / this.camZoom;

      this.camX = Math.max(0, Math.min(this.mapWidth, this.camX));
      this.camY = Math.max(0, Math.min(this.mapHeight, this.camY));
    }, { passive: false });

    this.canvas.addEventListener("click", (e) => {
      if (this.hasDragged) return;
      const { x, y } = this.screenToWorld(e.clientX, e.clientY);
      onClickEmptySpace(x, y);
    });
  }
}
