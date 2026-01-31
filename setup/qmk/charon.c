#include "charon.h"
#include "raw_hid.h"
#include QMK_KEYBOARD_H

/* Charon send messages
 * These are used to communicate with the Charon client.
 * The first byte is the message type, followed by data.
 * The size of a message is 32 bytes.
 */

enum CharonSendMessage {
    CHARON_MSG_LAYER_CHANGE = 0x02,
    CHARON_MSG_KEY_EVENT = 0x03,
    CHARON_MSG_CHANGE_MODE = 0x04,
    CHARON_MSG_TOGGLE_MODE = 0x05,
    CHARON_MSG_LAYER_CHUNK = 0x10,
    CHARON_MSG_SEND_KEYBOARD_INFO = 0x11,
};

void charon_send_layer_change(layer_state_t state, bool is_default) {
    uint8_t data[32] = {0};
    data[0] = CHARON_MSG_LAYER_CHANGE;
    data[1] = get_highest_layer(state);
    data[2] = (uint8_t)is_default;
    raw_hid_send(data, sizeof(data));
}

void charon_send_key_event(uint16_t keycode, keyrecord_t *record) {
    uint8_t data[32] = {
        CHARON_MSG_KEY_EVENT,
        (uint8_t)(keycode & 0xFF),
        (uint8_t)((keycode >> 8) & 0xFF),
        (uint8_t)record->event.pressed,
        record->event.key.row,
        record->event.key.col,
        0
    };
    raw_hid_send(data, sizeof(data));
}

void charon_send_mode_change(uint8_t mode) {
    uint8_t data[32] = {0};
    data[0] = CHARON_MSG_CHANGE_MODE;
    data[1] = mode;
    raw_hid_send(data, sizeof(data));
}

void charon_send_mode_toggle(void) {
    uint8_t data[32] = {0};
    data[0] = CHARON_MSG_TOGGLE_MODE;
    raw_hid_send(data, sizeof(data));
}

void charon_send_keyboard_info(void) {
    uint8_t data[32] = {0};
    data[0] = CHARON_MSG_SEND_KEYBOARD_INFO;
    data[1] = MATRIX_ROWS;
    data[2] = MATRIX_COLS;
    raw_hid_send(data, sizeof(data));
}

void charon_send_layer_chunk(uint8_t layer, uint16_t offset) {
    uint8_t data[32] = {0};
    data[0] = CHARON_MSG_LAYER_CHUNK;
    data[1] = layer;
    data[2] = offset >> 8;
    data[3] = offset & 0xFF;

    // pack up to N entries per packet (pick what fits your 32 bytes)
    // example: send 12 keycodes as 2 bytes each = 24 bytes payload
    for (int i = 0; i < 12; i++) {
        uint16_t pos = offset + i;
        uint8_t row = pos / MATRIX_COLS;
        uint8_t col = pos % MATRIX_COLS;
        if (row >= MATRIX_ROWS) break;

        uint16_t kc = keymap_key_to_keycode(layer, (keypos_t){ .row=row, .col=col });

        // little-endian 2 bytes per keycode
        data[4 + i*2 + 0] = (uint8_t)(kc & 0xFF);
        data[4 + i*2 + 1] = (uint8_t)(kc >> 8);
    }

    raw_hid_send(data, sizeof(data));
}

/* -----------------------------------------------------------
 * Charon receive messages
 * These are used to handle incoming messages from the Charon client.
 * The first byte is the message type, followed by data.
 * The size of a message is 32 bytes.
 */


enum CharonRecvMessage {
    CHARON_REQ_ECHO = 0x01,
    CHARON_REQ_LAYER_CHUNK = 0x10,
    CHARON_REQ_KEYBOARD_INFO = 0x11,
};

void charon_raw_hid_receive(uint8_t *data, uint8_t len) {
    if (len < 1) return;

    switch (data[0]) {
        case CHARON_REQ_ECHO: {
            raw_hid_send(data, len);
            break;
        }
        case CHARON_REQ_LAYER_CHUNK: {
            uint8_t layer = data[1];
            uint16_t offset = (data[2] << 8) | data[3];
            charon_send_layer_chunk(layer, offset);
            break;
        }
        case CHARON_REQ_KEYBOARD_INFO: {
            charon_send_keyboard_info();
            break;
        }
    }
}
