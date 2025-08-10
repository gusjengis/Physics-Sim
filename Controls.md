# Controls & Keymap

#### WARNING 
This is AI generated, I have yet to verify this information.

## 🖱 Mouse

**2D view** (`simulation.d3 == false`)
- **Left click + drag**: marquee-select (hold, drag, release).
- **Left click (tap)**: send a “click” action to the sim (pick/act on hit under cursor).
- **Left drag + release**: impart velocity based on drag vector.
- **Shift + Left drag**: **pan** the world (`x_off`, `y_off`).
- **Mouse wheel**: zoom in/out (exponential, 2^±1 per notch).

**3D view** (`simulation.d3 == true`)
- **Left click**: capture the mouse for freelook.
- **Move mouse (while captured)**: look around (yaw/pitch).
- **Esc**: release mouse capture.
- *(Wheel zoom disabled in 3D.)*

**Create mode**
- **Left click**: place **one** particle at cursor world position.
- **Hold Right mouse**: “paint” particles continuously while held.

---

## ⌨ Keyboard

**Simulation & selection**
- **Space** — Start/stop simulation.
- **Ctrl + A** — Select all.
- **D** — `drop()` selection.
- **F** — `fix()` selection.

**State (quick save/restore)**
- **B** — Backup current sim state.
- **Shift + R** — Reset sim to fresh state.
- **R** — Restore from last backup.

**File I/O**
- **Ctrl + S** — Save to current file.
- **Ctrl + O** — Load file from UI dialog.
- **Ctrl + C / Ctrl + V** — Copy/paste in egui text fields.

**Camera / View**
- **=** — Zoom in (2D).
- **-** — Zoom out (2D).
- **H** — Home view (reset pan; in 3D also resets camera).
- **F11** — Toggle fullscreen.
- **M** — Toggle Settings menu panel.
- **L** — Toggle highlight overlay.

**3D movement** (mouse captured)
- **W / A / S / D** — Move forward / left / back / right.

**Simulation rate**
- **Right Arrow (→)** — Increase generations per frame.
- **Left Arrow (←)** — Decrease generations per frame (or bump `generations` counter if at min).

**Debug**
- **F3** — Toggle FPS/bench logging in console.

---

## 💡 Tips / Behavior Notes

- **Drag-throw** — In 2D, left-drag release sends a normalized delta to `release()`—short, fast drags “throw” more.
- **Pan vs Select** — **Shift** modifies left-drag into **pan**; without Shift it selects.
- **Create mode** — Left-click to place once; hold right to “spray” particles while moving.
- **Home** — Resets pan (and 3D camera if applicable).
- **Mouse capture (3D)** — Click to capture, **Esc** to release.
