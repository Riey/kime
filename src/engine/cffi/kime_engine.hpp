#pragma once

/* Generated with cbindgen:0.17.0 */

/* DO NOT MODIFY THIS MANUALLY */

#include <cstdint>
#include <cstddef>

namespace kime {

enum class InputResultType {
  Bypass,
  ToggleHangul,
  ClearPreedit,
  Preedit,
  Commit,
  CommitBypass,
  CommitPreedit,
  CommitCommit,
};

struct Config;

struct InputEngine;

struct InputResult {
  InputResultType ty;
  uint32_t char1;
  uint32_t char2;
};

using ModifierState = uint32_t;
static const ModifierState ModifierState_CONTROL = (uint32_t)1;
static const ModifierState ModifierState_SUPER = (uint32_t)2;
static const ModifierState ModifierState_SHIFT = (uint32_t)4;
static const ModifierState ModifierState_ALT = (uint32_t)8;

extern "C" {

/// Create new engine
InputEngine *kime_engine_new();

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

/// Get preedit_char of engine
///
/// ## Return
///
/// valid ucs4 char NULL to represent empty
uint32_t kime_engine_preedit_char(const InputEngine *engine);

/// Reset preedit state then returm commit char
///
/// ## Return
///
/// valid ucs4 char NULL to represent empty
uint32_t kime_engine_reset(InputEngine *engine);

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
void kime_config_xim_preedit_font(const Config *config,
                                  const uint8_t **name,
                                  uintptr_t *len,
                                  double *font_size);

} // extern "C"

} // namespace kime
