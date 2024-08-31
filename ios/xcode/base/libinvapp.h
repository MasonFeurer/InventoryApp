#ifndef libinvapp_h
#define libinvapp_h

#include <stdint.h>

struct native_app;

struct ios_view_obj
{
    void *view;
    // CAMetalLayer
    void *metal_layer;
    int maximum_frames;
    void (*callback_to_swift)(int32_t arg);
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
