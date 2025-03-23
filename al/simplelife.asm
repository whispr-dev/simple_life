; SimpleLife update_grid demonstration program
; This demonstrates the grid update function for the SimpleLife cellular automaton

section .data
    ; Constants for the growth function
    half:     dd 0.5, 0.5, 0.5, 0.5      ; Constant 0.5 for 4 cells
    two:      dd 2.0, 2.0, 2.0, 2.0      ; Constant 2.0 for 4 cells
    one:      dd 1.0, 1.0, 1.0, 1.0      ; Constant 1.0 for 4 cells
    zero:     dd 0.0, 0.0, 0.0, 0.0      ; Constant 0.0 for 4 cells
    
    ; Test data - a 4x4 grid (16 cells) for simplicity
    grid_size: dd 16                     ; Number of cells in our test grid
    dt:       dd 0.1                     ; Time step value
    
    ; Output messages
    msg_before: db "Grid values before update:", 10, 0
    msg_after:  db "Grid values after update:", 10, 0
    msg_val_fmt: db "%.2f ", 0           ; Format for printing float values
    msg_newline: db 10, 0                ; Newline character
    
section .bss
    ; Our test data arrays
    grid:      resd 16                   ; 16 float values for grid
    potential: resd 16                   ; 16 float values for potential
    
section .text
    global main
    extern printf                        ; We'll use C's printf to display results
    
main:
    ; Standard function prologue
    push rbp
    mov rbp, rsp
    
    ; Initialize our test data
    call init_test_data
    
    ; Display initial grid values
    lea rdi, [msg_before]
    call printf
    call print_grid
    
    ; Call the update_grid function
    lea rdi, [grid]                      ; First parameter: grid pointer
    lea rsi, [potential]                 ; Second parameter: potential pointer
    mov edx, [grid_size]                 ; Third parameter: size
    movss xmm0, [dt]                     ; Fourth parameter: dt value
    shufps xmm0, xmm0, 0                 ; Broadcast dt to all elements
    call update_grid
    
    ; Display updated grid values
    lea rdi, [msg_after]
    call printf
    call print_grid
    
    ; Standard function epilogue
    mov rsp, rbp
    pop rbp
    xor eax, eax                         ; Return 0 (success)
    ret

; Function to initialize test data with some sample values
init_test_data:
    push rbp
    mov rbp, rsp
    
    ; Initialize our grid with some test values between 0 and 1
    mov dword [grid + 0*4],  0x3F000000  ; 0.5 in IEEE 754 single-precision
    mov dword [grid + 1*4],  0x3F400000  ; 0.75
    mov dword [grid + 2*4],  0x3F666666  ; 0.9
    mov dword [grid + 3*4],  0x3F800000  ; 1.0
    mov dword [grid + 4*4],  0x3F400000  ; 0.75
    mov dword [grid + 5*4],  0x3F000000  ; 0.5
    mov dword [grid + 6*4],  0x3E800000  ; 0.25
    mov dword [grid + 7*4],  0x3F000000  ; 0.5
    mov dword [grid + 8*4],  0x3E800000  ; 0.25
    mov dword [grid + 9*4],  0x3F400000  ; 0.75
    mov dword [grid + 10*4], 0x3F000000  ; 0.5
    mov dword [grid + 11*4], 0x3E800000  ; 0.25
    mov dword [grid + 12*4], 0x3F800000  ; 1.0
    mov dword [grid + 13*4], 0x3F666666  ; 0.9
    mov dword [grid + 14*4], 0x3F400000  ; 0.75
    mov dword [grid + 15*4], 0x3F000000  ; 0.5
    
    ; Initialize potential values - these would normally be calculated
    ; from the grid using the kernel, but we'll use test values directly
    mov dword [potential + 0*4],  0x3F000000  ; 0.5
    mov dword [potential + 1*4],  0x3F400000  ; 0.75
    mov dword [potential + 2*4],  0x3F000000  ; 0.5
    mov dword [potential + 3*4],  0x3E800000  ; 0.25
    mov dword [potential + 4*4],  0x3F000000  ; 0.5
    mov dword [potential + 5*4],  0x3F400000  ; 0.75
    mov dword [potential + 6*4],  0x3F000000  ; 0.5
    mov dword [potential + 7*4],  0x3E800000  ; 0.25
    mov dword [potential + 8*4],  0x3F000000  ; 0.5
    mov dword [potential + 9*4],  0x3F400000  ; 0.75
    mov dword [potential + 10*4], 0x3F000000  ; 0.5
    mov dword [potential + 11*4], 0x3E800000  ; 0.25
    mov dword [potential + 12*4], 0x3F000000  ; 0.5
    mov dword [potential + 13*4], 0x3F400000  ; 0.75
    mov dword [potential + 14*4], 0x3F000000  ; 0.5
    mov dword [potential + 15*4], 0x3E800000  ; 0.25
    
    mov rsp, rbp
    pop rbp
    ret

; Function to print the grid values in a readable format
print_grid:
    push rbp
    mov rbp, rsp
    sub rsp, 32                          ; Allocate shadow space for Windows calling convention
    
    mov ecx, 0                           ; Initialize counter
    
.loop:
    cmp ecx, [grid_size]                 ; Check if we've printed all values
    jge .done
    
    ; Print this grid cell value
    lea rdi, [msg_val_fmt]               ; Format string
    movss xmm0, [grid + ecx*4]           ; Load the float value
    cvtss2sd xmm0, xmm0                  ; Convert single to double precision for printf
    mov eax, 1                           ; Specify 1 vector register is used
    call printf
    
    ; Add a newline every 4 values (for our 4x4 grid)
    inc ecx
    test ecx, 3                          ; Check if counter is divisible by 4
    jnz .continue
    
    lea rdi, [msg_newline]
    xor eax, eax
    call printf
    
.continue:
    jmp .loop
    
.done:
    ; Add a final newline if we didn't just print one
    test ecx, 3
    jz .skip_newline
    
    lea rdi, [msg_newline]
    xor eax, eax
    call printf
    
.skip_newline:
    add rsp, 32                          ; Clean up shadow space
    mov rsp, rbp
    pop rbp
    ret

; The original update_grid function
update_grid:
    ; Save non-volatile registers
    push rbx
    
    ; Initialize loop counter
    xor rcx, rcx
    
    ; Load constants into SSE registers
    movaps xmm1, [half]                  ; xmm1 = 0.5 (vector)
    movaps xmm2, [two]                   ; xmm2 = 2.0 (vector)
    movaps xmm3, [one]                   ; xmm3 = 1.0 (vector)
    movaps xmm4, [zero]                  ; xmm4 = 0.0 (vector)
    movaps xmm5, xmm0                    ; xmm5 = dt (vector, broadcasted)
    
.loop:
    ; Check if we've processed all elements
    cmp rcx, rdx
    jge .done
    
    ; Make sure we don't go past the end of our arrays
    mov rax, rdx
    sub rax, rcx
    cmp rax, 4
    jge .process_four
    
    ; If we have fewer than 4 elements left, process them one at a time
    jmp .process_one_at_a_time
    
.process_four:
    ; Load 4 potential values at once
    movaps xmm6, [rsi + rcx * 4]         ; xmm6 = potential[rcx:rcx+3]
    
    ; Calculate 2*u*(1-u) - 0.5 for 4 values at once
    movaps xmm7, xmm3                    ; xmm7 = 1.0
    subps xmm7, xmm6                     ; xmm7 = 1.0 - u
    mulps xmm7, xmm6                     ; xmm7 = u * (1.0 - u)
    mulps xmm7, xmm2                     ; xmm7 = 2.0 * u * (1.0 - u)
    subps xmm7, xmm1                     ; xmm7 = 2.0 * u * (1.0 - u) - 0.5
    
    ; Calculate dt * growth
    mulps xmm7, xmm5                     ; xmm7 = dt * growth
    
    ; Load current grid values
    movaps xmm8, [rdi + rcx * 4]         ; xmm8 = grid[rcx:rcx+3]
    
    ; Update grid: grid += dt * growth
    addps xmm8, xmm7                     ; xmm8 = grid + dt * growth
    
    ; Clamp to [0, 1]
    maxps xmm8, xmm4                     ; xmm8 = max(grid, 0.0)
    minps xmm8, xmm3                     ; xmm8 = min(grid, 1.0)
    
    ; Store the result back to grid
    movaps [rdi + rcx * 4], xmm8
    
    ; Move to next 4 elements
    add rcx, 4
    jmp .loop
    
.process_one_at_a_time:
    ; Process remaining elements individually
    movss xmm6, [rsi + rcx * 4]          ; Load one potential value
    
    ; Calculate growth for this value
    movss xmm7, xmm3                     ; xmm7 = 1.0
    subss xmm7, xmm6                     ; xmm7 = 1.0 - u
    mulss xmm7, xmm6                     ; xmm7 = u * (1.0 - u)
    mulss xmm7, xmm2                     ; xmm7 = 2.0 * u * (1.0 - u)
    subss xmm7, xmm1                     ; xmm7 = 2.0 * u * (1.0 - u) - 0.5
    
    ; Calculate dt * growth
    mulss xmm7, xmm5                     ; xmm7 = dt * growth
    
    ; Load current grid value
    movss xmm8, [rdi + rcx * 4]          ; xmm8 = grid[rcx]
    
    ; Update grid: grid += dt * growth
    addss xmm8, xmm7                     ; xmm8 = grid + dt * growth
    
    ; Clamp to [0, 1]
    maxss xmm8, xmm4                     ; xmm8 = max(grid, 0.0)
    minss xmm8, xmm3                     ; xmm8 = min(grid, 1.0)
    
    ; Store the result back to grid
    movss [rdi + rcx * 4], xmm8
    
    ; Move to next element
    inc rcx
    cmp rcx, rdx
    jl .process_one_at_a_time
    
.done:
    pop rbx
    ret