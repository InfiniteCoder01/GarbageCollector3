import ui
import watch
from vec import Vector2


def frame(frame):
    frame.draw_image(Vector2(0, 0), ui.icons['cross'])
    frame.draw_text(
        Vector2(0, 16), "This app is not yet implemented", 12.0, 0xFFFFFF)

    if ui.in_rect(frame.mouse_pos(), Vector2(0, 0), watch.image_size(ui.icons['cross'])):
        if frame.click():
            return False
    return True
