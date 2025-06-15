# Waytab

Draw with Android tablets on Linux

## Design goals

1. Split into a backend driver and frontend app which should be easily installable and detect each other on the network
2. Share a single window to tablet, and force resize to the right size
3. Fully support all touch and stylus features such as multitouch, stylus tilt and pressure
4. Wayland support

Eventually a native app will be made, but initial prototypes use a website.

## Inspiration

Taken heavy inspiration from the work off:

- [Weylus](https://github.com/H-M-H/Weylus)
- [GfxTablet](https://github.com/rfc2822/GfxTablet)
