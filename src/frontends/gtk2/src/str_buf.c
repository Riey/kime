#include "./str_buf.h"
#include <gtk/gtk.h>
#include <stdlib.h>
#include <string.h>

#define INIT_CAP 128

StrBuf str_buf_new() {
  StrBuf buf;

  buf.ptr = (char *)calloc(INIT_CAP, 1);
  buf.len = 0;
  buf.cap = INIT_CAP;

  return buf;
}

void str_buf_delete(StrBuf *buf) { free(buf->ptr); }

void str_buf_set_str(StrBuf *buf, KimeRustStr s) {
  if (s.len >= buf->cap) {
    buf->cap = s.len + 1;
    buf->ptr = realloc(buf->ptr, buf->cap);
  }

  memcpy(buf->ptr, s.ptr, s.len);
  buf->len = s.len;
  buf->ptr[s.len] = '\0';
}

void str_buf_set_ch(StrBuf *buf, uint32_t ch) {
  gint len = g_unichar_to_utf8(ch, buf->ptr);
  buf->ptr[len] = '\0';
  buf->len = len;
}
