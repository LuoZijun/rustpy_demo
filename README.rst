Rust 执行 Python 代码
=======================

:Date: 9/24/ 2018

.. contents::

案例介绍
---------

在 Rust 层面创建一个静态数据，并为 Python 注入操作这个静态数据的方法。
如何在 Python 代码里面调用这个 Rust 方法得到数据。

.. Note::
    
    如果直接把静态数据以 `PyObject` 的形式传递给 Python 函数，则会花费将近 1 秒钟的时间。
    这就是为什么给 Python 提供函数来操作静态数据在性能上面的表现会更好。


运行
---------

*   因为 `python37` 和 `python36` 以及之前的几个版本存在一些 `ABI` 上的差异，所以请确保你安装了 `Python 3.7` 。

.. code:: bash
    
    # 如果已经安装了 Python37 版本，则忽略。
    brew install python@37 # apt install python37

    # 如果已安装每日构建版，则忽略.
    rustup toolchain install nightly
    
    git clone https://github.com/Luozijun/rust_py_demo.git
    cd rust_py_demo

    rustup default nightly
    cargo run
