.entry main

.func main 0 false
.locals 3
.local 0 8
.local 1 21
ldarg 0
ldarg 1
call name1
starg 2
breakpoint
call random
call random
add.u
breakpoint
ret

.func name1 2 true
.locals 3
ldarg 0
ldarg 1
add.u
call inc
starg 2
ret

.func inc 1 true
.locals 3
.local 2 1
ldarg 0
ldarg 2
add.u
starg 1
ret

.import random 0 true