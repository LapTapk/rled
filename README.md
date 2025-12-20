# Little Erlang Decompiler written in Rust
This project is aimed to enhance my programming skills in general.
# Developing version 1.0
The development of this version will focus on:
- expanding the architecture to allow for more flexible manipulation of the syntax tree
- some algorithms that will use this architecture, such as:
    - combining X0 = "str" and func(X0) into func("str")
    - more readable branching
    - and other algorithms that come to mind
# Example
BEAM code
```
{move,{literal,"FLAG"},{x,0}},
{call_ext,1,{extfunc,os,getenv,1}},
{gc_bif,length,{f,0},1,[{y,2}],{y,1}},
{move,{x,0},{y,0}},
{test,is_ge,
	 {f,8},
	 [{tr,{y,1},{t_integer,{0,288230376151711743}}},
	  {integer,2}]},
{put_list,{y,0},nil,{x,1}}
{move,{literal,"~s"},{x,0}},
{call_ext_last,2,{extfunc,io,format,2},3},
{label,8},
{move,{literal,":(\n"},{x,0}},
{call_ext_last,1,{extfunc,io,format,1},3}
```
v0 pseudo code 
```
X0 = "FLAG"
X0 = getenv(X0)
Y0 = X0
Y1 = length(Y2)

if y1 >= 2: goto label8   
X1 = [Y0]
X0 = "~s"
return io:format(X0, X1)

label8:
X0 = ":(\n"
return io:format(X0)
```
v1 pseudo code
```
Y0 = getenv("FLAG")
Y1 = length(Y2)

if y1 >= 2:
    return io:format("~s", [Y0])
else: 
    return io:format(":(\n")
```
