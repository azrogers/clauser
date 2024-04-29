#pragma once

#define CL_WRAP_CALL(FunctionCall)                                                                           \
    if (!FunctionCall) {                                                                                     \
        status.GetError().Log();                                                                             \
        return false;                                                                                        \
    }