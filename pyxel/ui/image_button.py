import pyxel

from .button import Button
from .ui_constants import (
    BUTTON_DISABLED_COLOR,
    BUTTON_ENABLED_COLOR,
    BUTTON_PRESSED_COLOR,
)


class ImageButton(Button):
    """
    Events:
        __on_press()
        __on_release()
    """

    def __init__(self, parent, x, y, img, sx, sy, **kwargs):
        super().__init__(parent, x, y, 7, 7, **kwargs)

        self._img = img
        self._sx = sx
        self._sy = sy

        self.add_event_handler("draw", self.__on_draw)

    def __on_draw(self):
        color = (
            BUTTON_PRESSED_COLOR
            if self.is_pressed
            else (BUTTON_ENABLED_COLOR if self.is_enabled else BUTTON_DISABLED_COLOR)
        )

        pyxel.pal(BUTTON_ENABLED_COLOR, color)
        pyxel.blt(
            self._x,
            self._y,
            self._img,
            self._sx,
            self._sy,
            self._width,
            self._height,
            0,
        )
        pyxel.pal()
