#ifndef _TREELVS
#define _TREELVS

#include "libipvs.h"
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>


extern int init_ipvs();
extern char * ipvs_error();

extern void * create_service(char * addr, int port, char * alg);
extern int add_service(void * svc_v);
extern int remove_service(void * svc_v);

extern void * create_dest(char *dst, int port);
extern int add_dest(void * svc_v, void *dst_v);
extern int remove_dest(void * svc_v, void *dst_v);



#endif