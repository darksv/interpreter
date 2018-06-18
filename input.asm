.func main 0 false
.locals 3
ldarg 0
ldarg 1
call 1
starg 2
breakpoint
ret

.func name1 2 true
.locals 3
ldarg 0
ldarg 1
add
starg 2
ret