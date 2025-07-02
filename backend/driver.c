#include <fcntl.h>
#include <linux/uinput.h>
#include <string.h>
#include <unistd.h>

#define ABS_MAXVAL 65535

void emit_event(int fd, int type, int code, int value) {
  struct input_event ev;
  ev.type = type;
  ev.code = code;
  ev.value = value;

  write(fd, &ev, sizeof(ev));
}

void setup_abs(int fd, int code, int minimum, int maximum, int resolution) {
  ioctl(fd, UI_SET_ABSBIT, code);

  struct uinput_abs_setup abs_setup;
  memset(&abs_setup, 0, sizeof(abs_setup));
  abs_setup.code = code;
  abs_setup.absinfo.value = 0;
  abs_setup.absinfo.minimum = minimum;
  abs_setup.absinfo.maximum = maximum;
  abs_setup.absinfo.fuzz = 0;
  abs_setup.absinfo.flat = 0;
  abs_setup.absinfo.resolution = resolution;

  ioctl(fd, UI_ABS_SETUP, &abs_setup);
}

int create_stylus(void) {
  int fd = open("/dev/uinput", O_WRONLY | O_NONBLOCK);

  ioctl(fd, UI_SET_EVBIT, EV_SYN);
  ioctl(fd, UI_SET_PROPBIT, INPUT_PROP_DIRECT);

  ioctl(fd, UI_SET_EVBIT, EV_KEY);
  ioctl(fd, UI_SET_KEYBIT, BTN_TOOL_PEN);
  ioctl(fd, UI_SET_KEYBIT, BTN_TOOL_RUBBER);
  ioctl(fd, UI_SET_KEYBIT, BTN_STYLUS);
  ioctl(fd, UI_SET_KEYBIT, BTN_STYLUS2);
  ioctl(fd, UI_SET_KEYBIT, BTN_TOUCH);

  ioctl(fd, UI_SET_EVBIT, EV_MSC);
  ioctl(fd, UI_SET_MSCBIT, MSC_TIMESTAMP);

  ioctl(fd, UI_SET_EVBIT, EV_ABS);

  setup_abs(fd, ABS_X, 0, ABS_MAXVAL, 12);
  setup_abs(fd, ABS_Y, 0, ABS_MAXVAL, 12);
  setup_abs(fd, ABS_PRESSURE, 0, ABS_MAXVAL, 12);
  setup_abs(fd, ABS_TILT_X, -90, 90, 12);
  setup_abs(fd, ABS_TILT_Y, -90, 90, 12);

  struct uinput_setup setup;
  memset(&setup, 0, sizeof(setup));
  setup.id.bustype = BUS_VIRTUAL;
  setup.id.vendor = 0x186d;
  setup.id.product = 0x598f;
  strcpy(setup.name, "waytab");

  ioctl(fd, UI_DEV_SETUP, &setup);
  ioctl(fd, UI_DEV_CREATE);

  sleep(1);

  return fd;
}

void destroy_device(int fd) {
  ioctl(fd, UI_DEV_DESTROY);
  close(fd);
}
