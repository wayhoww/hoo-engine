pub trait TComponent {
    // 目前的做法很不好，属于是能跑起来就行的程度
    // 需要考虑以下几点：
    // 1. 一个 Component 实例的类型 id 应当是始终不变的
    // 2. 可以避免 Component 类型重复
    // 3. 可以动态增加 Component 类型，用于脚本侧调用
    // 4. 在 Rust 侧可以安全高效做类型检查
    fn component_name(&self) -> &'static str;      
}