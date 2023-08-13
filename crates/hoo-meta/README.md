# HooMeta JS 绑定

!!! 暂不支持由 Js 实现 Rust Trait

1. 有两类对象：Rs值对象 和 Rs引用对象 (`tests::tests::nested_access`)
   1. Rs引用对象在 Rust 侧一般以 RcObject<T> 表示。例外是：构造函数返回值 `Self` 和 成员方法的第一个参数 `&self` 或 `&mut self` 也是 Rs 引用对象
   2. RcObject 只能包裹结构体，不可以包裹 整数、字符串 等基础类型
   3. Rs值对象在跨越语言边界时值传递
   4. Rs引用对象在跨越语言边界时引用传递
2. 构造
   1. 所有字段都是 `pub` 类型的结构体，可以构造 Rs值对象。否则，只能构造 Rs引用对象
   2. Rs值对象 在 JavaScript -> Rust 语言边界中支持鸭子类型到具体类型的自动转换，因此通过填写字段做构造
3. GC与析构 (`tests::tests::tests::tests::garbage_collection`)
   1. Rs值对象 不存在跨越语言边界的引用，不存在生命周期互通问题
   2. 语言边界两侧的 Rs引用对象 的引用同时持有所有权，在两侧都允许释放时才会释放对象：
      1. 在 JavaScript 侧，GC 机制决定该对象可以释放
      2. 在 Rust 侧，reference counting 机制认为该对象没有再被引用
4. 成员函数 (`tests::tests::fields_and_methods`)
   1. Rs值对象 没有成员函数
   2. Rs引用对象 的成员函数在 Rust 侧声明为第一个参数是 `&self` 或 `&mut self` 的函数 （重复：1.1）
5. 类型自动转换 (`tests::tests::auto_conversion`)
   1. Rs值对象 在 JavaScript -> Rust 的语言边界中支持鸭子类型到具体类型的自动转换 （重复：1.2）
   2. Rs值对象 不会 被自动转换成 Rs引用对象（1.1 所述例外除外）
   3. Rs引用对象 在跨越 JavaScript -> Rust 的语言边界时，会被自动转换为 Rs值对象