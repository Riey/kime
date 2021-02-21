#pragma once

/* Generated with cbindgen:0.17.0 */

/* DO NOT MODIFY THIS MANUALLY */

#include <stdint.h>
#include <stddef.h>

namespace kime {

struct Config;

struct InputEngine;

struct RustStr {
  const uint8_t *ptr;
  uintptr_t len;
};

using InputResult = uint32_t;
static const InputResult InputResult_CONSUMED = (uint32_t)1;
static const InputResult InputResult_LANGUAGE_CHANGED = (uint32_t)2;
static const InputResult InputResult_HAS_PREEDIT = (uint32_t)4;
static const InputResult InputResult_NEED_RESET = (uint32_t)8;
static const InputResult InputResult_NEED_FLUSH = (uint32_t)16;

using ModifierState = uint32_t;
static const ModifierState ModifierState_CONTROL = (uint32_t)1;
static const ModifierState ModifierState_SUPER = (uint32_t)2;
static const ModifierState ModifierState_SHIFT = (uint32_t)4;
static const ModifierState ModifierState_ALT = (uint32_t)8;

struct XimPreeditFont {
  RustStr name;
  double size;
};

extern "C" {

/// Return API version
uintptr_t kime_api_version();

/// Create new engine
InputEngine *kime_engine_new(const Config *config);

/// Set hangul enable state
void kime_engine_set_hangul_enable(InputEngine *engine, bool mode);

/// Delete engine
///
/// # Safety
///
/// engine must be created by `kime_engine_new` and never call delete more than once
void kime_engine_delete(InputEngine *engine);

/// Update hangul state
void kime_engine_update_hangul_state(InputEngine *engine);

/// Get commit string of engine
///
/// ## Return
///
/// valid utf8 string
RustStr kime_engine_commit_str(InputEngine *engine);

/// Get preedit string of engine
///
/// ## Return
///
/// valid utf8 string
RustStr kime_engine_preedit_str(InputEngine *engine);

void kime_engine_flush(InputEngine *engine);

/// Reset preedit state then returm commit char
void kime_engine_reset(InputEngine *engine);

/// Press key when modifier state
///
/// ## Return
///
/// input result
InputResult kime_engine_press_key(InputEngine *engine,
                                  const Config *config,
                                  uint16_t hardware_code,
                                  ModifierState state);

/// Load config from local file
Config *kime_config_load();

/// Create default config note that this function will not read config file
Config *kime_config_default();

/// Delete config
void kime_config_delete(Config *config);

/// Get xim_preedit_font config
/// name only valid while config is live
///
/// ## Return
///
/// utf-8 string when len
XimPreeditFont kime_config_xim_preedit_font(const Config *config);

} // extern "C"

} // namespace kime
