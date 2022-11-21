#pragma once

#include <kime_engine.h>

typedef struct StrBuf {
  char *ptr;
  size_t len;
  size_t cap;
} StrBuf;

StrBuf str_buf_new();
void str_buf_delete(StrBuf *buf);
void str_buf_set_str(StrBuf *buf, KimeRustStr s);
void str_buf_set_ch(StrBuf *buf, uint32_t ch);
