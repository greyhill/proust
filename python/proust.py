import ctypes as ct
import numpy as np
import ctypes.util

libpath = ctypes.util.find_library('proust')
if libpath is None:
    libpath = './libproust.so'
lib = ct.CDLL(libpath)

size_t = ct.c_size_t

class DPtr(object):
    def __init__(self, ptr, dtor):
        self.ptr = ct.c_voidp(ptr)
        self.dtor = dtor

    def __del__(self):
        self.dtor(self.ptr)

def num_platforms():
    lib.Proust_NumPlatforms.restype = size_t
    return int(lib.Proust_NumPlatforms())

def context_num_devices(context):
    lib.Proust_ContextNumDevices.restype = size_t
    return int(lib.Proust_ContextNumDevices(context.ptr))

def create_command_queue(context, device_number = 0):
    lib.Proust_CreateCommandQueue.restype = ct.c_voidp
    ptr = lib.Proust_CreateCommandQueue(\
            context.ptr,
            size_t(device_number))
    return DPtr(ptr, lib.Proust_DestroyCommandQueue)

def create_context(platform_number = 0):
    lib.Proust_CreateContextWithAllDevices.restype = ct.c_voidp
    ptr = lib.Proust_CreateContextWithAllDevices(size_t(platform_number))
    tr = DPtr(ptr, lib.Proust_DestroyContext)
    tr.num_devices = lambda: context_num_devices(tr)
    tr.create_command_queue = lambda device_number: create_command_queue(tr, device_number)
    return tr


