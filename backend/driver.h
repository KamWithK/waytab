#include <fcntl.h>
#include <linux/uinput.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#define ABS_MAXVAL 65535

void emit_event(int fd, int type, int code, int value);

int create_stylus(void);

void destroy_device(int fd);
