import ui
import watch
from vec import Vector2

line = ""
buffer = []
history = []
history_item = None

def on_run_output(output):
    # print(output)
    buffer.extend(output.split('\n'))

def frame(frame):
    global line
    global buffer
    global history
    global history_item
    font_size = 6.0

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
        if history_item is not None: history_item = None
        history.append(line)
        watch.run(line)
        line = ""

    if frame.jpressed("backspace"):
        line = line[:-1]
    
    if frame.jpressed("up"):
        if history_item is None:
            history_item = len(history) - 1
        elif history_item > 0:
            history_item -= 1
        line = history[history_item]

    if frame.jpressed("down"):
        if history_item is not None:
            if history_item + 1 < len(history):
                history_item += 1
            line = history[history_item]

    if ui.in_rect(frame.mouse_pos(), Vector2(0, 0), watch.image_size(ui.icons['cross'])):
        if frame.click():
            return False
    return True
