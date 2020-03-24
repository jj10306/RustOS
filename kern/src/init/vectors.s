.global context_save
context_save:
    // FIXME: Save the remaining context to the stack.
    
    //save the link register
    stp     lr, x29, [SP, #-16]!
    //pass parameters in registers x0-x3
    mov     x0, x29
    mrs     x1, ESR_EL1
    mov     x2, xzr
    //call handle_exception
    bl      handle_exception
    //restore link register
    ldp     lr, x29, [SP], #16

.global context_restore
context_restore:
    // FIXME: Restore the context from the stack.
    ret

.macro HANDLER source, kind
    .align 7
    stp     lr, xzr, [SP, #-16]!
    stp     x28, x29, [SP, #-16]!
    
    
    mov     x29, \source
    movk    x29, \kind, LSL #16
    bl      context_save
    
    ldp     x28, x29, [SP], #16
    ldp     lr, xzr, [SP], #16
    eret
.endm
    
.align 11
.global vectors
vectors:
    // FIXME: Setup the 16 exception vectors.
    HANDLER 0, 0
    HANDLER 0, 1
    HANDLER 0, 2
    HANDLER 0, 3
    HANDLER 1, 0
    HANDLER 1, 1
    HANDLER 1, 2
    HANDLER 1, 3
    HANDLER 2, 0
    HANDLER 2, 1
    HANDLER 2, 2
    HANDLER 2, 3
    HANDLER 3, 0
    HANDLER 3, 1
    HANDLER 3, 2
    HANDLER 3, 3


