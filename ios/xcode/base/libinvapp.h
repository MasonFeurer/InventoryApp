#ifndef libinvapp_h
#define libinvapp_h

#include <stdint.h>

struct native_app;

struct ios_view_obj
{
    void *view;
    void *metal_layer; // CAMetalLayer
    int maximum_frames;
    void (*open_keyboard)();
    void (*close_keyboard)();
};

struct native_app *create_app(struct ios_view_obj object);
void draw_frame(struct native_app *data);
void event_touch_begin(struct native_app *data, float x, float y);
void event_touch_move(struct native_app *data, float x, float y);
void event_touch_end(struct native_app *data, float x, float y);
void event_text_input(struct native_app *data, char *bytes, int bytes_len);
void event_key_typed_backspace(struct native_app *data);

#endif
