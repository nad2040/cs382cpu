.text

label1:  /* comment */

lablewfij__2:
    ADD R0 R1 R2 
    ADD R0 R1 '\0' 
    SUB R0 R1 R2 
    SUB R0 R1 129
    MUL R0 R1 R2 
    MUL R16 R8 0b10001011
    DIV R3 R0 R1
    DIV R3 R0 0xff
    

.data

data1: .string "129039", "0139090319031"

.char '0', ' ', '~' ,'"'
.1b 0x1f, 0xbd
.2b 3429
.4b 2147483647
.8b 9223000000000000000
    
