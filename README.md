# GarbageCollector3
A short game (more like a proof-of-concept) for OLC CodeJam 2024
Using [pixamo](https://github.com/InfiniteCoder01/pixamo) for sprites and [ldtk-codegen](https://github.com/InfiniteCoder01/ldtk-codegen) for level codegen
A lot of art was done by [Nigel](https://github.com/bhavyakukkar/)

You can play the game right now on [itch.io](https://infinitecoder.itch.io/garbagecollector3)
Or GitHub pages version: https://infinitecoder.org/GarbageCollector3

Ideas:
- More levels
- Better terminal: 1) Cursor, move/delete -by-word, Home/End 2) Clear command 3) Colors?
- Sounds
- Send message to Messages app when easter egg found

## Instructions for coders and modders
Scripting in the game is powered by [RustPython](https://github.com/RustPython/RustPython) with freeze-stdlib
Which means, `json`, `zlib` and a lot of other modules are available. I'm not sure about networking, but from what I've tested,
there is `urllib` (no `urllib.request`) and `ipaddress`
Additionally, vec module is provided, source code can be found [here](https://github.com/larryhastings/vec)

All the in-game API is located in the watch module, which can be imported with `import watch`
It provides following functions:
```
watch.set_weather(weather: str) # Set the weather in the world, weather can be "sunny", "rainy" and "snowy"
watch.run(code: str) # Run python code in user's scope (same scope repl is in) [will only be ran at the start of the next frame]
watch.print(message: str) # Print with built-in printer (printed text can be captured using on_run_output), print function is using this under the hood
watch.lock_nearest() # Lock all the doors in 80 pixel radius (technically, square) [will only be ran at the start of the next frame]
watch.unlock_nearest() # Unlock all the doors in 80 pixel radius (technically, square) [will only be ran at the start of the next frame]
watch.add_app(module: str) # Add an app to the watch, module being the name of the module for the app [will only be available at the start of the next frame]
# Instead of module, it can be any object in user's scope that has attribute frame. App API will be discussed later
watch.load_image(image_data: bytes) -> int # Load an image from data, returning image handle. [will only be available at the start of the next frame]
# Data is raw file bytes, for example, a PNG image of a white cross:
# b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x08\x00\x00\x00\x08\x08\x06\x00\x00\x00\xc4\x0f\xbe\x8b\x00\x00\x00\x01sRGB\x00\xae\xce\x1c\xe9\x00\x00\x00>IDAT\x18\x95\x85\x8d1\n\x000\x0c\x025k>\x90\xff\xbf\xae\x1f\xe8\x9cL\x01\x91B\x9d\xe4D\xe5\xed\xd3\x00\x90,B\xb4<\x1c\xb8\xa7\x03U\xb2\xc8WK/\xc3[\xae\xff\x85\x86;\xab,<t?\xa0~#\xc2\x1a\xf8\x9b\x9d\x00\x00\x00\x00IEND\xaeB`\x82'
# You can get the bytes printed by running https://github.com/InfiniteCoder01/GarbageCollector3/blob/main/apps/assets/convert.py on the file
watch.image_size(image: int) -> vec.Vector2 # Returns the size of an image that was previously loaded
```

### App API
Apps have to have a function called `frame`. It will be called every frame when the app is open. It takes an object of type `watch.Frame` and returns either True or False,
True if the app continues running and False if the app exits.
Here is an example of an app:
```
import ui
import watch
from vec import Vector2

class App:
    def frame(self, frame):
        frame.draw_image(Vector2(0, 0), ui.icons['cross'])
        if ui.in_rect(frame.mouse_pos(), Vector2(0, 0), watch.image_size(ui.icons['cross'])):
            if frame.click():
                return False
        return True

app = App()
watch.add_app('app')
```

Additionally, apps can have `on_run_output` function, which can capture any output (prints, exception messages) from a code that was ran with `watch.run`:
```
def on_run_output(output: str):
    print(output)
```

`watch.Frame` object is used to interact with the current frames. It provides the following methods:
```
watch.Frame.click() -> bool # Returns true if there was a mouse click in this frame
watch.Frame.mouse_pos() -> vec.Vector2 # Returns mouse position, relative to watch screen (which is 128x128 pixels in size)
watch.Frame.typed_text() -> str # Returns text typed in this frame
watch.Frame.pressed(key: str) -> bool # Returns true if the key is pressed, throws NameEror if the key name is invalid
watch.Frame.jpressed(key: str) -> bool # Returns true if the key became pressed this frame, throws NameEror if the key name is invalid
# Valid key names: "enter", "backspace", "delete", "left" (arrow), "right" (arrow), "up" (arrow), "down" (arrow), "home", "end", "page_up", "page_down", "escape"
watch.Frame.ctrl() -> bool # Returns true if Ctrl key is pressed this frame
watch.Frame.shift() -> bool # Returns true if Shift key is pressed this frame
watch.Frame.alt() -> bool # Returns true if Alt/Meta key is pressed this frame
watch.Frame.draw_image(position: vec.Vector2, image: int) # Draws image in the specified position (position is relative to watch screen)
watch.Frame.draw_image_pro(position: vec.Vector2, image: int, size: vec.Vector2 | None, uv_tl: vec.Vector2, uv_br: vec.Vector2) # Drawsimage in the specified position (position is relative to watch screen)
# If `size` is not None, will scale the image to size. `uv_tl` and `uv_br` are normalized UV coordinates of a portion of an image to draw (vec.Vector2(0, 0), vec.Vector2(1, 1) for full image)
watch.Frame.draw_text(position: vec.Vector2, text: str, size: float, color: int) # Draws text in the specified position (position is relative to watch screen)
# with the specified size (which matches character's height). Color is a hex representation of a color without an alpha channel (0xff0000 for red, 0x0000ff for blue, 0xffffff for white)
```

Note: font used to render text is 7:12 Serif by [Christian Munk] (https://www.1001fonts.com/users/christianmunk/) (monospace)

There is also `ui` module, which aims to simplify creation of apps. It provides the following functionality:
```
ui.icons # A set of default icons, includes: "cross"
ui.in_rect(pos: vec.Vector2, tl: vec.Vector2, size: vec.Vector2) -> bool # Returns true, if `pos` is inside of the rectangle defined by top-left point `tl` and `size`
```

Look at built-in apps for examples: https://github.com/InfiniteCoder01/GarbageCollector3/tree/main/apps

Here is a single-line that can be pasted into the terminal to create a blank app (without exit button):
```
import watch; watch.run('from vec import Vector2\nclass Test:\n  def frame(self, frame): frame.draw_text(Vector2(0, 0), "Test", 14.0, 0xFFFFFF); return True\ntest = Test();\nwatch.add_app("test")')
```

## Licenses
Code is licensed under MIT license
Except apps/vec.py, which is licensed under https://github.com/larryhastings/vec/blob/master/LICENSE
Assets (everything inside assets and apps/assets directories) are licensed under Creative Commons Zero v1.0 Universal
Except assets/712_serif.ttf, which is `The FontStruction “7:12 Serif” (http://fontstruct.com/fontstructions/show/243645) by “CMunk” is licensed under a Creative Commons Attribution Share Alike license (http://creativecommons.org/licenses/by-sa/3.0/).
