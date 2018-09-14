#![feature(duration_as_u128)]
#![allow(unused_imports)]

#[macro_use]
extern crate cpython;

use cpython::{ Python, PyObject, PyDict, PyList, PyString, PyResult };
use cpython::{ PythonObject, ObjectProtocol };

use std::env;
use std::time;
use std::thread;


fn python_init(py: Python) -> PyResult<()> {
    let sys = py.import("sys")?;

    // import sys
    // sys.path.append("/home/my/path")
    sys.get(py, "path")?
        .extract::<PyObject>(py)?
        .call_method(py, 
                    "append",
                    (env::current_dir().unwrap().to_str().unwrap(), ),
                    None
        )?;

    // sys.get(py, "path")?.extract::<PyList>(py)?.insert_item(py,
    //                         0usize,
    //                         PyString::new(py, env::current_dir().unwrap().to_str().unwrap()
    // ).into_object());

    let locals = PyDict::new(py);
    locals.set_item(py, "os", py.import("os")?)?;
    locals.set_item(py, "m", py.import("m")?)?;
    
    let user: String = py.eval("os.getenv('USER') or os.getenv('USERNAME')", None, Some(&locals))?.extract(py)?;
    
    let version: String = sys.get(py, "version")?.extract(py)?;
    println!("Hello {}, I'm Python {}\n\n", user, version);
    
    Ok(())
}

static mut PIXELS: [u8; 6220800] = [0u8; 1920*1080*3];

fn pixels_get(_py: Python, index: usize) -> PyResult<u8> {
    unsafe {
        Ok(PIXELS[index])
    }
}

fn pixels_length(_py: Python) -> PyResult<usize> {
    unsafe {
        Ok(PIXELS.len())
    }
}


fn demo(py: Python) -> PyResult<()> {
    // 初始化一些 像素数据
    unsafe {
        PIXELS[101] = 110u8;
        PIXELS[102] = 155u8;
        PIXELS[103] = 255u8;
        PIXELS[104] = 250u8;
    }

    let locals = PyDict::new(py);

    let f1 = py_fn!(py, pixels_get(index: usize));
    let f2 = py_fn!(py, pixels_length() );
    locals.set_item(py, "pixels_get", f1)?;
    locals.set_item(py, "pixels_len", f2)?;
    locals.set_item(py, "m", py.import("m")?)?;
    
    // 执行表达式
    println!("Rust Res: {:?}",
                py.eval("m.add(5, 6)",
                None,
                Some(&locals))?.extract::<i64>(py)?);
    // 执行脚本
    py.run("import m\nm.add(5, 6)", None, Some(&locals))?;

    println!("\n");

    let now = time::Instant::now();

    let f1 = py_fn!(py, pixels_get(index: usize));
    let f2 = py_fn!(py, pixels_length());

    let m = py.import("m")?;
    m.get(py, "f")?
        .call(py, (f1, f2,), None )?;

    py.run("print(pixels_get(100))", None, Some(&locals))?;
    py.run("print(pixels_get(102))", None, Some(&locals))?;
    py.run("print(pixels_get(103))", None, Some(&locals))?;
    py.run("print(pixels_len())", None, Some(&locals))?;

    println!("duration: {}", now.elapsed().as_millis());

    Ok(())
}


fn main() {
    let gil = Python::acquire_gil();

    python_init(gil.python()).unwrap();

    demo(gil.python()).unwrap();
}



