
.text

label1:  /* comment */
    LDS r0, data1
    LD R3, 0xffffffffffffffff
    LD2S r0, [r5, 0x12]
    LD4S r0, [r5, 0x12]
    LDS r0, [r5, 0x12]
    LD r0, [r5, 0x12]
    LD1 r0, [r5, 0x12]
    LD2 r0, [r5, 0x12]
    LD4 r0, [r5, 0x12]


lablewfij__2:
    ADD R0, R1, R2  // line comment
    ADD R0, R1, '\0'
    SUB R0, R1, RZR
    SUB R0, R1, 129
    MUL R0, R1, R2
    MUL R6, R2, 0b10001011
    DIV R3, R0, R1
    DIV R3, R0, 0xff
    HALT

.data

data1:
.char '0', ' ', '~' ,'"'
.1b 0x0, 0xff
.2b 3429
.4b 2147483647
.8b 9223000000000000000
data2: .string "123456", "0123456789987"
