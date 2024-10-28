#include <windows.h>

BOOL WINAPI DllMainCRTStartup(HINSTANCE dll, DWORD reason, LPVOID reserved) {
    (void)dll; (void)reason; (void)reserved;
    return TRUE;
}
