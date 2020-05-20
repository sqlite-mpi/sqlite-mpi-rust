#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef char *CRetIJSONPtr;

typedef const char *CReqJSON;

typedef char *CRetOJSONPtr;

typedef void (*COutFn)(CRetOJSONPtr);

void smpi_free_string(char *ptr);

CRetIJSONPtr smpi_input(CReqJSON c_req);

void smpi_start(COutFn c_out_fn);

void smpi_stop(void);
