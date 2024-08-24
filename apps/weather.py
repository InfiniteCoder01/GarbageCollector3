import ui
import watch
from vec import Vector2


def frame(frame):
    frame.draw_image(Vector2(0, 0), ui.icons['cross'])
    frame.draw_text(Vector2(
        0, 16), "Hello, hello, can you hear me, voice inside my head? Hello, hello, I believe you, how can I forget?", 14.0, 0xFFFFFF)

    if ui.in_rect(frame.mouse_pos(), Vector2(0, 0), watch.image_size(ui.icons['cross'])):
        if frame.click():
            return False
    return True
