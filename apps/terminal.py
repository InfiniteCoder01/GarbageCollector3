import ui
import watch
from vec import Vector2

line = ""
buffer = []

def on_run_output(output):
    print(output)
    buffer.extend(output.split('\n'))

def frame(frame):
    global line
    global buffer
    font_size = 7.0

    while len(buffer) > (128.0 - 16.0) / font_size:
        buffer.pop(0)

    frame.draw_image(Vector2(0, 0), ui.icons['cross'])
    cursor = Vector2(0, 16)
    for bline in buffer:
        frame.draw_text(cursor, bline, font_size, 0xffffff)
        cursor = Vector2(0.0, cursor.y + font_size)
    frame.draw_text(cursor, "> " + line, font_size, 0xffffff)

    line += frame.typed_text()
    if frame.jpressed("enter"):
        buffer.append("> " +  line)
        watch.run(line)
        line = ""

    if ui.in_rect(frame.mouse_pos(), Vector2(0, 0), watch.image_size(ui.icons['cross'])):
        if frame.click():
            return False
    return True
