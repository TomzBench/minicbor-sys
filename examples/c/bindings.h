#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define VERSION 2

typedef enum CBOR_KEY {
  PERSON = 0,
} CBOR_KEY;

typedef struct person {
  uint8_t name[8];
  uint8_t id;
} person;

int32_t mcbor_encode(uint8_t *dst, uint32_t dstlen, enum CBOR_KEY key, const void *ptr);

uint32_t mcbor_len(enum CBOR_KEY key, const void *ptr);

int32_t encode_person(uint8_t *dst, uint32_t dstlen, const struct person *src);

int32_t decode_person(struct person *dst, const uint8_t *bytes, uint32_t len);

int32_t decode_person_w_errmsg(struct person *dst,
                               uint8_t *errmsg,
                               uint32_t *errmsg_len,
                               const uint8_t *bytes,
                               uint32_t len);
