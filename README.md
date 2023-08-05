前缀说明：

F: 平平无奇的结构体
E：枚举值
B：bitflag
R: 需要序列化的资源
T：trait（接口）
H：GamePlay 对象（UE 的前缀是 U，HooEngine 的前缀是 H，这很合理）

无前缀：数学类



关于用不用单例：
可以看依赖注入的相关文章找思路
决定是传递一个不可变的 HooEngineRef
不用先 new 再 initialize 的模式，因为 Rust 比较适合 RAII 的写法


模块之间如果要相互调用，尽量用最直接的方式，不然都可能引起 borrow_mut + borrow 困境