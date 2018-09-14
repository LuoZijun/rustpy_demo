#coding: utf8

def add(a, b):
    r = a + b
    print("Py Add: %d" % r)
    return r

def f(pixels_get, pixels_len):
    # 这两个函数参数来自于 Rust
    print("Pixels Length: %d" % pixels_len() )
    print("Index 101: %d" % pixels_get(101) )
    print("Index 102: %d" % pixels_get(102) )
    print("Index 103: %d" % pixels_get(103) )
    print("Index 104: %d" % pixels_get(104) )

