#include "treelvs.h"

static int modprobe_ipvs(void)
{
    char *argv[] = { "/sbin/modprobe", "--", "ip_vs", NULL };
    int child;
    int status;
    int rc;

    if (!(child = fork())) {
        execv(argv[0], argv);
        exit(1);
    }

    rc = waitpid(child, &status, 0);

    if (!WIFEXITED(status) || WEXITSTATUS(status)) {
        return 1;
    }

    return 0;
}

extern int init_ipvs()
{
    int result = 0;
    result = ipvs_init();
    if (result)
    {
        modprobe_ipvs();
        result = ipvs_init();
        if (result)
        {
            return result;
        }
    }
    return result;
}

extern void close_ipvs()
{
    ipvs_close();
}

extern char * ipvs_error()
{
    return ipvs_strerror(errno);
}

extern void * create_service(char * addr, int port, char * alg)
{
    ipvs_service_t *svc;
    svc = (ipvs_service_t *)malloc(sizeof(ipvs_service_t));
    memset(svc, 0, sizeof(ipvs_service_t));
    struct in_addr inaddr;

    inet_aton(addr, &inaddr);
    svc->addr.ip = inaddr.s_addr;
    svc->af = AF_INET;
    svc->port = htons(port);
    svc->protocol = IPPROTO_TCP;
    strcpy(svc->sched_name, alg);
    svc->netmask = ((u_int32_t) 0xffffffff);
//    svc->flags |= IP_VS_SVC_F_PERSISTENT;
//    svc->timeout = 1300;
    return (void *)svc;
}

extern int add_service(void * svc_v)
{
    ipvs_service_t * svc;
    svc = (ipvs_service_t *) svc_v;
    return ipvs_add_service(svc);
}

extern int remove_service(void * svc_v)
{
    int result;
    result = 0;
    ipvs_service_t * svc;
    svc = (ipvs_service_t *) svc_v;
    result = ipvs_del_service(svc);
    if(!result)
    {
        free(svc);
    }

    return result;
}

extern void * create_dest(char *dst, int port)
{
    ipvs_dest_t *dest;
    dest = (ipvs_dest_t *)malloc(sizeof(ipvs_dest_t));
    memset(dest, 0, sizeof(ipvs_dest_t));
    int result = 0;
    struct in_addr inaddr;
    inet_aton(dst, &inaddr);
    dest->addr.ip = inaddr.s_addr;
    dest->port = htons(port);
    dest->af = AF_INET;
    dest->conn_flags = IP_VS_CONN_F_MASQ;
    dest->weight = 1;
    return (void*)dest;
}

extern int add_dest(void * svc_v, void *dst_v)
{
    ipvs_service_t * svc;
    int result = 0;
    ipvs_dest_t * dest;
    dest = (ipvs_dest_t *)dst_v;
    svc = (ipvs_service_t *) svc_v;
    result = ipvs_add_dest(svc, dest);
    if (result)
    {
        return result;
    }
    result = ipvs_update_service(svc);
    free(dest);
    return result;
}

extern int remove_dest(void * svc_v, void *dst_v)
{
    ipvs_service_t * svc;
    int result = 0;
    ipvs_dest_t * dest;
    dest = (ipvs_dest_t *)dst_v;
    svc = (ipvs_service_t *) svc_v;
    result = ipvs_del_dest(svc, dest);
    if (result)
    {
        return result;
    }
    result = ipvs_update_service(svc);
    free(dest);
    return result;
}