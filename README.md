# HooEngine

#### Introduction

一个完成度很低的基于Rust和WGSL的玩具级别ECS图形引擎。即使加上“玩具级别”的限定词，它的完成度依然很低。


#### Note

前缀说明：

F: 平平无奇的结构体
E：枚举值
B：bitflag
R: 需要序列化的资源
T：trait（接口）
H：GamePlay 对象（UE 的前缀是 U，HooEngine 的前缀是 H，这很合理）


object 模块 (gamplay 模块)
整个 object 模块需要支持脚本绑定、反射和GC

context: 真正的顶层，里面多个 space。是 Rust 对象，但是有脚本接口。是脚本的入口。

space：整个逻辑世界。可以多个并存。有脚本接口给 manager 访问。
components, systems & entity: ECS. S和C可以是脚本对象（但脚本化必然没效率可言，不要脚本化高耗能模块）
managers: 可以是脚本对象。表示一个 space 中的全局逻辑。和space一一对应。用来创建、初始化和销毁 E, C, S
        部分rust manager是需要的，用来做 editor viewport 之类的小 space.

components 的脚本化仅限于定义数据结构！components 本身不包含任何逻辑！
另外还需要表示其他非 ECS 的物件，统称为 objects



关于用不用单例：
可以看依赖注入的相关文章找思路
决定是传递一个不可变的 HooEngineRef
不用先 new 再 initialize 的模式，因为 Rust 比较适合 RAII 的写法


模块之间如果要相互调用，尽量用最直接的方式，不然都可能引起 borrow_mut + borrow 困境
